#[derive(Debug)]
pub enum Event {
    Twitch,
    QuirkMessage(String),
}
