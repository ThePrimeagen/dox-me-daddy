
use futures_util::{StreamExt};

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use futures_channel::mpsc::unbounded;

use crate::{async_receiver_giver};

use crate::forwarder::{ReceiverTakerAsync, ReceiverGiverAsync};
use crate::{
    forwarder::{ForwarderEvent, ReceiverGiver, ReceiverTaker}, simple_receiver_giver,
};
use log::{info, error};

type FutureReceiver = futures_channel::mpsc::UnboundedReceiver<ForwarderEvent>;
type FutureSender = futures_channel::mpsc::UnboundedSender<ForwarderEvent>;
pub struct FanInAsync {
    pub tx: FutureSender,
    rx: Option<FutureReceiver>,
}

async_receiver_giver!(FanInAsync);

impl FanInAsync {
    pub fn new() -> FanInAsync {
        let (tx, rx) = unbounded();
        return FanInAsync { rx: Some(rx), tx };
    }

    pub fn take_raw(&mut self, rx: FutureReceiver) {
        tokio::spawn(handle_receiver_async(rx, self.tx.clone()));
    }
}

async fn handle_receiver_async(mut rx: FutureReceiver, tx: FutureSender) -> Result<(), crate::error::DoxMeDaddyError> {
    while let Some(message) = rx.next().await {
        info!("handle_receiver_async rx.map {:?}", message);
        match tx.unbounded_send(message) {
            Err(e) => {
                error!("handle_receiver_async unbounded_send failed: {:?}", e);
                break;
            },
            _ => {}
        }
    }

    return Ok(());
}

impl ReceiverTakerAsync for FanInAsync {
    fn take<T: ReceiverGiverAsync>(
        &mut self,
        giver: &mut T,
    ) -> Result<(), crate::error::DoxMeDaddyError> {

        if let Some(rx) = giver.take_receiver() {
            tokio::spawn(handle_receiver_async(rx, self.tx.clone()));
        }

        return Ok(());
    }
}

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

