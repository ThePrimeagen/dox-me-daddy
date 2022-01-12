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

    let (_, mut incoming) = socket.split();

    while let Some(msg) = incoming.next().await {
        println!("Message Received {:?}", msg);
    };

    return Ok(());
}
