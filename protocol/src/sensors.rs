use serde::{Deserialize, Serialize};

/// Subscribe commands for various sensors.
/// After subscription, sensors will start sending [`Data`] data at specified intervals.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SensorCmd {
    SubscribeTo {
        sensor: Sensor,
        poll_interval_ms: u32,
    },
}

/// Telemetry data from various sensors.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Data {
    DistanceFront { mm: u16 },
    DistanceBack { mm: u16 },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Sensor {
    /// Mesures distance, detects obstacle.
    Distance,
    /// Detects no ground under the robot (cliffs, stairs).
    Cliff,
    /// Inertial Measurement Unit, measures acceleration and rotation.
    Imu,
}
