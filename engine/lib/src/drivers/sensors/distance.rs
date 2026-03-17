//! Distance sensor module (VL53L0X) for measuring distance to objects and obstacles.

use defmt::info;
use embassy_stm32::{
    gpio::Output,
    i2c::{I2c, Master},
    mode::Blocking,
};
use embassy_time::{Duration, Instant, block_for};
use embedded_hal_bus::i2c::RefCellDevice;
use vl53l0x::{Error, VL53L0x};

use crate::drivers::sensors::SensorError;
use protocol::sensors::Distance;

/// VL53L0X reliable range limit in mm.
const MAX_RANGE_MM: u16 = 1200;

pub type SharedI2c = RefCellDevice<'static, I2c<'static, Blocking, Master>>;

/// Distance sensor driver for VL53L0X.
pub struct DistanceSensor {
    sensor: Option<VL53L0x<SharedI2c>>,
    poll_interval: Duration,
    next_poll_at: Instant,
}

impl DistanceSensor {
    /// Creates distance sensor (VL53L0X).
    /// Returns a sensor in disabled state if hardware is not connected.
    /// - `name` sensor name for logging.
    /// - `i2c` interface to communicate with the sensor.
    /// - `shut_pin` pin for shutting down the sensor (for setting new I2C addr).
    /// - `new_addr` I2C address that will be set for the sensor.
    pub fn new(
        name: &'static str,
        i2c: SharedI2c,
        mut shut_pin: Output<'static>,
        new_addr: u8,
    ) -> Self {
        // enable sensor
        shut_pin.set_high();
        // wait a bit for sensor init
        block_for(Duration::from_millis(10));

        // init driver and change default 0x29 address to new_addr
        let sensor = match VL53L0x::new(i2c) {
            Ok(mut s) => match s.set_address(new_addr) {
                Ok(()) => {
                    info!(
                        "distance sensor '{}' initialized at 0x{:02x}",
                        name, new_addr
                    );
                    // for better accuracy at long distances (5 measurements per second is enough for our use case)
                    s.set_measurement_timing_budget(200_000).ok();
                    Some(s)
                }
                Err(_) => {
                    defmt::warn!("distance sensor '{}': failed to set address", name);
                    None
                }
            },
            Err(_) => {
                defmt::warn!("distance sensor '{}': not connected", name);
                None
            }
        };

        DistanceSensor {
            sensor,
            poll_interval: Duration::MAX,
            next_poll_at: Instant::MAX,
        }
    }

    /// Checks if the sensor is ready to be polled at given time.
    pub fn ready(&self, now: Instant) -> bool {
        now >= self.next_poll_at
    }

    /// Sets the polling interval for the sensor.
    pub fn set_poll_interval(&mut self, interval: Duration) {
        self.poll_interval = interval;
        self.next_poll_at = Instant::now();
    }

    /// Reads distance from the sensor.
    /// Returns `Distance::Mm(mm)` for reliable readings, `Distance::Far` when beyond range.
    pub fn read_distance(&mut self) -> Result<Distance, SensorError> {
        match &mut self.sensor {
            Some(s) => {
                let mm = s
                    .read_range_single_millimeters_blocking()
                    .map_err(map_err)?;
                if mm > MAX_RANGE_MM {
                    Ok(Distance::Far)
                } else {
                    Ok(Distance::Mm(mm))
                }
            }
            None => Err(SensorError("sensor not connected")),
        }
    }

    /// Returns true if sensor hardware is present.
    pub fn is_connected(&self) -> bool {
        self.sensor.is_some()
    }

    /// Updates the next poll time and returns it.
    pub fn update_next_poll_at(&mut self, now: Instant) -> Instant {
        self.next_poll_at = now + self.poll_interval;
        self.next_poll_at
    }

    /// Returns the next scheduled poll time.
    pub fn next_poll_at(&self) -> Instant {
        self.next_poll_at
    }
}

fn map_err<E>(err: Error<E>) -> SensorError {
    match err {
        Error::InvalidDevice(_) | Error::InvalidAddress(_) => {
            SensorError("invalid device or address")
        }
        Error::BusError(_) | Error::Timeout => SensorError("sensor is unavailable"),
    }
}
