use std::time::Duration;

use dotenv::dotenv;
use dox_me_daddy::{error::DoxMeDaddyError, opts::ServerOpts};
use futures::{future, pin_mut};
use futures_util::StreamExt;
use log::info;
use reqwest::Url;
use structopt::StructOpt;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() -> Result<(), DoxMeDaddyError> {
    dotenv().expect("dotenv to work");
    env_logger::init();

    let opts = ServerOpts::from_args();
    let url = format!("ws://{}:{}", opts.addr, opts.port);

    let (socket, _) = connect_async(Url::parse(url.as_str()).unwrap()).await?;

    let (outgoing, incoming) = socket.split();
    let (tx, rx) = futures_channel::mpsc::unbounded();

    let write_outgoing = rx.map(Ok).forward(outgoing);
    let read_incoming = incoming.for_each(|msg| async move {
        info!("Message Received {:?}", msg);
        /*
         * NOTE: uncomment to just send back the messages I got
        inner_tx
            .unbounded_send(msg.unwrap().clone())
            .expect("transactions to be successful");
        */
    });

    pin_mut!(write_outgoing, read_incoming);
    tokio::spawn(async move {
        loop {
            std::thread::sleep(Duration::from_secs(5));
            tx.unbounded_send(Message::Text("hello".to_string())).expect("test client send failed");
        }
    });
    future::select(write_outgoing, read_incoming).await;

    return Ok(());
}
