//! Distance sensor module (VL53L0X) for measuring distance to objects and obstacles.

use defmt::info;
use embassy_stm32::{
    gpio::Output,
    i2c::{I2c, Master},
    mode::Blocking,
};
use embassy_time::{block_for, Duration, Instant};
use embedded_hal_bus::i2c::RefCellDevice;
use vl53l0x::{Error, VL53L0x};

use crate::drivers::sensors::SensorError;

pub type SharedI2c = RefCellDevice<'static, I2c<'static, Blocking, Master>>;

/// Distance sensor driver for VL53L0X.
pub struct DistanceSensor {
    sensor: VL53L0x<SharedI2c>,
    poll_interval: Duration,
    next_poll_at: Instant,
}

impl DistanceSensor {
    /// Creates distance sensor (VL53L0X).
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
        let mut sensor = VL53L0x::new(i2c).expect("fail to create VL53L0x");
        sensor
            .set_address(new_addr)
            .expect("fail to set new VL53L0x address");

        info!(
            "Distance sensor '{}' initialized at address 0x{}",
            name, new_addr
        );

        DistanceSensor {
            sensor,
            // poll is disabled until first subscription request
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

    /// Reads distance from the sensor in millimeters.
    /// Schedules next poll based on the poll interval.
    pub fn read_distance_mm(&mut self) -> Result<u16, SensorError> {
        self.sensor
            .read_range_single_millimeters_blocking()
            .map_err(map_err)
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
