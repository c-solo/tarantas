//! Serial communication with STM32.

pub mod serial;

use protocol::{Command, EngineEvent, Error};

#[allow(async_fn_in_trait)]
pub trait Connection {
    /// Sends a command to the engine.
    async fn send_cmd(&self, cmd: Command) -> Result<(), Error>;
    /// Receives an event from the engine.
    async fn receive_event(&self) -> Result<EngineEvent, Error>;
}
