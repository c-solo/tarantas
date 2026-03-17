use serde::{Deserialize, Serialize};

/// Subscribe commands for various sensors.
/// After subscription, sensors will start sending [`Data`] data at specified intervals.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum I2cSensorCmd {
    SubscribeTo {
        sensor: I2cSensor,
        poll_interval_ms: u32,
    },
}

/// Distance measurement result.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Distance {
    /// Exact distance in millimeters (reliable range).
    Mm(u16),
    /// Beyond sensor range (>1200mm), no obstacle detected.
    Far,
}

/// Telemetry data from various sensors.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Data {
    DistanceFront(Distance),
    DistanceBack(Distance),
    /// Wheel encoder odometry. Sent continuously without subscription,
    /// at a fixed interval configured in engine settings (default 100ms).
    Encoder {
        /// Distance traveled by left wheels, mm.
        left_mm: f32,
        /// Distance traveled by right wheels, mm.
        right_mm: f32,
        /// Left wheels speed, mm/s.
        left_speed: f32,
        /// Right wheels speed, mm/s.
        right_speed: f32,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum I2cSensor {
    /// Measures distance, detects obstacle.
    Distance,
    /// Detects no ground under the robot (cliffs, stairs).
    Cliff,
    /// Inertial Measurement Unit, measures acceleration and rotation.
    Imu,
}
