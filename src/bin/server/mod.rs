use std::{collections::{HashMap}, sync::{Arc, Mutex}, net::SocketAddr, mem::swap};

use dox_me_daddy::{opts::{EventOpts}, error::{DoxMeDaddyError}, socket::Socket, forwarder::{Forwarder, ForwarderEvent, ReceiverGiver}};

use futures::SinkExt;
use futures_channel::mpsc::{unbounded};
use log::{info, error};
use tokio::{net::{TcpListener, TcpStream}, task::JoinHandle};
use futures_util::{future, stream::TryStreamExt, StreamExt};


type PeerMap = Arc<Mutex<HashMap<usize, Socket>>>;
type TokioUSender = tokio::sync::mpsc::UnboundedSender<ForwarderEvent>;
type TokioUReceiver = tokio::sync::mpsc::UnboundedReceiver<ForwarderEvent>;

async fn handle_socket<T: Forwarder>(id: usize, stream: TcpStream, addr: SocketAddr, peer_map: PeerMap, forwarder: T) -> Result<(), DoxMeDaddyError> {
    info!("New Websocket Server Connection");

    let websocket = tokio_tungstenite::accept_async(stream).await?;
    let (outgoing, incoming) = websocket.split();
    let (tx, rx) = unbounded();

    match peer_map.lock() {
        Ok(mut peer_map) => {
            peer_map.insert(id, Socket {
                addr,
                tx: tx.clone(),
                is_prime: true,
                id
            });
        },
        _ => { }
    };

    let incoming_msg = incoming.try_for_each(|msg| {
        // odd piece of logic here..
        {
            let peer_map = peer_map.lock().expect("peer_map.lock to never fail");
            if let Some(peer) = peer_map.get(&id) {
                if !peer.is_prime {
                    return future::ok(());
                }
                info!("prime message: {:?}", msg);
            } else {
                return future::ok(());
            }
        }

        match forwarder.push(ForwarderEvent::WebsocketMessage(msg)) {
            Ok(_) => {},
            Err(e) => {
                error!("Forwarder error from: {:?}", e);
            }
        }
        return future::ok(());
    });

    let outgoing_msg = rx.map(Ok).forward(outgoing);

    future::select(incoming_msg, outgoing_msg).await;
    info!("Websocket has disconnected {}", id);

    return Ok(());
}

async fn handle_websocket_to_server(mut rx: TokioUReceiver, peer_map: PeerMap) -> Result<(), DoxMeDaddyError> {
    while let Some(message) = rx.recv().await {
        for (_, peer) in peer_map.lock().expect("peer_map lock to never fail").iter() {
            peer.push(message.clone())?;
        }
    }

    return Ok(());
}

struct ServerForwarder {
    tx: TokioUSender
}

impl ServerForwarder {
    fn new(tx: TokioUSender) -> ServerForwarder {
        return ServerForwarder { tx };
    }
}

impl Forwarder for ServerForwarder {
    fn push(&self, event: ForwarderEvent) -> Result<(), DoxMeDaddyError> {
        self.tx.send(event).expect("ServerForwarder#tx should never fail");
        return Ok(());
    }
}

pub struct Server {
    pub peer_map: PeerMap,
    pub join_handle: JoinHandle<()>,

    tx: TokioUSender,
    rx: Option<TokioUReceiver>
}

impl Forwarder for Server {
    fn push(&self, event: ForwarderEvent) -> Result<(), DoxMeDaddyError> {
        self.tx.send(event).expect("this to never fail");
        return Ok(());
    }
}

impl ReceiverGiver for Server {
    fn take_receiver(&mut self) -> Option<tokio::sync::mpsc::UnboundedReceiver<ForwarderEvent>> {
        let mut rx: Option<TokioUReceiver> = None;
        swap(&mut self.rx, &mut rx);
        return rx;
    }

    fn give_receiver(&mut self, rx: Option<tokio::sync::mpsc::UnboundedReceiver<ForwarderEvent>>) {
        if let Some(rx) = rx {
            self.rx = Some(rx);
        }
    }
}

impl Server {
    pub async fn new(opts: EventOpts) -> Result<Server, DoxMeDaddyError> {
        let server = TcpListener::bind(format!("{}:{}", opts.addr, opts.port)).await?;
        let peer_map: PeerMap = Arc::new(Mutex::new(HashMap::new()));

        // TODO: I don't know how to do them with future channels...
        let (to_server_sockets, to_server_sockets_receiver) = tokio::sync::mpsc::unbounded_channel::<ForwarderEvent>();
        let (websocket_push, server_websocket_receive) = tokio::sync::mpsc::unbounded_channel::<ForwarderEvent>();

        info!("TcpListener created on {}:{}", opts.addr, opts.port);

        let peer_map_inner = peer_map.clone();
        let join_handle = tokio::spawn(async move {
            let mut id = 1;
            while let Ok((stream, client_addr)) = server.accept().await {
                let forwarder = ServerForwarder::new(websocket_push.clone());
                tokio::spawn(handle_socket(id, stream, client_addr, peer_map_inner.clone(), forwarder));

                id += 1;
            }
        });

        tokio::spawn(handle_websocket_to_server(to_server_sockets_receiver, peer_map.clone()));

        return Ok(Server {
            peer_map,
            join_handle,

            tx: to_server_sockets,
            rx: Some(server_websocket_receive),
        });
    }
}
