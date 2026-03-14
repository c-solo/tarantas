use serde::{Deserialize, Serialize};

/// Command to control the movement of the robot.
/// New commands override previous ones, acceleration smoothly to the new target.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MoveCmd {
    /// Left side speed: -1.0 to 1.0 (Forward/Backward)
    pub left: f32,
    /// Right side speed: -1.0 to 1.0 (Forward/Backward)
    pub right: f32,
    /// How long (in seconds) to reach target speed from standstill.
    ///
    /// - `0.0` — instant, no acceleration
    /// - `0.5` — reach full speed in 0.5s (aggressive)
    /// - `1.0` — reach full speed in 1.0s (smooth)
    /// - `2.0` — reach full speed in 2.0s (very smooth)
    ///
    /// Bigger value = slower, smoother acceleration.
    /// Engine accelerates linearly from current speed to target over this duration.
    pub accel_secs: f32,
}

impl MoveCmd {
    /// Moves the robot with specified left and right speeds.
    /// 1.0 is full forward, -1.0 is full backward.
    pub fn drive(left: f32, right: f32, accel_secs: f32) -> Self {
        Self {
            left: left.clamp(-1.0, 1.0),
            right: right.clamp(-1.0, 1.0),
            accel_secs: accel_secs.max(0.0),
        }
    }

    /// Stops the robot instantly.
    pub fn stop() -> Self {
        Self::drive(0.0, 0.0, 0.0)
    }
}
