//! UART serial bridge between Jetson and internal bus.
//!
//! Two embassy tasks:
//! - [`network_rx`]: reads UART bytes, decodes [`Command`], dispatches to bus channels
//! - [`network_tx`]: reads from [`outbound::TELEMETRY`] and [`outbound::REPORT`], encodes [`Report`], writes to UART

use defmt::{info, warn};
use embassy_futures::select::{Either, select};
use embassy_stm32::{mode::Async, usart::{UartRx, UartTx}};
use protocol::{
    Command, Report,
    codec::{DecodeResult, EngineCodec},
    system::SystemCmd,
};

use crate::bus::bus::{inbound, outbound};

/// Reads bytes from UART, decodes commands via [`EngineCodec`], and dispatches them to bus channels.
#[embassy_executor::task]
pub async fn network_rx(mut rx: UartRx<'static, Async>) {
    info!("network rx started");
    let mut codec = EngineCodec::new();
    let mut buf = [0u8; 256];

    loop {
        match rx.read_until_idle(&mut buf).await {
            Ok(n) => {
                for &byte in &buf[..n] {
                    match codec.decode(byte) {
                        DecodeResult::Complete(cmd) => dispatch(cmd),
                        DecodeResult::DeserError(_) => warn!("decode error"),
                        DecodeResult::Overflow => warn!("frame overflow, discarded"),
                        DecodeResult::Pending => {}
                    }
                }
            }
            Err(e) => warn!("uart rx error: {}", e),
        }
    }
}

fn dispatch(cmd: Command) {
    match cmd {
        Command::Move(cmd) => inbound::MOVE_CMD.signal(cmd),
        Command::Sensor(cmd) => {
            if inbound::SENSOR_CMD.try_send(cmd).is_err() {
                warn!("sensor cmd channel full, dropping");
            }
        }
        Command::System(SystemCmd::Ping(n)) => {
            if outbound::REPORT.try_send(Report::Pong(n)).is_err() {
                warn!("report channel full, dropping pong");
            }
        }
    }
}

/// Reads reports from bus channels, encodes via [`EngineCodec`], and writes to UART.
#[embassy_executor::task]
pub async fn network_tx(mut tx: UartTx<'static, Async>) {
    info!("network tx started");
    let mut codec = EngineCodec::new();

    loop {
        // outbound::REPORT first — control messages (pong, errors) have priority over bulk telemetry
        let report = match select(outbound::REPORT.receive(), outbound::TELEMETRY.receive()).await {
            Either::First(report) => report,
            Either::Second(data) => Report::Telemetry(data),
        };

        match codec.encode(&report) {
            Ok(bytes) => {
                if let Err(e) = tx.write(bytes).await {
                    warn!("uart tx error: {}", e);
                }
            }
            Err(_) => warn!("encode error"),
        }
    }
}
