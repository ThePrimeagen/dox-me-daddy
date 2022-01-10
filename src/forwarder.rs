use tokio_tungstenite::tungstenite::Message;

use crate::error::DoxMeDaddyError;

#[derive(Debug, Clone)]
pub enum ForwarderEvent {
    WebsocketMessage(Message),
    Message(String),
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
