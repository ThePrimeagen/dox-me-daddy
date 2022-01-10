use tokio_tungstenite::tungstenite::Message;

#[derive(Debug)]
pub enum Event {
    Twitch,
    QuirkMessage(String),
    WebsocketMessage(Message),
}
