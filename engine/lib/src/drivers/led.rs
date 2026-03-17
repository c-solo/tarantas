use embassy_stm32::gpio::Output;
use embassy_time::with_timeout;

use crate::bus::bus::inbound;

#[allow(dead_code)]
pub struct Led {
    name: &'static str,
    pin: Output<'static>,
}

impl Led {
    /// Creates new LED driver.
    pub fn new(name: &'static str, pin: Output<'static>) -> Self {
        defmt::info!("led '{}' initialized", name);
        Self { name, pin }
    }

    /// Turn LED on (active low: pin LOW = LED on).
    pub fn on(&mut self) {
        self.pin.set_low();
    }

    /// Turn LED off (active low: pin HIGH = LED off).
    pub fn off(&mut self) {
        self.pin.set_high();
    }
}

/// Command to control the LED.
pub enum LedCmd {
    On,
    Off,
    /// Blink for given ms.
    Blink(u64),
}

/// Main operation task for the LED.
/// Listens for commands on the [`inbound::LED`] signal.
#[embassy_executor::task]
pub async fn led_handler(mut led: Led) {
    let mut current_state = LedCmd::Off;

    loop {
        match current_state {
            LedCmd::On => {
                led.on();
                // blocks here in on state until next signal
                current_state = inbound::LED.wait().await;
            }
            LedCmd::Off => {
                led.off();
                // blocks here in off state until next signal
                current_state = inbound::LED.wait().await;
            }
            LedCmd::Blink(delay_ms) => {
                let duration = embassy_time::Duration::from_millis(delay_ms);

                led.on();
                if let Ok(new_cmd) = with_timeout(duration, inbound::LED.wait()).await {
                    current_state = new_cmd;
                    continue;
                };

                led.off();
                if let Ok(new_cmd) = with_timeout(duration, inbound::LED.wait()).await {
                    current_state = new_cmd;
                    continue;
                };
            }
        }
    }
}
