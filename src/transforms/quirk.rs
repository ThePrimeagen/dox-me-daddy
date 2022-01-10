use crate::{forwarder::{ForwarderEvent, QuirkMessage}, pipeline::PipelineTransform};

pub struct QuirkTransform;
impl PipelineTransform for QuirkTransform {
    fn transform(&self, event: Option<ForwarderEvent>) -> Option<ForwarderEvent> {
        if let Some(ForwarderEvent::QuirkMessageRaw(string)) = event {
            let mut ret: Option<ForwarderEvent> = None;
            match QuirkMessage::try_from(string) {
                Ok(msg) => {
                    ret = Some(ForwarderEvent::QuirkMessage(msg));
                }
                _ => {}
            }
            return ret;
        }

        return event;
    }
}

pub struct QuirkFilterTransform;
impl PipelineTransform for QuirkFilterTransform {
    fn transform(&self, event: Option<ForwarderEvent>) -> Option<ForwarderEvent> {
        if let Some(ForwarderEvent::QuirkMessage(m)) = event {
            if m.is_twitch_event() {
                return Some(ForwarderEvent::QuirkMessage(m));
            }
            return None;
        }

        return event;
    }
}
