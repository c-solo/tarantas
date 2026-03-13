use core::cell::RefCell;
use defmt::warn;
use embassy_stm32::{
    i2c::{I2c, Master},
    mode::Blocking,
};
use embassy_time::{with_timeout, Duration, Instant};
use protocol::sensors::{Data, I2cSensorCmd};
use static_cell::StaticCell;

use crate::bus::{
    bus::{inbound, internal, outbound},
    SystemError,
};
use distance::DistanceSensor;
use protocol::sensors::I2cSensor;

pub mod cliff;
pub mod distance;
pub mod encoder;

/// Static storage for the I2C bus using StaticCell + RefCell pattern
pub static SHARED_I2C: StaticCell<RefCell<I2c<'static, Blocking, Master>>> = StaticCell::new();

#[derive(defmt::Format)]
pub struct SensorError(&'static str);

/// Polls all i2c sensors in one task.
/// Handles sensor subscription commands from [`inbound::SENSOR_CMD`] and sends telemetry to [`outbound::TELEMETRY`]
#[embassy_executor::task]
pub async fn sensor_polling(mut front_dist: DistanceSensor, mut back_dist: DistanceSensor) {
    loop {
        let now = Instant::now();

        if front_dist.ready(now) {
            match front_dist.read_distance_mm() {
                Ok(mm) => outbound::TELEMETRY.send(Data::DistanceFront { mm }).await,
                Err(err) => {
                    warn!("front distance sensor error: {:?}", err);
                    internal::ERROR
                        .send(SystemError::SensorError(I2cSensor::Distance))
                        .await
                }
            }
            front_dist.update_next_poll_at(now);
        };

        if back_dist.ready(now) {
            match back_dist.read_distance_mm() {
                Ok(mm) => outbound::TELEMETRY.send(Data::DistanceBack { mm }).await,
                Err(e) => {
                    warn!("back distance sensor error: {:?}", e);
                    internal::ERROR
                        .send(SystemError::SensorError(I2cSensor::Distance))
                        .await;
                }
            }
            back_dist.update_next_poll_at(now);
        };

        // sleep until next poll or command received
        let next_poll_at = front_dist.next_poll_at().min(back_dist.next_poll_at());
        let cmd = if next_poll_at == Instant::MAX {
            // no active subscriptions — block until first command
            Ok(inbound::SENSOR_CMD.receive().await)
        } else {
            let sleep_duration = next_poll_at.duration_since(now);
            with_timeout(sleep_duration, inbound::SENSOR_CMD.receive()).await
        };

        if let Ok(cmd) = cmd {
            match cmd {
                I2cSensorCmd::SubscribeTo {
                    sensor: I2cSensor::Distance,
                    poll_interval_ms,
                } => {
                    front_dist.set_poll_interval(Duration::from_millis(poll_interval_ms as u64));
                    back_dist.set_poll_interval(Duration::from_millis(poll_interval_ms as u64));
                }
                I2cSensorCmd::SubscribeTo {
                    sensor: I2cSensor::Cliff,
                    ..
                } => {
                    todo!("cliff sensor subscription not implemented yet");
                }
                I2cSensorCmd::SubscribeTo {
                    sensor: I2cSensor::Imu,
                    ..
                } => {
                    todo!("imu sensor subscription not implemented yet");
                }
            }
        };
    }
}
