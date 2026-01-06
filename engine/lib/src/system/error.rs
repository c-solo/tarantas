//! Error handling module for the firmware.

use crate::{
    bus::{bus::ERROR_CH, SystemError},
    system::reflex::emergency_stop,
};
use defmt::error;
use protocol::sensors::Sensor;

/// Handles all inner system errors.
/// Some errors trigger an emergency stop, while others are logged and sent via network to main system.
#[embassy_executor::task]
pub async fn error_handler() {
    loop {
        match ERROR_CH.receive().await {
            err @ SystemError::SensorError(Sensor::Cliff) => {
                emergency_stop(err);
            }
            err @ SystemError::SensorError(Sensor::Distance)
            | err @ SystemError::SensorError(Sensor::Imu) => {
                // no need to stop the system for distance or Imu errors
                error!("{}", err);
                // TODO: send notification to main system status
            }
        }
    }
}
