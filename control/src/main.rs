use std::time::Duration;

use control::network::serial::SerialConnection;
use eyre::Result;
use futures::{SinkExt, StreamExt};
use protocol::{
    Command, EngineEvent, Report,
    movements::MoveCmd,
    sensors::Data,
};
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("tarantas control starting");

    loop {
        info!("connecting to engine...");
        match SerialConnection::new("/dev/ttyTHS1", 115200) {
            Ok(conn) => {
                info!("connected");
                match run(conn).await {
                    Ok(Shutdown::Graceful) => {
                        info!("shutting down");
                        return Ok(());
                    },
                    Ok(Shutdown::Reconnect) => {
                        warn!("connection lost, reconnecting ...");
                    }
                    Err(e) => {
                        error!("error: {e:#}, reconnecting ...");
                    }
                }
            }
            Err(e) => {
                error!("failed to connect: {e:#}");
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

enum Shutdown {
    Graceful,
    Reconnect,
}

async fn run(conn: SerialConnection) -> Result<Shutdown> {
    let mut stream = conn.stream;
    let mut sink = conn.sink;

    loop {
        tokio::select! {
            report = stream.next() => {
                match report {
                    Some(Ok(report)) => handle_report(&report),
                    Some(Err(e)) => {
                        error!("decode error: {e}");
                        return Ok(Shutdown::Reconnect);
                    }
                    None => return Ok(Shutdown::Reconnect),
                }
            }
            _ = tokio::signal::ctrl_c() => {
                info!("ctrl+c, stopping motors");
                let _ = sink.send(Command::Move(MoveCmd::stop())).await;
                return Ok(Shutdown::Graceful);
            }
        }
    }
}

fn handle_report(report: &Report) {
    match report {
        Report::Telemetry(data) => match data {
            Data::Encoder { left_mm, right_mm, left_speed, right_speed } => {
                info!(left_mm, right_mm, left_speed, right_speed, "encoder");
            }
            Data::DistanceFront { mm } => info!(mm, "distance front"),
            Data::DistanceBack { mm } => info!(mm, "distance back"),
        },
        Report::Event(event) => match event {
            EngineEvent::Ready => info!("engine ready"),
            EngineEvent::EmergencyStop => warn!("emergency stop"),
            EngineEvent::LowBattery => warn!("low battery"),
            EngineEvent::Unavailable(sensor) => warn!(?sensor, "sensor unavailable"),
        },
        Report::Pong(nonce) => info!(nonce, "pong"),
        Report::Error(err) => error!(?err, "engine error"),
    }
}
