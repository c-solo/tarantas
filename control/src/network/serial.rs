use crate::network::Connection;
use protocol::{EngineCommand, EngineEvent, Error};

pub struct SerialConnection {
    // TODO: implement
}

impl SerialConnection {
    pub fn new(_port: &str, _baud_rate: u32) -> Self {
        Self {}
    }
}

impl Connection for SerialConnection {
    async fn send_cmd(&self, _cmd: EngineCommand) -> Result<(), Error> {
        todo!()
    }

    async fn receive_event(&self) -> Result<EngineEvent, Error> {
        todo!()
    }
}
