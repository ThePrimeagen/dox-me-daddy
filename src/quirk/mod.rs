
use futures::{StreamExt, TryStreamExt, future};
use log::{info};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async};

use url::Url;

use crate::error::DoxMeDaddyError;
use crate::forwarder::{ForwarderEvent, ReceiverGiver};
use crate::opts::ServerOpts;
use crate::simple_receiver_giver;

pub struct Quirk {
    pub join_handle: JoinHandle<Result<(), tokio_tungstenite::tungstenite::Error>>,
    rx: Option<UnboundedReceiver<ForwarderEvent>>,
}

#[derive(Deserialize, Serialize, Debug)]
struct RequestBody {
    access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseBody {
    access_token: String,
}

pub async fn get_quirk_token() -> Result<String, DoxMeDaddyError> {
    let token = std::env::var("QUIRK_TOKEN").expect("QUIRK_TOKEN should be an env variable");
    let request = RequestBody {
        access_token: token,
    };

    let client = reqwest::Client::new();
    let res: ResponseBody = client
        .post("https://websocket.quirk.tools/token")
        .json(&request)
        .header("Content-Type", "application/json")
        .send()
        .await?
        .json()
        .await?;

    return Ok(res.access_token);
}

simple_receiver_giver!(Quirk);

impl Quirk {
    pub async fn new(_opts: &ServerOpts) -> Result<Quirk, DoxMeDaddyError> {
        let quirk_token = get_quirk_token().await?;
        let url = format!("wss://websocket.quirk.tools?access_token={}", quirk_token);

        let (socket, _) = connect_async(Url::parse(url.as_str()).unwrap()).await.expect("Can't connect");
        let (_, incoming) = socket.split();
        let (tx, rx) = unbounded_channel();

        let join_handle = tokio::spawn(incoming.try_for_each(move |msg| {
            if !msg.is_text() {
                return future::ok(());
            }
            if let Ok(text) = msg.into_text() {
                tx.send(ForwarderEvent::QuirkMessageRaw(text)).expect("test");
            }
            return future::ok(());
        }));

        return Ok(Quirk {
            join_handle,
            rx: Some(rx),
        });
    }
}
