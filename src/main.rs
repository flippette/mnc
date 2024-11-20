#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

mod display;
mod light_sensor;
mod moisture_sensor;

use defmt::{debug, info};
use defmt_rtt as _;
use embassy_executor::{main, Spawner};
use embassy_rp::{
    adc::{self, Adc},
    bind_interrupts,
    gpio::{Level, Output, Pull},
    i2c::{self, I2c},
    peripherals::I2C0,
    spi::Spi,
};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};
use embassy_time::Duration;
use panic_probe as _;
use static_cell::ConstStaticCell;

const COOLDOWN: Duration = Duration::from_secs(1);

#[main]
async fn main(s: Spawner) {
    let p = embassy_rp::init(<_>::default());
    info!("peripherals init");

    static LIGHT_SENSOR_SIGNAL: ConstStaticCell<Signal<NoopRawMutex, f32>> =
        ConstStaticCell::new(Signal::new());
    static MOISTURE_SENSOR_SIGNAL: ConstStaticCell<Signal<NoopRawMutex, u16>> =
        ConstStaticCell::new(Signal::new());

    let light_sig = &*LIGHT_SENSOR_SIGNAL.take();
    let moisture_sig = &*MOISTURE_SENSOR_SIGNAL.take();

    s.must_spawn(light_sensor::driver(
        I2c::new_async(p.I2C0, p.PIN_17, p.PIN_16, Irqs, <_>::default()),
        light_sig,
    ));
    debug!("light sensor driver spawned");

    s.must_spawn(moisture_sensor::driver(
        Adc::new(p.ADC, Irqs, <_>::default()),
        adc::Channel::new_pin(p.PIN_26, Pull::Down),
        moisture_sig,
    ));
    debug!("moisture sensor driver spawned");

    s.must_spawn(display::driver(
        Spi::new_blocking_txonly(p.SPI1, p.PIN_14, p.PIN_15, <_>::default()),
        Output::new(p.PIN_13, Level::Low),
        Output::new(p.PIN_11, Level::Low),
        Output::new(p.PIN_10, Level::Low),
        light_sig,
        moisture_sig,
    ));
    debug!("display driver spawned");
}

#[defmt::panic_handler]
fn defmt_panic() -> ! {
    cortex_m::asm::udf()
}

bind_interrupts! {
    struct Irqs {
        I2C0_IRQ => i2c::InterruptHandler<I2C0>;
        ADC_IRQ_FIFO => adc::InterruptHandler;
    }
}
