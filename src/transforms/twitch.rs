use twitch_irc::message::ServerMessage;

use crate::{forwarder::ForwarderEvent, pipeline::PipelineTransform};

pub struct TwitchTransform;
impl PipelineTransform for TwitchTransform {
    fn transform(&self, event: Option<ForwarderEvent>) -> Option<ForwarderEvent> {
        match event {
            Some(ForwarderEvent::TwitchMessageRaw(ServerMessage::Privmsg(m))) => {
                let e = Some(ForwarderEvent::TwitchMessage(m.into()));
                return e;
            }
            _ => {
                return event;
            }
        }
    }
}

