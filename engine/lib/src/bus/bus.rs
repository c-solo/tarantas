use super::SystemError;
use crate::drivers::led::LedCmd;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal,
};
use protocol::{
    Report,
    movements::MoveCmd,
    sensors::{Data, I2cSensorCmd},
};

/// Inbound channels: Control (Jetson) → Engine tasks.
pub mod inbound {
    use super::*;

    /// LED control signal.
    pub static LED: Signal<CriticalSectionRawMutex, LedCmd> = Signal::new();

    /// Movement command signal. Latest command always wins.
    pub static MOVE_CMD: Signal<CriticalSectionRawMutex, MoveCmd> = Signal::new();

    /// I2C sensor subscription commands.
    pub static SENSOR_CMD: Channel<CriticalSectionRawMutex, I2cSensorCmd, 10> = Channel::new();
}

/// Outbound channels: Engine tasks → Control (Jetson) via network_tx.
pub mod outbound {
    use super::*;

    /// Outgoing reports (pong, errors, etc.) — high priority.
    pub static REPORT: Channel<CriticalSectionRawMutex, Report, 10> = Channel::new();

    /// Sensor/encoder telemetry data — bulk.
    pub static TELEMETRY: Channel<CriticalSectionRawMutex, Data, 10> = Channel::new();
}

/// Internal channels: Engine task → Engine task.
pub mod internal {
    use super::*;

    /// Error reporting from any task to error_handler.
    pub static ERROR: Channel<CriticalSectionRawMutex, SystemError, 10> = Channel::new();
}
