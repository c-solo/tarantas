//! Communication between Core (STM32 MCU) and Main system (Rpi5).

#![no_std]

// TODO: Design

pub mod movements;
pub mod sensors;

/// Commands sent from Main system (Rpi5) to Engine (STM32 MCU).
#[derive(defmt::Format)]
pub enum EngineCommand {
    Move(movements::MoveCmd),
    Sensor(sensors::SensorCmd),
}

pub enum Error {}

#[derive(defmt::Format)]
pub enum Sensor {
    /// Mesures distance, detects obstacle.
    Distance,
    /// Detects no ground under the robot (cliffs, stairs).
    Cliff,
    /// Inertial Measurement Unit, measures acceleration and rotation.
    Imu,
}

#[derive(defmt::Format)]
pub enum EngineEvent {
    Ready,
    EmergencyStop,
    LowBattery,
    Unavailable(Sensor),
}
