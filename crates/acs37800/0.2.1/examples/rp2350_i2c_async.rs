#![no_std]
#![no_main]

use acs37800::prelude::*;
use defmt::{Debug2Format, info, warn};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::i2c::{self, Config};
use embassy_time::Timer;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<embassy_rp::peripherals::I2C0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_rp::init(Default::default());

    let mut i2c_config = Config::default();
    i2c_config.frequency = 400_000; // 400 kHz fast-mode

    let scl = p.PIN_1;
    let sda = p.PIN_0;

    let i2c = i2c::I2c::new_async(p.I2C0, scl, sda, Irqs, i2c_config);

    let mut sensor = Acs37800I2c::builder().i2c(i2c).build();

    loop {
        match sensor.read_eeprom().await {
            Ok(eeprom) => info!("EEPROM snapshot: {}", Debug2Format(&eeprom)),
            Err(err) => warn!("EEPROM read failed: {}", Debug2Format(&err)),
        }

        Timer::after_secs(60).await;
    }
}
