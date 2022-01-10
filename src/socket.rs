use tokio_tungstenite::tungstenite::Message;

#[derive(Debug)]
pub struct Socket {
    pub is_prime: bool,
    pub tx: futures_channel::mpsc::UnboundedSender<Message>,
    pub addr: std::net::SocketAddr,
    pub id: usize,
}

impl Eq for Socket { }

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



