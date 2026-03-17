#![no_std]
#![no_main]

use engine::{
    drivers::{led, sensors, sensors::distance::DistanceSensor},
    system::{error, network},
};

use core::cell::RefCell;
use embassy_executor::Spawner;
use embassy_stm32::{
    Config, bind_interrupts,
    gpio::{Level, Output, Speed},
    i2c::{self},
    peripherals,
    time::Hertz,
    usart,
};

use embedded_hal_bus::i2c::RefCellDevice;
use engine::{bus::bus::inbound, drivers::chassis::SkidSteer};
use protocol::sensors::{I2cSensor, I2cSensorCmd};

use defmt as _;
use defmt_rtt as _;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USART2 => usart::InterruptHandler<peripherals::USART2>;
});

use embassy_stm32::timer::qei::Qei;
use engine::drivers::{
    chassis,
    sensors::{
        encoder,
        encoder::{Encoder, WheelEncoderConfig},
    },
};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Config::default());

    // Enable debug during sleep so RTT works with WFI (embassy low power).
    // DMA1 clock keeps SRAM accessible to debugger while CPU sleeps.
    embassy_stm32::pac::DBGMCU.cr().modify(|w| {
        w.set_dbg_sleep(true);
        w.set_dbg_standby(true);
        w.set_dbg_stop(true);
    });
    embassy_stm32::pac::RCC
        .ahb1enr()
        .modify(|w| w.set_dma1en(true));

    defmt::info!("booting tarantas engine");

    // Initialize LED — blink during boot, solid on when ready
    let led_pin = Output::new(p.PC6, Level::High, Speed::Low);
    let led = led::Led::new("status_led", led_pin);
    spawner.spawn(led::led_handler(led)).expect("spawn led");
    inbound::LED.signal(led::LedCmd::Blink(150));
    spawner.spawn(error::error_handler()).expect("spawn error");

    // Initialize UART (serial link to Jetson)
    let mut usart_config = usart::Config::default();
    usart_config.baudrate = 115200;
    let uart = usart::Uart::new(
        p.USART2,
        p.PA3,
        p.PA2,
        Irqs,
        p.DMA1_CH6,
        p.DMA1_CH5,
        usart_config,
    )
    .expect("USART2 init failed");
    let (tx, rx) = uart.split();
    spawner
        .spawn(network::network_rx(rx))
        .expect("spawn network_rx");
    spawner
        .spawn(network::network_tx(tx))
        .expect("spawn network_tx");
    defmt::info!("uart started");

    // Initialize motors
    let skid_steer = SkidSteer::new(p.TIM3, p.PA6, p.PA7, p.TIM4, p.PB6, p.PB7, Hertz::khz(20));
    spawner
        .spawn(chassis::movement_handler(skid_steer))
        .expect("spawn movement");
    defmt::info!("motors started");

    // Initialize wheel encoders
    let left = Qei::new(p.TIM1, p.PA8, p.PA9, Default::default());
    let right = Qei::new(p.TIM2, p.PA0, p.PA1, Default::default());
    let config = WheelEncoderConfig {
        pulses_per_revolution: 1500.0,
        wheel_diameter_mm: 80.0,
    };
    let wheel_encoders = Encoder::new(left, right, config);
    spawner
        .spawn(encoder::encoder_handler(wheel_encoders))
        .expect("spawn encoder");
    defmt::info!("encoders started");

    // Initialize I2C sensors
    let mut i2c_cfg = i2c::Config::default();
    i2c_cfg.frequency = Hertz::khz(400);
    let i2c = i2c::I2c::new_blocking(p.I2C1, p.PA15, p.PB9, i2c_cfg);
    let shared_i2c = sensors::SHARED_I2C.init(RefCell::new(i2c));
    defmt::info!("i2c bus initialized");

    let front_dist_sensor = DistanceSensor::new(
        "front_dist",
        RefCellDevice::new(shared_i2c),
        Output::new(p.PB0, Level::Low, Speed::Low),
        0x30,
    );

    let back_dist_sensor = DistanceSensor::new(
        "back_dist",
        RefCellDevice::new(shared_i2c),
        Output::new(p.PB1, Level::Low, Speed::Low),
        0x31,
    );

    spawner
        .spawn(sensors::sensor_polling(front_dist_sensor, back_dist_sensor))
        .expect("spawn sensors");

    // Auto-subscribe distance sensors for testing (1 second interval)
    // TODO REMOVE
    inbound::SENSOR_CMD
        .send(I2cSensorCmd::SubscribeTo {
            sensor: I2cSensor::Distance,
            poll_interval_ms: 1000,
        })
        .await;

    // Boot complete — solid LED
    inbound::LED.signal(led::LedCmd::On);
    defmt::info!("engine ready");
}
