
use std::{marker::PhantomData, time::Duration};
use crate::overlay::protocol::OverlayProtocolConfig;

pub struct OverlayHandlerConfig {
    pub protocol_config: OverlayProtocolConfig,
    pub allow_listening: bool,
    pub idle_timeout: Duration,
}

pub struct OverlayHandlerProto<T> {
    config: OverlayHandlerConfig,
    _type: PhantomData<T>
} 

impl<T> OverlayHandlerProto<T> {
    pub fn new(config: OverlayHandlerConfig) -> Self {
        OverlayHandlerProto {
            config,
            _type: PhantomData
        }
    }
}

