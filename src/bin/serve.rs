mod server;

use dox_me_daddy::fan::{FanInAsync};
use dox_me_daddy::forwarder::{ReceiverGiverAsync, ReceiverTakerAsync, connect_async};
use dox_me_daddy::pipeline::Pipeline;
use dox_me_daddy::quirk::Quirk;
use dox_me_daddy::transforms::debug::DebugTransform;
use dox_me_daddy::transforms::quirk::{QuirkTransform, QuirkFilterTransform};
use dox_me_daddy::transforms::twitch::TwitchTransform;
use dox_me_daddy::twitch::Twitch;
use dox_me_daddy::{error::DoxMeDaddyError, opts::ServerOpts};

use dotenv::dotenv;
use futures::future;
use log::warn;
use structopt::StructOpt;

use crate::server::Server;

#[tokio::main]
async fn main() -> Result<(), DoxMeDaddyError> {
    dotenv().expect("dotenv to work");
    env_logger::init();

    warn!("Starting Application");

    let opts = ServerOpts::from_args();
    let mut to_pipeline = FanInAsync::new();

    let mut server = Server::new(&opts).await?;
    let mut quirk = Quirk::new(&opts).await?;
    let mut twitch = Twitch::new(&opts).await;

    to_pipeline.take(&mut server)?;
    to_pipeline.take(&mut quirk)?;
    to_pipeline.take(&mut twitch)?;

    let mut pipeline = Pipeline::new(to_pipeline, &opts);
    pipeline.add_transformer(Box::new(DebugTransform { pre_message: "Pre Pipeline".to_string() }));
    pipeline.add_transformer(Box::new(TwitchTransform));
    pipeline.add_transformer(Box::new(QuirkTransform));
    pipeline.add_transformer(Box::new(QuirkFilterTransform));
    pipeline.add_transformer(Box::new(DebugTransform { pre_message: "Post Pipeline".to_string() }));

    tokio::spawn(connect_async(pipeline.take_receiver(), server.tx.clone(), "top level serve".to_string()));

    // Until the server dies, we ride
    let quirk_and_twitch = future::select(quirk.join_handle, twitch.join_handle);
    let server_and_pipeline = future::select(server.join_handle, pipeline.join_handle);

    future::select(
        quirk_and_twitch,
        server_and_pipeline,
    ).await;

    warn!("Application Ended, You Lose");

    return Ok(());
}
