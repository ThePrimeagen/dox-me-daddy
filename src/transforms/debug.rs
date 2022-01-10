use log::debug;

use crate::{forwarder::ForwarderEvent, pipeline::PipelineTransform};

pub struct DebugTransform;
impl PipelineTransform for DebugTransform {
    fn transform(&self, event: Option<ForwarderEvent>) -> Option<ForwarderEvent> {
        debug!("DebugTransform#Pipeline {:?}", event);
        return event;
    }
}
