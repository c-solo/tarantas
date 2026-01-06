//! Driver for chassis hardware components.

use embassy_stm32::{
    gpio::OutputType,
    peripherals,
    time::Hertz,
    timer::{
        simple_pwm::{PwmPin, SimplePwm},
        GeneralInstance4Channel,
    },
    Peri,
};
use protocol::movements::MOVE_CMD_SIGNAL;

/// Skid-steer chassis with 4 wheels driven by two drivers (`BTS7960`).
/// Each driver controls two motors on one side (left/right).
pub struct SkidSteer {
    /// Left side motor driver.
    pub left: SimplePwm<'static, peripherals::TIM3>,
    /// Right side motor driver.
    pub right: SimplePwm<'static, peripherals::TIM4>,
}

impl SkidSteer {
    /// Creates new skid-steer chassis driver.
    ///
    /// # Arguments
    /// * `left_motor` - Left motor driver
    /// * `right_motor` - Right motor driver
    pub fn new(
        l_timer: Peri<'static, peripherals::TIM3>,
        l_fwd_pin: Peri<'static, peripherals::PA6>,
        l_rev_pin: Peri<'static, peripherals::PA7>,
        r_timer: Peri<'static, peripherals::TIM4>,
        r_fwd_pin: Peri<'static, peripherals::PB6>,
        r_rev_pin: Peri<'static, peripherals::PB7>,
        frequency: Hertz,
    ) -> Self {
        let mut left_pwm = SimplePwm::new(
            l_timer,
            Some(PwmPin::new(l_fwd_pin, OutputType::PushPull)),
            Some(PwmPin::new(l_rev_pin, OutputType::PushPull)),
            None,
            None,
            frequency,
            Default::default(),
        );
        left_pwm.ch1().enable();
        left_pwm.ch2().enable();

        let mut right_pwm = SimplePwm::new(
            r_timer,
            Some(PwmPin::new(r_fwd_pin, OutputType::PushPull)),
            Some(PwmPin::new(r_rev_pin, OutputType::PushPull)),
            None,
            None,
            frequency,
            Default::default(),
        );
        right_pwm.ch1().enable();
        right_pwm.ch2().enable();

        Self {
            left: left_pwm,
            right: right_pwm,
        }
    }

    /// Sets speed for left and right motors.
    /// - `left_speed` - Speed for left motors (-1.0..+1.0).
    /// - `right_speed` - Speed for right motors (-1.0..+1.0).
    pub fn set_speed(&mut self, left_speed: f32, right_speed: f32) {
        let max = self.left.max_duty_cycle() as f32;

        fn go(pwm: &mut SimplePwm<'static, impl GeneralInstance4Channel>, speed: f32, max: f32) {
            let duty = (speed.clamp(-1.0, 1.0).abs() * max) as u32;
            if speed >= 0.0 {
                pwm.ch1().set_duty_cycle(duty);
                pwm.ch2().set_duty_cycle(0);
            } else {
                pwm.ch1().set_duty_cycle(0);
                pwm.ch2().set_duty_cycle(duty);
            }
        }

        go(&mut self.left, left_speed, max);
        go(&mut self.right, right_speed, max);
    }

    /// Stops the chassis (sets speed to 0).
    pub fn stop(&mut self) {
        self.set_speed(0.0, 0.0);
    }
}

/// Main operation task for the chassis.
/// Gets commands from [`MOVE_CMD_SIGNAL`] channel.
#[embassy_executor::task]
pub async fn movement_handler(mut skid_steer: SkidSteer) {
    loop {
        let cmd = MOVE_CMD_SIGNAL.wait().await;
        skid_steer.set_speed(cmd.left, cmd.right);
    }
}
