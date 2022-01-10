use log::info;

use crate::{forwarder::ForwarderEvent, pipeline::PipelineTransform};

pub struct DebugTransform;
impl PipelineTransform for DebugTransform {
    fn transform(&self, event: Option<ForwarderEvent>) -> Option<ForwarderEvent> {
        info!("DebugTransform#Pipeline {:?}", event);
        return event;
    }
}
