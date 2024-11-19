#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

mod light_sensor;
mod moisture_sensor;

use defmt::{debug, info};
use defmt_rtt as _;
use embassy_executor::{main, Spawner};
use embassy_rp::{
    adc::{self, Adc},
    bind_interrupts,
    gpio::Pull,
    i2c::{self, I2c},
    peripherals::I2C0,
};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, watch::Watch};
use embassy_time::Duration;
use panic_probe as _;
use static_cell::ConstStaticCell;

const COOLDOWN: Duration = Duration::from_secs(1);

#[main]
async fn main(s: Spawner) {
    let p = embassy_rp::init(<_>::default());
    info!("peripherals init");

    static LIGHT_SENSOR_WATCH: ConstStaticCell<Watch<NoopRawMutex, f32, 1>> =
        ConstStaticCell::new(Watch::new());
    s.must_spawn(light_sensor::driver(
        I2c::new_async(p.I2C0, p.PIN_17, p.PIN_16, Irqs, <_>::default()),
        LIGHT_SENSOR_WATCH.take().sender(),
    ));
    debug!("light sensor driver spawned");

    static MOISTURE_SENSOR_WATCH: ConstStaticCell<Watch<NoopRawMutex, u16, 1>> =
        ConstStaticCell::new(Watch::new());
    s.must_spawn(moisture_sensor::driver(
        Adc::new(p.ADC, Irqs, <_>::default()),
        adc::Channel::new_pin(p.PIN_26, Pull::Down),
        MOISTURE_SENSOR_WATCH.take().sender(),
    ));
    debug!("moisture sensor driver spawned");
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
