use serde::{Deserialize, Serialize};

/// Command to control the movement of the robot.
/// Note that new commands override previous ones instantly.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MoveCmd {
    /// Left side speed: -1.0 to 1.0 (Forward/Backward)
    pub left: f32,
    /// Right side speed: -1.0 to 1.0 (Forward/Backward)
    pub right: f32,
}

impl MoveCmd {
    /// Moves the robot with specified left and right speeds.
    /// 1.0 is full forward, -1.0 is full backward.
    pub fn drive(left: f32, right: f32) -> Self {
        Self {
            left: left.clamp(-1.0, 1.0),
            right: right.clamp(-1.0, 1.0),
        }
    }

    /// Stops the robot.
    pub fn stop() -> Self {
        Self::drive(0.0, 0.0)
    }
}

// todo: consider adding more advanced movements like turn 90 degrees, turn around, etc.
