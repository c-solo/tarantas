//! Inner bus modules for the STM32 firmware.
//!
//! Channels are split by data flow direction:
//! - [`bus::inbound`] — Control (Jetson) → Engine (STM32)
//! - [`bus::outbound`] — Engine (STM32) → Control (Jetson)
//! - [`bus::internal`] — Engine (STM32) → Engine task

use defmt::Format;
use protocol::sensors::I2cSensor;

pub mod bus;

#[derive(Format)]
pub enum SystemError {
    /// I2C bus error, or sensor not responding
    SensorError(I2cSensor),
}
