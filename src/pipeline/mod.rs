use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;

use crate::error::DoxMeDaddyError;
use crate::fan::FanIn;
use crate::forwarder::{Forwarder, ForwarderEvent, ReceiverGiver};
use crate::opts::ServerOpts;
use crate::transforms::debug::DebugTransform;
use crate::{simple_forwarder, simple_receiver_giver};
use log::info;

type TokioUReceiver = UnboundedReceiver<ForwarderEvent>;
type TokioUSender = UnboundedSender<ForwarderEvent>;
type Transforms = Arc<Mutex<Vec<Box<dyn PipelineTransform + Send>>>>;

pub struct Pipeline {
    pub join_handle: JoinHandle<()>,

    tx: TokioUSender,
    rx: Option<TokioUReceiver>,
    transforms: Transforms,
}

simple_receiver_giver!(Pipeline);
simple_forwarder!(Pipeline);

pub trait PipelineTransform {
    fn transform(&self, event: Option<ForwarderEvent>) -> Option<ForwarderEvent>;
}

async fn handle_pipeline(mut rx: TokioUReceiver, transforms: Transforms, tx: TokioUSender) {
    while let Some(message) = rx.recv().await {
        let message = transforms
            .lock()
            .expect("Pipeline#add_transforms lock should never fail")
            .iter()
            .fold(Some(message), |m, t| t.transform(m));

        if let Some(message) = message {
            tx.send(message)
                .expect("handle_pipeline#send should never fail");
        }
    }
}

impl Pipeline {
    pub fn new(mut fan_in: FanIn, opts: &ServerOpts) -> Pipeline {
        let (tx, rx) = unbounded_channel();
        let mut transforms: Vec<Box<dyn PipelineTransform + Send>> = vec![];
        if opts.debug {
            transforms.push(Box::new(DebugTransform {}));
        }
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
