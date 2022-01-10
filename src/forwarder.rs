use log::{info, error};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_tungstenite::tungstenite::Message;
use twitch_irc::message::{ServerMessage, PrivmsgMessage};

use crate::error::DoxMeDaddyError;

type TokioUSender = tokio::sync::mpsc::UnboundedSender<ForwarderEvent>;
type TokioUReceiver = tokio::sync::mpsc::UnboundedReceiver<ForwarderEvent>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuirkReward {
    pub title: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuirkData {
    pub user_name: String,
    pub reward: QuirkReward,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuirkMessage {
    pub source: String,
    pub data: QuirkData,
}

impl QuirkMessage {
    pub fn is_twitch_event(&self) -> bool {
        return self.source == "TWITCH_EVENTSUB";
    }
}

impl TryFrom<String> for QuirkMessage {
    type Error = DoxMeDaddyError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let this: Self = match serde_json::from_str(&s) {
            Ok(v) => v,
            Err(e) => return Err(e.into()),
        };

        return Ok(this);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TwitchMessage {
    source: String,
    text: String,
    name: String,
    sub: bool,
}

impl From<PrivmsgMessage> for TwitchMessage {
    fn from(s: PrivmsgMessage) -> Self {
        return TwitchMessage {
            source: "TWITCH_CHAT".to_string(),
            text: s.message_text,
            name: s.sender.name,
            sub: s.badges.iter().any(|b| b.name.to_lowercase().contains("subscriber")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ForwarderEvent {
    TwitchMessageRaw(ServerMessage),
    TwitchMessage(TwitchMessage),
    QuirkMessageRaw(String),
    QuirkMessage(QuirkMessage),
    WebsocketMessage(Message),
    Message(String),
}

impl TryFrom<ForwarderEvent> for Message {
    type Error = DoxMeDaddyError;

    fn try_from(event: ForwarderEvent) -> Result<Self, Self::Error> {
        match event {
            ForwarderEvent::WebsocketMessage(m) => return Ok(m),
            ForwarderEvent::TwitchMessage(m) => {
                return Ok(Message::Text(serde_json::to_string(&m)?));
            }
            ForwarderEvent::QuirkMessage(m) => {
                return Ok(Message::Text(serde_json::to_string(&m)?));
            },
            _ => {
                return Ok(Message::Text("Bad Serialization".to_string()));
            }
        }
    }
}

impl From<&str> for ForwarderEvent {
    fn from(s: &str) -> Self {
        return ForwarderEvent::Message(s.to_string());
    }
}

impl ForwarderEvent {
    pub fn from_str(s: &str) -> Self {
        return ForwarderEvent::Message(s.to_string());
    }
    pub fn from_string(s: String) -> Self {
        return ForwarderEvent::Message(s);
    }
    pub fn make_message(s: String) -> Self {
        return ForwarderEvent::WebsocketMessage(Message::from(s));
    }
}

pub trait Forwarder {
    fn push(&self, event: ForwarderEvent) -> Result<(), DoxMeDaddyError>;
}

pub trait ReceiverGiver {
    fn take_receiver(&mut self) -> Option<TokioUReceiver>;

    // TODO: Do we even need to do this?
    fn give_receiver(&mut self, rx: Option<TokioUReceiver>);
}

pub trait ReceiverTaker {
    fn take<T: ReceiverGiver>(&mut self, giver: &mut T) -> Result<(), DoxMeDaddyError>;
}

pub async fn connect(
    giver: Option<TokioUReceiver>,
    forwarder: TokioUSender,
) -> Result<(), DoxMeDaddyError> {

    if let Some(mut rx) = giver {
        while let Some(message) = rx.recv().await {
            forwarder.send(message);
        }
    }

    return Ok(());
}

mod forwarder_macros {

    #[macro_export]
    macro_rules! simple_forwarder {
        ($id:ident) => {
            impl Forwarder for $id {
                // TODO: how to expand id into the string
                // TODO: stringify! ??
                fn push(&self, event: ForwarderEvent) -> Result<(), DoxMeDaddyError> {
                    self.tx.send(event).expect("{$id}#tx should never fail");
                    return Ok(());
                }
            }
        };
    }

    #[macro_export]
    macro_rules! simple_receiver_giver {
        ($id:ident) => {
            impl ReceiverGiver for $id {
                fn take_receiver(
                    &mut self,
                ) -> Option<tokio::sync::mpsc::UnboundedReceiver<ForwarderEvent>> {
                    let mut rx: Option<tokio::sync::mpsc::UnboundedReceiver<ForwarderEvent>> = None;
                    std::mem::swap(&mut self.rx, &mut rx);
                    return rx;
                }

                fn give_receiver(
                    &mut self,
                    rx: Option<tokio::sync::mpsc::UnboundedReceiver<ForwarderEvent>>,
                ) {
                    if let Some(rx) = rx {
                        self.rx = Some(rx);
                    }
                }
            }
        };
    }
}
