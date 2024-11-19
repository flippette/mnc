#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use bh1750::{BH1750Error, BH1750};
use defmt::{debug, error, info, panic};
use defmt_rtt as _;
use embassy_executor::{main, task, Spawner};
use embassy_rp::{
    adc::{self, Adc},
    bind_interrupts,
    gpio::Pull,
    i2c::{self, I2c},
    peripherals::*,
};
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    watch::{self, Watch},
};
use embassy_time::{Delay, Duration, Timer};
use panic_probe as _;
use static_cell::ConstStaticCell;

const COOLDOWN: Duration = Duration::from_secs(1);

#[main]
async fn main(s: Spawner) {
    let p = embassy_rp::init(<_>::default());
    info!("init");

    static LIGHT_SENSOR_WATCH: ConstStaticCell<Watch<NoopRawMutex, f32, 1>> =
        ConstStaticCell::new(Watch::new());
    s.must_spawn(light_sensor(
        I2c::new_async(p.I2C0, p.PIN_17, p.PIN_16, Irqs, <_>::default()),
        LIGHT_SENSOR_WATCH.take().sender(),
    ));
    debug!("spawned light sensor handler");

    static MOISTURE_SENSOR_WATCH: ConstStaticCell<Watch<NoopRawMutex, u16, 1>> =
        ConstStaticCell::new(Watch::new());
    s.must_spawn(moisture_sensor(
        Adc::new(p.ADC, Irqs, <_>::default()),
        adc::Channel::new_pin(p.PIN_26, Pull::Down),
        MOISTURE_SENSOR_WATCH.take().sender(),
    ));
    debug!("spawned moisture sensor handler");
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

#[task]
async fn light_sensor(
    i2c: I2c<'static, I2C0, i2c::Async>,
    tx: watch::Sender<'static, NoopRawMutex, f32, 1>,
) {
    let mut sensor = BH1750::new(i2c, Delay, false);
    match sensor.start_continuous_measurement(bh1750::Resolution::High) {
        Err(BH1750Error::MeasurementTimeOutOfRange) => panic!(
            "failed to initialize light sensor: measurement time out of range"
        ),
        Err(BH1750Error::I2C(e)) => {
            panic!("failed to initialize light sensor: i2c error: {}", e)
        }
        Ok(_) => {}
    }

    debug!("initialized light sensor");

    loop {
        match sensor.get_current_measurement(bh1750::Resolution::High) {
            Ok(v) => {
                tx.send(v);
                debug!("read from light sensor: {}", v);
            }
            Err(BH1750Error::MeasurementTimeOutOfRange) => error!(
                "error reading light sensor: measurement time out of range",
            ),
            Err(BH1750Error::I2C(e)) => {
                error!("error reading light sensor: i2c error: {}", e)
            }
        }

        Timer::after(COOLDOWN).await;
    }
}

#[task]
async fn moisture_sensor(
    mut adc: Adc<'static, adc::Async>,
    mut ch: adc::Channel<'static>,
    tx: watch::Sender<'static, NoopRawMutex, u16, 1>,
) {
    loop {
        match adc.read(&mut ch).await {
            Ok(v) => {
                tx.send(v);
                debug!("read from moisture sensor: {}", v);
            }
            Err(e) => error!("error reading moisture sensor: adc error: {}", e),
        }

        Timer::after(COOLDOWN).await;
    }
}
