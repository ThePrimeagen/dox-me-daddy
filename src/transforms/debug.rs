use log::info;

use crate::{forwarder::ForwarderEvent, pipeline::PipelineTransform};

pub struct DebugTransform {
    pub pre_message: String,
}

impl PipelineTransform for DebugTransform {
    fn transform(&self, event: Option<ForwarderEvent>) -> Option<ForwarderEvent> {
        info!("DebugTransform#Pipeline({}) {:?}", self.pre_message, event);
        return event;
    }
}
