#![no_std]

// based on https://github.com/espressif/arduino-esp32/blob/3dd4b0d112c16311c39efc10e8eda9f63e513775/variants/adafruit_feather_esp32s3_tft/pins_arduino.h
// based on https://github.com/espressif/esp-idf/blob/f420609c332fbd2d2f7f188c6579d046c9560e42/components/soc/esp32s3/include/soc/soc_caps.h

pub const USB_VID: u32 = 0x239A;
pub const USB_PID: u32 = 0x811D;
pub const USB_MANUFACTURER: &str = "Adafruit";
pub const USB_PRODUCT: &str = "Feather ESP32-S3 TFT";
pub const USB_SERIAL: &str = ""; // Empty string for MAC address

// User LED
pub const LED_BUILTIN: u8 = 13;

// Neopixel
pub const PIN_NEOPIXEL: u8 = 33;
pub const SOC_GPIO_PIN_COUNT: u8 = 49;
pub const RGB_BUILTIN: u8 = PIN_NEOPIXEL + SOC_GPIO_PIN_COUNT;
pub const RGB_BRIGHTNESS: u8 = 64;

pub const NEOPIXEL_NUM: u8 = 1; // number of neopixels
pub const NEOPIXEL_POWER: u8 = 34; // power pin

pub const TFT_I2C_POWER: u8 = 21;
pub const TFT_CS: u8 = 7;
pub const TFT_RST: u8 = 40;
pub const TFT_DC: u8 = 39;
pub const TFT_BACKLITE: u8 = 45;

pub const SDA: u8 = 42;
pub const SCL: u8 = 41;

pub const SS: u8 = 7;
pub const MOSI: u8 = 35;
pub const SCK: u8 = 36;
pub const MISO: u8 = 37;

pub const A0: u8 = 18;
pub const A1: u8 = 17;
pub const A2: u8 = 16;
pub const A3: u8 = 15;
pub const A4: u8 = 14;
pub const A5: u8 = 8;

pub const TX: u8 = 1;
pub const RX: u8 = 2;
pub const TX1: u8 = TX;
pub const RX1: u8 = RX;

pub const T1: u8 = 1;
pub const T2: u8 = 2;
pub const T5: u8 = 5;
pub const T6: u8 = 6;
pub const T8: u8 = 8;
pub const T9: u8 = 9;
pub const T10: u8 = 10;
pub const T11: u8 = 11;
pub const T12: u8 = 12;
pub const T13: u8 = 13;
pub const T14: u8 = 14;

pub const DAC1: u8 = 17;
pub const DAC2: u8 = 18;

#[cfg(feature = "esp-idf-hal")]
pub mod gpio {
    use esp_idf_hal::gpio::*;

    pub type BootButton = Gpio0;
    pub type Tx = Gpio1;
    pub type Rx = Gpio2;

    pub type D5 = Gpio5;
    pub type D6 = Gpio6;

    pub type TftCs = Gpio7;

    pub type A5 = Gpio8;

    pub type D9 = Gpio9;
    pub type D10 = Gpio10;
    pub type D11 = Gpio11;
    pub type D12 = Gpio12;
    pub type D13 = Gpio13;
    pub type LedPin = Gpio13;

    pub type A4 = Gpio14;
    pub type A3 = Gpio15;
    pub type A2 = Gpio16;
    pub type A1 = Gpio17;
    pub type A0 = Gpio18;

    pub type TftI2cPower = Gpio21;

    pub type NeopixelPin = Gpio33;
    pub type NeopixelPower = Gpio34;

    pub type Mosi = Gpio35;
    pub type Sck = Gpio36;
    pub type Miso = Gpio37;

    pub type TftDc = Gpio39;
    pub type TftReset = Gpio40;

    pub type Scl = Gpio41;
    pub type Mtdi = Gpio41;
    pub type Sda = Gpio42;
    pub type Mtms = Gpio42;

    pub type TftBacklight = Gpio45;
}
