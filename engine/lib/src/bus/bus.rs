use super::SystemError;
use crate::drivers::led::LedCmd;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal,
};

/// Led control bus.
pub static LED_SIGNAL: Signal<CriticalSectionRawMutex, LedCmd> = Signal::new();

/// Error reporting channel for inner system status updates.
pub static ERROR_CH: Channel<CriticalSectionRawMutex, SystemError, 10> = Channel::new();
