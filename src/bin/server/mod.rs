use std::{collections::HashSet, sync::{Arc, Mutex}, net::SocketAddr};

use dox_me_daddy::{opts::{EventOpts}, error::{DoxMeDaddyError}, socket::Socket};

use futures_channel::mpsc::unbounded;
use log::{info};
use tokio::net::{TcpListener, TcpStream};
use futures_util::{future, stream::TryStreamExt, StreamExt};


type PeerMap = Arc<Mutex<HashSet<Socket>>>;

async fn handle_socket(id: usize, stream: TcpStream, addr: SocketAddr, peer_map: PeerMap) -> Result<(), DoxMeDaddyError> {
    info!("New Websocket Server Connection");

    let websocket = tokio_tungstenite::accept_async(stream).await?;
    let (outgoing, incoming) = websocket.split();
    let (tx, rx) = unbounded();

    match peer_map.lock() {
        Ok(mut peer_map) => {
            peer_map.insert(Socket {
                addr,
                tx: tx.clone(),
                is_prime: false,
                id
            });
        },
        _ => { }
    };

    let incoming_msg = incoming.try_for_each(|msg| {
        info!("msg: {:?}", msg);
        return future::ok(());
    });

    let outgoing_msg = rx.map(Ok).forward(outgoing);

    future::select(incoming_msg, outgoing_msg).await;
    info!("Websocket has disconnected {}", id);

    return Ok(());
}

pub async fn spawn_server(opts: EventOpts) -> Result<(), DoxMeDaddyError> {
    let server = TcpListener::bind(format!("{}:{}", opts.addr, opts.port)).await?;
    let peer_map: PeerMap = Arc::new(Mutex::new(HashSet::new()));

    info!("TcpListener created on {}:{}", opts.addr, opts.port);
    let mut id = 0;
    while let Ok((stream, client_addr)) = server.accept().await {
        id += 1;
        tokio::spawn(handle_socket(id, stream, client_addr, peer_map.clone()));
    }

    return Ok(());
}
