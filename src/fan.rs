use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::{forwarder::{ForwarderEvent, ReceiverGiver, ReceiverTaker}, simple_receiver_giver};

type TokioUReceiver = UnboundedReceiver<ForwarderEvent>;
type TokioUSender = UnboundedSender<ForwarderEvent>;
pub struct FanIn {
    pub tx: TokioUSender,
    rx: Option<TokioUReceiver>
}

simple_receiver_giver!(FanIn);

async fn handle_receiver(mut rx: TokioUReceiver, tx: TokioUSender) {
    while let Some(message) = rx.recv().await {
        tx.send(message).expect("FanIn#handle_receiver should never fail.");
    }
}

impl ReceiverTaker for FanIn {
    fn take<T: ReceiverGiver>(&mut self, giver: &mut T) -> Result<(), crate::error::DoxMeDaddyError> {
        if let Some(rx) = giver.take_receiver() {
            tokio::spawn(handle_receiver(rx, self.tx.clone()));
        }

        return Ok(());
    }
}

impl FanIn {
    pub fn new() -> FanIn {
        let (tx, rx) = unbounded_channel();
        FanIn {
            rx: Some(rx),
            tx,
        }
    }

    pub fn take_raw(&mut self, rx: TokioUReceiver) {
        tokio::spawn(handle_receiver(rx, self.tx.clone()));
    }
}

pub struct FanOut {
}

