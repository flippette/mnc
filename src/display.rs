use core::convert::Infallible;

use defmt::{debug, error, Debug2Format};
use display_interface_spi::SPIInterface;
use embassy_executor::task;
use embassy_futures::join::join;
use embassy_rp::{
    gpio::Output,
    peripherals::SPI1,
    spi::{self, Spi},
};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};
use embassy_time::{Delay, Duration, Timer};
use embedded_graphics::{
    geometry::*, mono_font::MonoTextStyle, pixelcolor::Rgb565, prelude::*,
    primitives::*, text::Text,
};
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_layout::{layout::linear::LinearLayout, prelude::*};
use heapless::String;
use mipidsi::{
    error::Error as DisplayError,
    models::ST7789,
    options::{ColorInversion, Orientation, Rotation},
    Display,
};
use profont::PROFONT_18_POINT;

const DISPLAY_RETRY_DELAY: Duration = Duration::from_secs(1);
const UPDATE_DELAY: Duration = Duration::from_secs(1);

/// Driver task for the ST7789 240x320 TFT LCD display.
#[task]
pub async fn driver(
    spi: Spi<'static, SPI1, spi::Blocking>,
    cs: Output<'static>,
    dc: Output<'static>,
    rst: Output<'static>,
    light_sig: &'static Signal<NoopRawMutex, f32>,
    moisture_sig: &'static Signal<NoopRawMutex, u16>,
) {
    // unwrapping is okay because `CS::Error == Infallible`
    let di =
        SPIInterface::new(ExclusiveDevice::new(spi, cs, Delay).unwrap(), dc);
    let mut display = mipidsi::Builder::new(ST7789, di)
        .display_size(240, 320)
        .orientation(Orientation::new().rotate(Rotation::Deg90))
        .invert_colors(ColorInversion::Inverted)
        .reset_pin(rst)
        .init(&mut Delay)
        .unwrap();

    loop {
        // `try_drive` only returns if there is one. log it, and retry.
        error!(
            "display driver failed: {}",
            Debug2Format(
                &try_drive(&mut display, light_sig, moisture_sig).await
            )
        );

        Timer::after(DISPLAY_RETRY_DELAY).await;
    }
}

/// Main display loop for the ST7789 TFT LCD.
///
/// This function wakes and clears the display to black before displaying any
/// content.
#[allow(clippy::type_complexity)]
async fn try_drive(
    display: &mut Display<
        SPIInterface<
            ExclusiveDevice<
                Spi<'static, SPI1, spi::Blocking>,
                Output<'static>,
                Delay,
            >,
            Output<'static>,
        >,
        ST7789,
        Output<'static>,
    >,
    light_sig: &'static Signal<NoopRawMutex, f32>,
    moisture_sig: &'static Signal<NoopRawMutex, u16>,
) -> Result<Infallible, DisplayError> {
    display.wake(&mut Delay)?;
    display.clear(Rgb565::CSS_BLACK)?;

    let text_style = MonoTextStyle::new(&PROFONT_18_POINT, Rgb565::CSS_WHITE);

    let labels = LinearLayout::vertical(
        Chain::new(Text::new("light: ", Point::zero(), text_style))
            .append(Text::new("moisture: ", Point::zero(), text_style)),
    )
    .with_alignment(horizontal::Right)
    .arrange();

    let mut last_table_bounds = Rectangle::zero();

    loop {
        let (light, moisture) =
            join(light_sig.wait(), moisture_sig.wait()).await;
        debug!(
            "display driver drawing: light = {} lx, moisture = {}",
            light, moisture
        );

        // bh1750 gives reading in u16 range so this is ok
        let light_str = u16_to_string(light as u16);
        let moisture_str = u16_to_string(moisture);

        // build the value display here
        let values = LinearLayout::vertical(
            Chain::new(Text::new(&light_str, Point::zero(), text_style))
                .append(Text::new(&moisture_str, Point::zero(), text_style)),
        )
        .with_alignment(horizontal::Left)
        .arrange();

        let table =
            LinearLayout::horizontal(Chain::new(labels.clone()).append(values))
                .arrange()
                .align_to(
                    &display.bounding_box(),
                    horizontal::Center,
                    vertical::Center,
                );

        display.fill_solid(&last_table_bounds, Rgb565::CSS_BLACK)?;
        table.draw(display)?;

        last_table_bounds = table.bounds();

        Timer::after(UPDATE_DELAY).await;
    }
}

/// Converts a [`u16`] to a [`heapless::String`].
fn u16_to_string(mut n: u16) -> String<5> {
    let mut s = String::<5>::new();
    while n != 0 {
        s.push(char::from((n % 10) as u8 + 48)).unwrap();
        n /= 10;
    }
    s.chars().rev().collect()
}
