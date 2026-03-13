//! Error handling module for the firmware.

use crate::{
    bus::{bus::internal, SystemError},
    system::reflex::emergency_stop,
};
use defmt::error;
use protocol::sensors::I2cSensor;

/// Handles all inner system errors.
/// Some errors trigger an emergency stop, while others are logged and sent via network to main system.
#[embassy_executor::task]
pub async fn error_handler() {
    loop {
        match internal::ERROR.receive().await {
            err @ SystemError::SensorError(I2cSensor::Cliff) => {
                emergency_stop(err);
            }
            err @ SystemError::SensorError(I2cSensor::Distance)
            | err @ SystemError::SensorError(I2cSensor::Imu) => {
                // no need to stop the system for distance or Imu errors
                error!("{}", err);
                // TODO: send notification to main system status
            }
        }
    }
}
