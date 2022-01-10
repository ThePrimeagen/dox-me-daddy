mod server;

use dox_me_daddy::{error::DoxMeDaddyError, opts::EventOpts, event::Event, quirk::{Quirk, get_quirk_token}};
use structopt::StructOpt;
use dotenv::dotenv;
use log::{warn, error};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::server::{spawn_server};

#[tokio::main]
async fn main() -> Result<(), DoxMeDaddyError> {
    dotenv().expect("dotenv to work");
    env_logger::init();

    warn!("Starting Application");

    let opts = EventOpts::from_args();
    let (tx, _rx): (UnboundedSender<Event>, UnboundedReceiver<Event>) = tokio::sync::mpsc::unbounded_channel();
    let _quirk = Quirk::new(tx.clone(), get_quirk_token().await?);
    match spawn_server(opts).await {
        Err(e) => {
            error!("spawn_server has failed. {:?}", e);
        },
        Ok(()) => {
            warn!("spawn_server has finished.");
        }
    }

    return Ok(());
}
