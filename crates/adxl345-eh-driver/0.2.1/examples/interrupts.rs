#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{clock::ClockControl, gpio, i2c::I2C, interrupt, peripherals, prelude::*, Delay};
use esp_println::println;

use core::cell::RefCell;
use core::sync::atomic::AtomicBool;

use critical_section::Mutex;

use adxl345_eh_driver;

static ADXL345_INT1_FLAG: AtomicBool = AtomicBool::new(false);
static ADXL345_INT1_PIN: Mutex<RefCell<Option<gpio::Gpio18<gpio::Input<gpio::PullDown>>>>> =
    Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = peripherals::Peripherals::take();
    let system = peripherals.SYSTEM.split();

    let clocks = ClockControl::max(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    let io = gpio::IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let sda = io.pins.gpio5;
    let scl = io.pins.gpio6;
    let i2c = I2C::new(peripherals.I2C0, sda, scl, 100u32.kHz(), &clocks);
    delay.delay_ms(100u32);

    let mut accel_int1 = io.pins.gpio18.into_pull_down_input();

    let mut accel =
        adxl345_eh_driver::Driver::new(i2c, Some(adxl345_eh_driver::address::SECONDARY)).unwrap();
    accel
        .set_tap_detection_ints_defaults(Some(adxl345_eh_driver::InterruptMapPin::INT1), None)
        .unwrap();

    interrupt::enable(peripherals::Interrupt::GPIO, interrupt::Priority::Priority1).unwrap();
    critical_section::with(|cs| {
        accel_int1.listen(gpio::Event::RisingEdge);
        ADXL345_INT1_PIN.borrow_ref_mut(cs).replace(accel_int1)
    });

    loop {
        println!("Loop...");
        if ADXL345_INT1_FLAG.load(core::sync::atomic::Ordering::Relaxed) {
            println!("Tap detected!");
            println!("Clearing interrupts");
            accel.clear_ints().unwrap();
            ADXL345_INT1_FLAG.store(false, core::sync::atomic::Ordering::Relaxed);
        }

        delay.delay_ms(1000u32);
    }
}

#[interrupt]
fn GPIO() {
    critical_section::with(|cs| {
        ADXL345_INT1_PIN
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt();

        ADXL345_INT1_FLAG.store(true, core::sync::atomic::Ordering::Relaxed);
    });
}
