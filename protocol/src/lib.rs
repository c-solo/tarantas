//! Communication between Core (STM32 MCU) and Main system (Rpi5).

#![cfg_attr(not(feature = "codec"), no_std)]

use serde::{Deserialize, Serialize};

pub mod movements;
pub mod sensors;
pub mod system;

#[cfg(feature = "codec")]
pub mod codec;

/// Commands sent from Main system (Rpi5) to Engine (STM32 MCU).
/// There is no direct response to commands (fire and forget), instead Engine sends [`Report`]
/// messages back instantly or periodically.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Command {
    Move(movements::MoveCmd),
    Sensor(sensors::SensorCmd),
    System(system::SystemCmd),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    // todo finish
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Report {
    /// Periodic updates from various sensors.
    Telemetry(sensors::Data),
    /// Immediate engine events.
    Event(EngineEvent),
    /// Response to ping command with the same nonce.
    Pong(u32),
    /// System error report.
    Error(Error),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EngineEvent {
    Ready,
    EmergencyStop,
    LowBattery,
    Unavailable(sensors::Sensor),
}
