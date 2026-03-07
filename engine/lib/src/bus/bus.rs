use super::SystemError;
use crate::drivers::led::LedCmd;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal,
};
use protocol::{
    movements::MoveCmd,
    sensors::{Data, I2cSensorCmd},
};

/// Led control bus.
pub static LED_SIGNAL: Signal<CriticalSectionRawMutex, LedCmd> = Signal::new();

/// Movement command signal channel. Latest command is always winning.
pub static MOVE_CMD_SIGNAL: Signal<CriticalSectionRawMutex, MoveCmd> = Signal::new();

/// Error reporting channel for inner system status updates.
pub static ERROR_CH: Channel<CriticalSectionRawMutex, SystemError, 10> = Channel::new();

/// Request sensors channel.
pub static SENSOR_CMD_CH: Channel<CriticalSectionRawMutex, I2cSensorCmd, 10> = Channel::new();

/// Response sensors channel with telemetry data.
pub static TELEMETRY_CH: Channel<CriticalSectionRawMutex, Data, 10> = Channel::new();
