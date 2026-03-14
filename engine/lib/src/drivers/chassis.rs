//! Driver for chassis hardware components.

use crate::bus::bus::inbound;
use embassy_futures::select::{Either, select};
use embassy_stm32::{
    Peri,
    gpio::OutputType,
    peripherals,
    time::Hertz,
    timer::{
        GeneralInstance4Channel,
        simple_pwm::{PwmPin, SimplePwm},
    },
};
use embassy_time::{Duration, Ticker};

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
        fn go(pwm: &mut SimplePwm<'static, impl GeneralInstance4Channel>, speed: f32) {
            let max = pwm.max_duty_cycle() as f32;
            let duty = (speed.clamp(-1.0, 1.0).abs() * max) as u32;
            if speed >= 0.0 {
                pwm.ch1().set_duty_cycle(duty);
                pwm.ch2().set_duty_cycle(0);
            } else {
                pwm.ch1().set_duty_cycle(0);
                pwm.ch2().set_duty_cycle(duty);
            }
        }

        go(&mut self.left, left_speed);
        go(&mut self.right, right_speed);
    }

    /// Stops the chassis (sets speed to 0).
    pub fn stop(&mut self) {
        self.set_speed(0.0, 0.0);
    }
}

const ACCEL_TICK_MS: u64 = 20;

/// Moves `current` towards `target` by at most `max_step`.
fn acceleration(current: f32, target: f32, max_step: f32) -> f32 {
    let diff = target - current;
    if diff.abs() <= max_step {
        target
    } else {
        current + max_step.copysign(diff)
    }
}

/// Main operation task for the chassis.
/// Gets commands from [`inbound::MOVE_CMD`] channel, accelerates smoothly.
#[embassy_executor::task]
pub async fn movement_handler(mut skid_steer: SkidSteer) {
    let mut current_left: f32 = 0.0;
    let mut current_right: f32 = 0.0;
    let mut target_left: f32 = 0.0;
    let mut target_right: f32 = 0.0;
    let mut max_step: f32 = 0.0;

    let mut ticker = Ticker::every(Duration::from_millis(ACCEL_TICK_MS));

    loop {
        match select(inbound::MOVE_CMD.wait(), ticker.next()).await {
            Either::First(cmd) => {
                target_left = cmd.left;
                target_right = cmd.right;
                if cmd.accel_secs == 0.0 {
                    // instant
                    current_left = target_left;
                    current_right = target_right;
                    skid_steer.set_speed(current_left, current_right);
                } else {
                    // 1.0 / accel_secs = speed units per second
                    // * tick_s = speed units per tick
                    let tick_s = ACCEL_TICK_MS as f32 / 1000.0;
                    max_step = tick_s / cmd.accel_secs;
                }
            }
            Either::Second(_) => {
                if current_left == target_left && current_right == target_right {
                    continue;
                }
                current_left = acceleration(current_left, target_left, max_step);
                current_right = acceleration(current_right, target_right, max_step);
                skid_steer.set_speed(current_left, current_right);
            }
        }
    }
}
