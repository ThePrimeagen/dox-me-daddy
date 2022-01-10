use tokio_tungstenite::tungstenite::Message;
use twitch_irc::message::ServerMessage;

use crate::error::DoxMeDaddyError;

type TokioUSender = tokio::sync::mpsc::UnboundedSender<ForwarderEvent>;
type TokioUReceiver = tokio::sync::mpsc::UnboundedReceiver<ForwarderEvent>;

#[derive(Debug, Clone)]
pub enum ForwarderEvent {
    TwitchMessage(ServerMessage),
    QuirkMessage(String),
    WebsocketMessage(Message),
    Message(String),
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
            forwarder
                .send(message)
                .expect("connect#forwarder should never fail.");
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
                    info!("{}: forwarder#push {:?}", stringify!($id), event);
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
