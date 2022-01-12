use log::info;
use tokio_tungstenite::tungstenite::Message;


use crate::{
    error::DoxMeDaddyError,
    forwarder::{Forwarder, ForwarderEvent},
};

#[derive(Debug)]
pub struct Socket {
    pub is_prime: bool,
    pub tx: futures_channel::mpsc::UnboundedSender<Message>,
    pub addr: std::net::SocketAddr,
    pub id: usize,
}

impl Eq for Socket {}

impl std::hash::Hash for Socket {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.addr.hash(state);
        self.is_prime.hash(state);
        self.id.hash(state);
    }
}

impl PartialEq<Socket> for Socket {
    fn eq(&self, other: &Socket) -> bool {
        return other.addr == self.addr;
    }
}

impl Forwarder for Socket {
    fn push(&self, event: ForwarderEvent) -> Result<(), DoxMeDaddyError> {
        info!("Socket({})#push event({:?})", self.id, event);
        self.tx.unbounded_send(event.try_into()?)?;
        return Ok(());
    }
}
