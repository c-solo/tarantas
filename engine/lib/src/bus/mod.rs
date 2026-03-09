//! Inner bus modules for the STM32 firmware.

use defmt::Format;
use protocol::sensors::I2cSensor;

pub mod bus;

pub use bus::{ERROR_CH, LED_SIGNAL, MOVE_CMD_SIGNAL, SENSOR_CMD_CH, TELEMETRY_CH};

#[derive(Format)]
pub enum SystemError {
    /// I2C bus error, or sensor not responding
    SensorError(I2cSensor),
}
