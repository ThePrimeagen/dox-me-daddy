mod server;

use dox_me_daddy::forwarder::ReceiverGiver;
use dox_me_daddy::{error::DoxMeDaddyError, opts::EventOpts, forwarder::ForwarderEvent};

use structopt::StructOpt;
use dotenv::dotenv;
use log::{info, warn};

use crate::server::Server;

async fn mux_that(mut rx: tokio::sync::mpsc::UnboundedReceiver<ForwarderEvent>) {
    while let Some(message) = rx.recv().await {
        info!("Server pushed message: {:?}", message);
    }
}

#[tokio::main]
async fn main() -> Result<(), DoxMeDaddyError> {
    dotenv().expect("dotenv to work");
    env_logger::init();

    warn!("Starting Application");

    let opts = EventOpts::from_args();
    //let _quirk = Quirk::new(tx.clone(), get_quirk_token().await?);

    let mut server = Server::new(opts).await?;
    if let Some(rx) = server.take_receiver() {
        tokio::spawn(mux_that(rx));
    }

    server.join_handle.await?;

    warn!("Application Ended");

    return Ok(());
}
