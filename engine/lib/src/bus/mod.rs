//! Inner bus modules for the STM32 firmware.

use defmt::Format;
use protocol::Sensor;

pub mod bus;

pub use bus::{ERROR_CH, LED_SIGNAL};

#[derive(Format)]
pub enum SystemError {
    /// I2C bus error, or sensor not responding
    SensorError(Sensor),
}
