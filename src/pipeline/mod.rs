use std::sync::{Arc, Mutex};
use futures_channel::mpsc::unbounded;
use futures_util::{StreamExt};


use tokio::task::JoinHandle;

use crate::error::DoxMeDaddyError;
use crate::fan::{FanInAsync};
use crate::forwarder::{Forwarder, ForwarderEvent, ReceiverGiverAsync};
use crate::opts::ServerOpts;

use crate::{async_receiver_giver, async_forwarder};


type Transforms = Arc<Mutex<Vec<Box<dyn PipelineTransform + Send>>>>;

type FutureReceiver = futures_channel::mpsc::UnboundedReceiver<ForwarderEvent>;
type FutureSender = futures_channel::mpsc::UnboundedSender<ForwarderEvent>;
pub struct Pipeline {
    pub join_handle: JoinHandle<()>,

    tx: FutureSender,
    rx: Option<FutureReceiver>,
    transforms: Transforms,
}

async_receiver_giver!(Pipeline);
async_forwarder!(Pipeline);

pub trait PipelineTransform {
    fn transform(&self, event: Option<ForwarderEvent>) -> Option<ForwarderEvent>;
}

async fn handle_pipeline(mut rx: FutureReceiver, transforms: Transforms, tx: FutureSender) {
    while let Some(message) = rx.next().await {
        let message = transforms
            .lock()
            .expect("Pipeline#add_transforms lock should never fail")
            .iter()
            .fold(Some(message), |m, t| t.transform(m));

        if let Some(message) = message {
            tx.unbounded_send(message)
                .expect("handle_pipeline#send should never fail");
        }
    }
}

impl Pipeline {
    pub fn new(mut fan_in: FanInAsync, _opts: &ServerOpts) -> Pipeline {
        let (tx, rx) = unbounded();
        let transforms: Vec<Box<dyn PipelineTransform + Send>> = vec![];
        let transforms = Arc::new(Mutex::new(transforms));

        // Unwrap safe here.
        let join_handle = tokio::spawn(handle_pipeline(
            fan_in.take_receiver().expect("pipeline fan_in must have receiver"),
            transforms.clone(),
            tx.clone(),
        ));

        return Pipeline {
            // TODO: should I fix this?
            tx: fan_in.tx.clone(),
            rx: Some(rx),
            transforms,
            join_handle,
        };
    }

    pub fn add_transformer(&mut self, transformer: Box<dyn PipelineTransform + Send>) {
        self.transforms
            .lock()
            .expect("Pipeline#add_transforms lock should never fail")
            .push(transformer);
    }
}
