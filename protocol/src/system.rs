use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SystemCmd {
    /// Ping command with nonce to check if the system is responsive.
    /// Engine should respond with the same nonce in a [`crate::Report::Pong`] report.
    Ping(u32),
}
