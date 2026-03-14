//! Wheel XD-37GB520 Encoder Driver.

use embassy_stm32::{
    peripherals::{TIM1, TIM2},
    timer::qei::Qei,
};
use embassy_time::{Duration, Instant, Timer};
use protocol::sensors::Data;

use crate::bus::bus::outbound;

const POLL_INTERVAL_MS: u64 = 100;

#[derive(defmt::Format)]
pub struct WheelEncoderConfig {
    /// Number of pulses per wheel revolution.
    pub pulses_per_revolution: f32,
    /// Wheel diameter in millimeters.
    pub wheel_diameter_mm: f32,
}

/// Wheel encoder driver.
pub struct Encoder {
    /// Left side encoder.
    left: Qei<'static, TIM1>,
    /// Right side encoder.
    right: Qei<'static, TIM2>,
    config: WheelEncoderConfig,
}

impl Encoder {
    pub fn new(
        left: Qei<'static, TIM1>,
        right: Qei<'static, TIM2>,
        config: WheelEncoderConfig,
    ) -> Self {
        Encoder {
            left,
            right,
            config,
        }
    }

    /// Raw 16-bit hardware counter for left encoder.
    pub fn left_count(&self) -> u16 {
        self.left.count()
    }

    /// Raw 16-bit hardware counter for right encoder.
    pub fn right_count(&self) -> u16 {
        self.right.count()
    }
}

/// Reads wheel encoders at a fixed interval, computes odometry and speed,
/// sends [`Data::Encoder`] to [`outbound::TELEMETRY`].
#[embassy_executor::task]
pub async fn encoder_handler(encoder: Encoder) {
    let mut odom = Odometry::new(&encoder.config, encoder.left_count(), encoder.right_count());
    let mut prev_time = Instant::now();

    loop {
        Timer::after(Duration::from_millis(POLL_INTERVAL_MS)).await;

        let now = Instant::now();
        let dt_s = (now - prev_time).as_micros() as f32 / 1_000_000.0;

        let data = odom.update(encoder.left_count(), encoder.right_count(), dt_s);
        outbound::TELEMETRY.send(data).await;

        prev_time = now;
    }
}

/// Pure odometry computation from raw encoder counts.
pub struct Odometry {
    mm_per_pulse: f32,
    total_left_mm: f32,
    total_right_mm: f32,
    prev_left: u16,
    prev_right: u16,
}

impl Odometry {
    pub fn new(config: &WheelEncoderConfig, initial_left: u16, initial_right: u16) -> Self {
        Self {
            mm_per_pulse: core::f32::consts::PI * config.wheel_diameter_mm
                / config.pulses_per_revolution,
            total_left_mm: 0.0,
            total_right_mm: 0.0,
            prev_left: initial_left,
            prev_right: initial_right,
        }
    }

    /// Update with new raw encoder counts and elapsed time.
    /// Returns [`Data::Encoder`] with cumulative distance and instantaneous speed.
    pub fn update(&mut self, left: u16, right: u16, dt_s: f32) -> Data {
        let delta_left = (left as i16).wrapping_sub(self.prev_left as i16);
        let delta_right = (right as i16).wrapping_sub(self.prev_right as i16);

        let left_delta_mm = delta_left as f32 * self.mm_per_pulse;
        let right_delta_mm = delta_right as f32 * self.mm_per_pulse;

        self.total_left_mm += left_delta_mm;
        self.total_right_mm += right_delta_mm;

        self.prev_left = left;
        self.prev_right = right;

        Data::Encoder {
            left_mm: self.total_left_mm,
            right_mm: self.total_right_mm,
            left_speed: left_delta_mm / dt_s,
            right_speed: right_delta_mm / dt_s,
        }
    }
}
