use log::debug;

use crate::{pipeline::PipelineTransform, forwarder::ForwarderEvent};

pub struct DebugTransform;
impl PipelineTransform for DebugTransform {
    fn transform(&self, event: Option<ForwarderEvent>) -> Option<ForwarderEvent> {
        debug!("DebugTransform#Pipeline {:?}", event);
        return event;
    }
}


