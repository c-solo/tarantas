use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind};
use futures::StreamExt;
use protocol::{Command, movements::MoveCmd};
use tokio::sync::mpsc;
use tracing::info;

const SPEED: f32 = 0.5;
const ACCEL_SECS: f32 = 0.5;
const HEARTBEAT_MS: u64 = 200;

pub async fn stdin_task(tx: mpsc::Sender<Command>) {
    info!("controls: W/S forward/back, A/D turn, Space stop, Q quit");

    let mut reader = EventStream::new();
    let mut last_cmd: Option<Command> = None;
    let mut heartbeat = tokio::time::interval(Duration::from_millis(HEARTBEAT_MS));

    loop {
        tokio::select! {
            event = reader.next() => {
                let Some(Ok(Event::Key(KeyEvent {
                    code,
                    kind: KeyEventKind::Press,
                    ..
                }))) = event else {
                    continue;
                };

                let cmd = match code {
                    KeyCode::Char('w') => MoveCmd::drive(SPEED, SPEED, ACCEL_SECS),
                    KeyCode::Char('s') => MoveCmd::drive(-SPEED, -SPEED, ACCEL_SECS),
                    KeyCode::Char('a') => MoveCmd::drive(-SPEED, SPEED, ACCEL_SECS),
                    KeyCode::Char('d') => MoveCmd::drive(SPEED, -SPEED, ACCEL_SECS),
                    KeyCode::Char(' ') => MoveCmd::stop(),
                    KeyCode::Char('q') => {
                        info!("quit requested");
                        return;
                    }
                    _ => continue,
                };

                let cmd = Command::Move(cmd);
                last_cmd = Some(cmd.clone());
                if tx.send(cmd).await.is_err() {
                    return;
                }
            }
            _ = heartbeat.tick() => {
                if let Some(cmd) = &last_cmd
                    && tx.send(cmd.clone()).await.is_err()
                {
                    return;
                }
            }
        }
    }
}
