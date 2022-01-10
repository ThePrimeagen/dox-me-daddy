use log::error;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::connect;
use url::Url;

use crate::error::DoxMeDaddyError;
use crate::event::Event;

pub struct Quirk {
    pub join_handle: JoinHandle<()>,
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

impl Quirk {
    pub fn new(tx: UnboundedSender<Event>, quirk_token: String) -> Quirk {
        let url = format!("wss://websocket.quirk.tools?access_token={}", quirk_token);
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
                    let event = Event::QuirkMessage(text);
                    match tx.send(event) {
                        Ok(_) => {}
                        Err(e) => {
                            error!("There was an error with Quirk#tx.send: {:?}", e);
                        }
                    }
                }
            }

            return ();
        });

        return Quirk {
            join_handle,
        };
    }
}
