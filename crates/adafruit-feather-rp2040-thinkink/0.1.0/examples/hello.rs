//! Adafruit Feather RP2040 ThinkInk e-paper display example
#![deny(unsafe_code)]
#![no_main]
#![no_std]

use adafruit_feather_rp2040_thinkink as rp;
use embedded_graphics::prelude::*;
use embedded_hal::{delay::DelayNs, digital::OutputPin};
use embedded_hal_bus::spi::ExclusiveDevice;
use epd_waveshare::{epd1in54, prelude::*};
use hal::{
    clocks::{init_clocks_and_plls, Clock},
    fugit::*,
    gpio, spi,
    watchdog::Watchdog,
    Sio,
};
use panic_halt as _;
use rp::hal;
use rp::{entry, pac, Pins, XOSC_CRYSTAL_FREQ};
use u8g2_fonts::{fonts, types::*, FontRenderer};

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let _core = pac::CorePeripherals::take().unwrap();

    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    let clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let sio = Sio::new(pac.SIO);
    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let epd_busy = pins.epd_busy.into_pull_down_input();
    let epd_reset = pins.epd_reset.into_push_pull_output();
    let epd_dc = pins.epd_dc.into_push_pull_output();
    let epd_sck = pins.epd_sck.into_function();
    let epd_mosi = pins.epd_mosi.into_function();
    let epd_cs = pins.epd_cs.into_push_pull_output();

    let spi = spi::Spi::<_, _, _, 8>::new(pac.SPI0, (epd_mosi, epd_sck)).init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        1_u32.MHz(),
        embedded_hal::spi::MODE_0,
    );

    let mut delay = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let mut spi_delay = delay.clone();
    let mut spi_dev = ExclusiveDevice::new(spi, epd_cs, &mut spi_delay).unwrap();

    let mut disp_delay = delay.clone();
    // To be used with embedded_graphics to render image
    let mut frame = epd1in54::Display1in54::default();
    frame.set_rotation(DisplayRotation::Rotate270);
    // Physical display driver
    let mut display = epd1in54::Epd1in54::new(
        &mut spi_dev,
        epd_busy,
        epd_dc,
        epd_reset,
        &mut disp_delay,
        None,
    )
    .unwrap();

    frame.clear(display.background_color().clone()).unwrap();

    // Render a message
    let font = FontRenderer::new::<fonts::u8g2_font_logisoso24_tf>();
    let message = "Hello there!";
    font.render_aligned(
        message,
        frame.bounding_box().center(),
        VerticalPosition::Center,
        HorizontalAlignment::Center,
        // Inversed color works with both light and dark background
        FontColor::Transparent(display.background_color().inverse()),
        &mut frame,
    )
    .unwrap();

    // Display a message
    display
        .update_and_display_frame(&mut spi_dev, frame.buffer(), &mut disp_delay)
        .unwrap();
    display.sleep(&mut spi_dev, &mut disp_delay).unwrap();

    let mut led = pins.led.into_push_pull_output_in_state(gpio::PinState::Low);

    loop {
        led.set_high().ok();
        delay.delay_ms(2);

        led.set_low().ok();
        delay.delay_ms(2000);
    }
}
