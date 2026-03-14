//! Only emergency stop related control logic (aka reflexes).
//! Planning and higher level control should be handled by Jetson via UDP.

use crate::{
    bus::{SystemError, bus::inbound},
    drivers::led::LedCmd,
};
use defmt::error;
use protocol::movements::MoveCmd;

/// Trigger an emergency stop due to the given cause.
pub fn emergency_stop(cause: SystemError) {
    error!("emergency stop triggered ({})!", cause);
    inbound::MOVE_CMD.signal(MoveCmd::stop());
    inbound::LED.signal(LedCmd::Blink(10));
    // TODO: send emergency stop command to motor controller and report to Jetson
}
