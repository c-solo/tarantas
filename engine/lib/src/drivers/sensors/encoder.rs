//! Wheel XD-37GB520 Encoder Driver.

use embassy_stm32::{
    peripherals::{TIM1, TIM2},
    timer::qei::Qei,
};

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
    _config: WheelEncoderConfig,
}

impl Encoder {
    pub fn new(
        left: Qei<'static, TIM1>,
        right: Qei<'static, TIM2>,
        _config: WheelEncoderConfig,
    ) -> Self {
        Encoder {
            left,
            right,
            _config,
        }
    }

    pub fn left_count(&self) -> i32 {
        self.left.count() as i32
    }

    pub fn right_count(&self) -> i32 {
        self.right.count() as i32
    }
}

#[embassy_executor::task]
pub async fn encoder_handler(mut _encoder: Encoder) {
    todo!();
}
