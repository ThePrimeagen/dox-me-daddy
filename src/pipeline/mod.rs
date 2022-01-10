use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use std::rc::Rc;

use crate::event::Event;
use crate::forwarder::Forwarder;

pub struct Pipeline {
    pub tx: UnboundedSender<Event>,
    forwarders: Vec<Rc<dyn Forwarder>>,
}

impl Pipeline {
    pub fn new() -> Pipeline {
        let (tx, _rx) = unbounded_channel::<Event>();
        return Pipeline {
            tx,
            forwarders: vec![],
        };
    }

    pub fn add_forwarder(&mut self, forwarder: Rc<dyn Forwarder>) {
        self.forwarders.push(forwarder);
    }
}


