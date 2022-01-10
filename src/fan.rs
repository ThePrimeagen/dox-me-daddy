use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::error::DoxMeDaddyError;
use crate::{
    forwarder::{Forwarder, ForwarderEvent, ReceiverGiver, ReceiverTaker},
    simple_forwarder, simple_receiver_giver,
};
use log::info;

type TokioUReceiver = UnboundedReceiver<ForwarderEvent>;
type TokioUSender = UnboundedSender<ForwarderEvent>;
pub struct FanIn {
    pub tx: TokioUSender,
    rx: Option<TokioUReceiver>,
}

simple_receiver_giver!(FanIn);

async fn handle_receiver(mut rx: TokioUReceiver, tx: TokioUSender) {
    while let Some(message) = rx.recv().await {
        tx.send(message)
            .expect("FanIn#handle_receiver should never fail.");
    }
}

impl ReceiverTaker for FanIn {
    fn take<T: ReceiverGiver>(
        &mut self,
        giver: &mut T,
    ) -> Result<(), crate::error::DoxMeDaddyError> {
        if let Some(rx) = giver.take_receiver() {
            tokio::spawn(handle_receiver(rx, self.tx.clone()));
        }

        return Ok(());
    }
}

impl FanIn {
    pub fn new() -> FanIn {
        let (tx, rx) = unbounded_channel();
        return FanIn { rx: Some(rx), tx };
    }

    pub fn take_raw(&mut self, rx: TokioUReceiver) {
        tokio::spawn(handle_receiver(rx, self.tx.clone()));
    }
}

type ListOfSenders = Arc<Mutex<Vec<TokioUSender>>>;
pub struct FanOut {
    pub tx: TokioUSender,
    txes: ListOfSenders,
}

simple_forwarder!(FanOut);

async fn handle_fan_out(mut rx: TokioUReceiver, txes: ListOfSenders) {
    // TODO: Does RC help here instead of cloning message?
    while let Some(message) = rx.recv().await {
        txes.lock()
            .expect("txes lock should never fail.")
            .iter()
            .for_each(|tx| {
                tx.send(message.clone())
                    .expect("FanIn#handle_receiver should never fail.");
            });
    }
}

impl FanOut {
    pub fn new() -> FanOut {
        let (tx, rx) = unbounded_channel();
        let txes: ListOfSenders = Arc::new(Mutex::new(vec![]));
        tokio::spawn(handle_fan_out(rx, txes.clone()));
        return FanOut { tx, txes };
    }

    pub fn fan_out_to(&mut self, tx: TokioUSender) {
        self.txes.lock().expect("rxes lock to never fail").push(tx);
    }
}
