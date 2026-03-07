//! Only emergency stop related control logic (aka reflexes).
//! Planning and higher level control should be handled by Jetson via UDP.

use crate::{
    bus::{bus::MOVE_CMD_SIGNAL, SystemError, LED_SIGNAL},
    drivers::led::LedCmd,
};
use defmt::error;
use protocol::movements::MoveCmd;

/// Trigger an emergency stop due to the given cause.
pub fn emergency_stop(cause: SystemError) {
    error!("Emergency stop triggered ({})!", cause);
    MOVE_CMD_SIGNAL.signal(MoveCmd::stop());
    LED_SIGNAL.signal(LedCmd::Blink(10));
    // TODO: send emergency stop command to motor controller and report to Jetson
}
