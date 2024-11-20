use bh1750::{BH1750Error, Resolution, BH1750};
use defmt::{debug, error, unreachable};
use embassy_executor::task;
use embassy_rp::{
    i2c::{self, I2c},
    peripherals::I2C0,
};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};
use embassy_time::{Delay, Duration, Timer};

use crate::COOLDOWN;

type Sensor<'i2c> = BH1750<I2c<'i2c, I2C0, i2c::Async>, Delay>;

const UNRESPONSIVE_RETRY_DELAY: Duration = Duration::from_secs(5);

/// The amount of time a high-resolution measurement takes by default.
const HIGH_RES_MT: Duration = Duration::from_millis(120);
/// The default value of the measurement time register.
const MTREG_DEFAULT: u8 = 69; // no really

/// Driver task for a BH1750 light sensor.
#[task]
pub async fn driver(
    i2c: I2c<'static, I2C0, i2c::Async>,
    sig: &'static Signal<NoopRawMutex, f32>,
) {
    let mut sensor = BH1750::new(i2c, Delay, false);

    loop {
        match measure(&mut sensor).await {
            Ok(v) => {
                sig.signal(v);
                debug!("light sensor read -> {} lx", v);
            }
            Err(i2c::Error::Abort(i2c::AbortReason::NoAcknowledge)) => {
                error!("light sensor read failed: i2c device unresponsive");
                Timer::after(UNRESPONSIVE_RETRY_DELAY).await;
                continue;
            }
            Err(e) => {
                error!("light sensor read failed: i2c error: {}", e);
            }
        }

        Timer::after(COOLDOWN).await;
    }
}

/// Perform a continuous, non-blocking, high-resolution measurement on the
/// light sensor.
async fn measure(sensor: &mut Sensor<'static>) -> Result<f32, i2c::Error> {
    async fn measure_inner(
        sensor: &mut Sensor<'static>,
    ) -> Result<f32, BH1750Error<i2c::Error>> {
        sensor.set_measurement_time_register(MTREG_DEFAULT)?;
        sensor.start_continuous_measurement(Resolution::High)?;
        Timer::after(HIGH_RES_MT).await;
        sensor.get_current_measurement(Resolution::High)
    }

    match measure_inner(sensor).await {
        Ok(v) => Ok(v),
        Err(BH1750Error::I2C(e)) => Err(e),
        Err(BH1750Error::MeasurementTimeOutOfRange) => {
            unreachable!("default mtreg out of range???");
        }
    }
}
