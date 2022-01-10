mod server;

use dox_me_daddy::fan::FanIn;
use dox_me_daddy::forwarder::{ReceiverGiver, ReceiverTaker, connect};
use dox_me_daddy::pipeline::Pipeline;
use dox_me_daddy::quirk::Quirk;
use dox_me_daddy::{error::DoxMeDaddyError, opts::ServerOpts};


use structopt::StructOpt;
use dotenv::dotenv;
use log::{warn};

use crate::server::Server;

#[tokio::main]
async fn main() -> Result<(), DoxMeDaddyError> {
    dotenv().expect("dotenv to work");
    env_logger::init();

    warn!("Starting Application");

    let opts = ServerOpts::from_args();
    let mut to_pipeline = FanIn::new();
    let mut server = Server::new(&opts).await?;
    let mut quirk = Quirk::new().await?;

    to_pipeline.take(&mut server)?;
    to_pipeline.take(&mut quirk)?;

    let mut pipeline = Pipeline::new(to_pipeline, &opts);

    tokio::spawn(connect(pipeline.take_receiver(), server.tx.clone()));

    // Until the server dies, we ride
    // future::select(server.join_handle, pipeline.join_handle).await;
    server.join_handle.await?;

    warn!("Application Ended, You Lose");

    return Ok(());
}
