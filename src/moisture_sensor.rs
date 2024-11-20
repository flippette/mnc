use defmt::{debug, error};
use embassy_executor::task;
use embassy_rp::adc::{self, Adc};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};
use embassy_time::Timer;

use crate::COOLDOWN;

// TODO: these values seem too low for our specific sensor, calibrate or maybe
//       let the user calibrate it themself
const MOISTURE_DRY_MAX: u16 = 300;
const MOISTURE_WET_MAX: u16 = 700;
const MOISTURE_SUB_MAX: u16 = 950;

/// Driver task for a Grove Moisture Sensor.
#[task]
pub async fn driver(
    mut adc: Adc<'static, adc::Async>,
    mut ch: adc::Channel<'static>,
    sig: &'static Signal<NoopRawMutex, u16>,
) {
    loop {
        match adc.read(&mut ch).await {
            Ok(v) => {
                sig.signal(v);
                debug!(
                    "moisture sensor read -> {} ({})",
                    v,
                    match v {
                        ..=MOISTURE_DRY_MAX => "dry soil",
                        ..=MOISTURE_WET_MAX => "wet soil",
                        ..=MOISTURE_SUB_MAX => "submerged",
                        _ => "unknown",
                    }
                );
            }
            Err(e) => error!("moisture sensor read failed: adc error: {}", e),
        }

        Timer::after(COOLDOWN).await;
    }
}
