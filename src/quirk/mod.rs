use log::error;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::connect;
use url::Url;

use crate::error::DoxMeDaddyError;
use crate::forwarder::{ForwarderEvent, ReceiverGiver};
use crate::simple_receiver_giver;

pub struct Quirk {
    pub join_handle: JoinHandle<()>,
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
    let request = RequestBody { access_token: token };

    let client = reqwest::Client::new();
    let res: ResponseBody = client.post("https://websocket.quirk.tools/token")
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
    pub async fn new() -> Result<Quirk, DoxMeDaddyError> {
        let quirk_token = get_quirk_token().await?;
        let url = format!("wss://websocket.quirk.tools?access_token={}", quirk_token);
        let (tx, rx) = unbounded_channel();

        let (mut socket, _) =
            connect(Url::parse(url.as_str()).unwrap())
                .expect("Can't connect");

        // first thing you should do: start consuming incoming messages,
        // otherwise they will back up.
        let join_handle: JoinHandle<()> = tokio::spawn(async move {
            while let Ok(msg) = socket.read_message() {
                if !msg.is_text() {
                    continue;
                }
                if let Ok(text) = msg.into_text() {
                    match tx.send(ForwarderEvent::QuirkMessage(text)) {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Quirk#tx unable to send. {:?}", e);
                            continue;
                        }
                    }
                }
            }
            return ();
        });

        return Ok(Quirk {
            join_handle,
            rx: Some(rx),
        });
    }
}
