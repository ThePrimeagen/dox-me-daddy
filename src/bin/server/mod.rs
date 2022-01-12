use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use dox_me_daddy::{
    error::DoxMeDaddyError,
    fan::FanIn,
    forwarder::{Forwarder, ForwarderEvent, ReceiverGiver},
    opts::ServerOpts,
    simple_forwarder, simple_receiver_giver,
    socket::Socket,
};

use futures_channel::mpsc::unbounded;
use futures_util::{future, stream::TryStreamExt, StreamExt};
use log::{info, error};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::unbounded_channel,
    task::JoinHandle,
};

type PeerMap = Arc<Mutex<HashMap<usize, Socket>>>;
type TokioUSender = tokio::sync::mpsc::UnboundedSender<ForwarderEvent>;
type TokioUReceiver = tokio::sync::mpsc::UnboundedReceiver<ForwarderEvent>;

async fn handle_socket(
    id: usize,
    stream: TcpStream,
    addr: SocketAddr,
    peer_map: PeerMap,
    fan_in: Arc<Mutex<FanIn>>,
) -> Result<(), DoxMeDaddyError> {
    let websocket = tokio_tungstenite::accept_async(stream).await?;
    let (outgoing, incoming) = websocket.split();
    let (inbound_tx, inbound_rx) = unbounded();
    let (outbound_tx, outbound_rx) = unbounded_channel::<ForwarderEvent>();

    fan_in
        .lock()
        .expect("FanIn lock to never fail")
        .take_raw(outbound_rx);

    // TODO: I bet a weak ptr to socket and converting a socket to a Forwarder
    // would allow for me to have a better fan-out method
    match peer_map.lock() {
        Ok(mut peer_map) => {
            info!("attaching ws {} to peer_map which has {} peers", id, peer_map.len());
            peer_map.insert(
                id,
                Socket {
                    addr,
                    tx: inbound_tx.clone(),
                    is_prime: false, // TODO: I am still wondering about this...
                    id,
                },
            );
        }
        _ => {
            error!("Unable to attach {} to peer_map, lock failed", id);
        }
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

        outbound_tx
            .send(ForwarderEvent::WebsocketMessage(msg))
            .expect("Socket#outbound_tx to never fail");

        return future::ok(());
    });

    let outgoing_msg = inbound_rx.map(|e| {
        info!("socket#inbound_rx({}) -> {:?}", id, e);
        Ok(e)
    }).forward(outgoing);

    future::select(incoming_msg, outgoing_msg).await;
    info!("Websocket has disconnected {}", id);
    match peer_map.lock() {
        Ok(mut peer_map) => {
            info!("removing {} from peer_map", id);
            peer_map.insert(
                id,
                Socket {
                    addr,
                    tx: inbound_tx.clone(),
                    is_prime: false, // TODO: I am still wondering about this...
                    id,
                },
            );
        }
        _ => {
            error!("Unable to detach {} to peer_map, lock failed", id);
        }
    };

    return Ok(());
}

async fn handle_websocket_to_server(
    mut rx: TokioUReceiver,
    peer_map: PeerMap,
) -> Result<(), DoxMeDaddyError> {
    loop {
        info!("Server#handle_websocket_to_server waiting for message");
        if let Some(message) = rx.recv().await {
            info!("Server#handle_websocket_to_server got Some(message). {:?}", message);
            info!("Server#handle_websocket_to_server unlocking peer_map");
            for (id, peer) in peer_map.lock().expect("peer_map lock to never fail").iter() {
                info!("Server#handle_websocket_to_server#peer_map sending message to {}", id);
                peer.push(message.clone())?;
            }
        } else {
            error!("Server#handle_websocket_to_server got a non Some(message).  Restarting loop.");
        }
    }
}

pub struct Server {
    pub peer_map: PeerMap,
    pub join_handle: JoinHandle<()>,
    pub tx: TokioUSender,

    rx: Option<TokioUReceiver>,
}

simple_receiver_giver!(Server);
simple_forwarder!(Server);

impl Server {
    pub async fn new(opts: &ServerOpts) -> Result<Server, DoxMeDaddyError> {
        let server = TcpListener::bind(format!("{}:{}", opts.addr, opts.port)).await?;
        let peer_map: PeerMap = Arc::new(Mutex::new(HashMap::new()));

        // TODO: I don't know how to do them with future channels...
        // TODO: FanOut turns out to be harder...  I probably need to implement with WeakPointers.
        let fan_in_ws_messages = Arc::new(Mutex::new(FanIn::new()));
        let (to_server_sockets, to_server_sockets_receiver) =
            tokio::sync::mpsc::unbounded_channel::<ForwarderEvent>();

        info!("TcpListener created on {}:{}", opts.addr, opts.port);

        let peer_map_inner = peer_map.clone();
        let inner_fan_in = fan_in_ws_messages.clone();
        let join_handle = tokio::spawn(async move {
            let mut id = 1;
            info!("Waiting for connection");
            while let Ok((stream, client_addr)) = server.accept().await {
                info!("Spawning handler for connection {}", id);
                tokio::spawn(handle_socket(
                    id,
                    stream,
                    client_addr,
                    peer_map_inner.clone(),
                    inner_fan_in.clone(),
                ));
                id += 1;
            }
        });

        tokio::spawn(handle_websocket_to_server(
            to_server_sockets_receiver,
            peer_map.clone(),
        ));

        return Ok(Server {
            peer_map,
            join_handle,

            tx: to_server_sockets,
            rx: fan_in_ws_messages
                .lock()
                .expect("fan_in_ws_messages to never fail")
                .take_receiver(),
        });
    }
}
