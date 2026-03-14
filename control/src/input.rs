use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind};
use futures::StreamExt;
use protocol::{Command, movements::MoveCmd};
use tokio::sync::mpsc;
use tracing::info;

const SPEED: f32 = 0.5;
const ACCEL_SECS: f32 = 0.5;

pub async fn stdin_task(tx: mpsc::Sender<Command>) {
    info!("controls: W/S forward/back, A/D turn, Space stop, Q quit");

    let mut reader = EventStream::new();

    while let Some(Ok(event)) = reader.next().await {
        let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = event
        else {
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
                break;
            }
            _ => continue,
        };

        if tx.send(Command::Move(cmd)).await.is_err() {
            break;
        }
    }
}
