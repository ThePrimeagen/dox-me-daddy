use dotenv::dotenv;
use dox_me_daddy::{error::DoxMeDaddyError, opts::EventOpts};
use futures::{pin_mut, future};
use log::info;
use reqwest::Url;
use structopt::StructOpt;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt};


#[tokio::main]
async fn main() -> Result<(), DoxMeDaddyError> {
    dotenv().expect("dotenv to work");
    env_logger::init();

    let opts = EventOpts::from_args();
    let url = format!("ws://{}:{}", opts.addr, opts.port);

    let (socket, _) =
        connect_async(Url::parse(url.as_str()).unwrap()).await?;

    let (outgoing, incoming) = socket.split();
    let (tx, rx) = futures_channel::mpsc::unbounded();

    let write_outgoing = rx.map(Ok).forward(outgoing);
    let inner_tx = tx.clone();
    pin_mut!(inner_tx);
    let read_incoming = incoming.for_each(|msg| async {
        info!("Message Received {:?}", msg);
        inner_tx.unbounded_send(msg.unwrap().clone()).expect("transactions to be successful");
    });

    pin_mut!(write_outgoing, read_incoming);
    tx.unbounded_send(Message::Text("hello".to_string()))?;
    future::select(write_outgoing, read_incoming).await;

    return Ok(());
}
