use core::cell::RefCell;
use defmt::warn;
use embassy_stm32::{
    i2c::{I2c, Master},
    mode::Blocking,
};
use embassy_time::{with_timeout, Duration, Instant};
use protocol::{
    sensors::{SensorCmd, Telemetry, SENSOR_CMD_CH, TELEMETRY_CH},
    Sensor,
};
use static_cell::StaticCell;

use crate::bus::{SystemError, ERROR_CH};
use distance::DistanceSensor;

pub mod cliff;
pub mod distance;
pub mod encoder;

/// Static storage for the I2C bus using StaticCell + RefCell pattern
pub static SHARED_I2C: StaticCell<RefCell<I2c<'static, Blocking, Master>>> = StaticCell::new();

#[derive(defmt::Format)]
pub struct SensorError(&'static str);

/// Polls all i2c sensors in one task.
/// Handles sensor subscription commands from [`SENSOR_CMD_CH`] and sends telemetry data to [`TELEMETRY_CH`]
#[embassy_executor::task]
pub async fn sensor_polling(mut front_dist: DistanceSensor, mut back_dist: DistanceSensor) {
    loop {
        let now = Instant::now();

        if front_dist.ready(now) {
            match front_dist.read_distance_mm() {
                Ok(mm) => TELEMETRY_CH.send(Telemetry::DistanceFront { mm }).await,
                Err(err) => {
                    warn!("Front distance sensor error: {:?}", err);
                    ERROR_CH
                        .send(SystemError::SensorError(Sensor::Distance))
                        .await
                }
            }
            front_dist.update_next_poll_at(now);
        };

        if back_dist.ready(now) {
            match back_dist.read_distance_mm() {
                Ok(mm) => TELEMETRY_CH.send(Telemetry::DistanceBack { mm }).await,
                Err(e) => {
                    warn!("Back distance sensor error: {:?}", e);
                    ERROR_CH
                        .send(SystemError::SensorError(Sensor::Distance))
                        .await;
                }
            }
            back_dist.update_next_poll_at(now);
        };

        // select the nearest next poll time for awaiting
        let next_poll_at = front_dist.next_poll_at().min(back_dist.next_poll_at());
        let sleep_duration = next_poll_at.duration_since(now);

        // sleep until next poll or command received
        if let Ok(cmd) = with_timeout(sleep_duration, SENSOR_CMD_CH.receive()).await {
            match cmd {
                SensorCmd::SubscribeTo {
                    sensor: Sensor::Distance,
                    poll_interval_ms,
                } => {
                    front_dist.set_poll_interval(Duration::from_millis(poll_interval_ms));
                    back_dist.set_poll_interval(Duration::from_millis(poll_interval_ms));
                }
                SensorCmd::SubscribeTo {
                    sensor: Sensor::Cliff,
                    ..
                } => {
                    todo!("cliff sensor subscription not implemented yet");
                }
                SensorCmd::SubscribeTo {
                    sensor: Sensor::Imu,
                    ..
                } => {
                    todo!("imu sensor subscription not implemented yet");
                }
            }
        };
    }
}
