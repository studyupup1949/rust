#![doc = "Peripheral access API for ADUCM302X microcontrollers (generated using svd2rust v0.17.0)\n\nYou can find an overview of the API [here].\n\n[here]: https://docs.rs/svd2rust/0.17.0/svd2rust/#peripheral-api"]
#![deny(const_err)]
#![deny(dead_code)]
#![deny(improper_ctypes)]
#![deny(missing_docs)]
#![deny(no_mangle_generic_items)]
#![deny(non_shorthand_field_patterns)]
#![deny(overflowing_literals)]
#![deny(path_statements)]
#![deny(patterns_in_fns_without_body)]
#![deny(private_in_public)]
#![deny(unconditional_recursion)]
#![deny(unused_allocation)]
#![deny(unused_comparisons)]
#![deny(unused_parens)]
#![deny(while_true)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![no_std]
extern crate bare_metal;
extern crate cortex_m;
#[cfg(feature = "rt")]
extern crate cortex_m_rt;
extern crate vcell;
use core::marker::PhantomData;
use core::ops::Deref;
#[doc = r"Number available in the NVIC for configuring priority"]
pub const NVIC_PRIO_BITS: u8 = 3;
#[cfg(feature = "rt")]
extern "C" {
    fn RTC1_EVT();
    fn XINT_EVT0();
    fn XINT_EVT1();
    fn XINT_EVT2();
    fn XINT_EVT3();
    fn WDT_EXP();
    fn PMG0_VREG_OVR();
    fn PMG0_BATT_RANGE();
    fn RTC0_EVT();
    fn SYS_GPIO_INTA();
    fn SYS_GPIO_INTB();
    fn TMR0_EVT();
    fn TMR1_EVT();
    fn UART_EVT();
    fn SPI0_EVT();
    fn SPI2_EVT();
    fn I2C_SLV_EVT();
    fn I2C_MST_EVT();
    fn DMA_CHAN_ERR();
    fn DMA0_CH0_DONE();
    fn DMA0_CH1_DONE();
    fn DMA0_CH2_DONE();
    fn DMA0_CH3_DONE();
    fn DMA0_CH4_DONE();
    fn DMA0_CH5_DONE();
    fn DMA0_CH6_DONE();
    fn DMA0_CH7_DONE();
    fn DMA0_CH8_DONE();
    fn DMA0_CH9_DONE();
    fn DMA0_CH10_DONE();
    fn DMA0_CH11_DONE();
    fn DMA0_CH12_DONE();
    fn DMA0_CH13_DONE();
    fn DMA0_CH14_DONE();
    fn DMA0_CH15_DONE();
    fn SPORT_A_EVT();
    fn SPORT_B_EVT();
    fn CRYPT_EVT();
    fn DMA0_CH24_DONE();
    fn TMR2_EVT();
    fn CLKG_XTAL_OSC_EVT();
    fn SPI1_EVT();
    fn CLKG_PLL_EVT();
    fn RNG0_EVT();
    fn BEEP_EVT();
    fn ADC0_EVT();
    fn DMA0_CH16_DONE();
    fn DMA0_CH17_DONE();
    fn DMA0_CH18_DONE();
    fn DMA0_CH19_DONE();
    fn DMA0_CH20_DONE();
    fn DMA0_CH21_DONE();
    fn DMA0_CH22_DONE();
    fn DMA0_CH23_DONE();
}
#[doc(hidden)]
pub union Vector {
    _handler: unsafe extern "C" fn(),
    _reserved: u32,
}
#[cfg(feature = "rt")]
#[doc(hidden)]
#[link_section = ".vector_table.interrupts"]
#[no_mangle]
pub static __INTERRUPTS: [Vector; 64] = [
    Vector { _handler: RTC1_EVT },
    Vector {
        _handler: XINT_EVT0,
    },
    Vector {
        _handler: XINT_EVT1,
    },
    Vector {
        _handler: XINT_EVT2,
    },
    Vector {
        _handler: XINT_EVT3,
    },
    Vector { _handler: WDT_EXP },
    Vector {
        _handler: PMG0_VREG_OVR,
    },
    Vector {
        _handler: PMG0_BATT_RANGE,
    },
    Vector { _handler: RTC0_EVT },
    Vector {
        _handler: SYS_GPIO_INTA,
    },
    Vector {
        _handler: SYS_GPIO_INTB,
    },
    Vector { _handler: TMR0_EVT },
    Vector { _handler: TMR1_EVT },
    Vector { _reserved: 0 },
    Vector { _handler: UART_EVT },
    Vector { _handler: SPI0_EVT },
    Vector { _handler: SPI2_EVT },
    Vector {
        _handler: I2C_SLV_EVT,
    },
    Vector {
        _handler: I2C_MST_EVT,
    },
    Vector {
        _handler: DMA_CHAN_ERR,
    },
    Vector {
        _handler: DMA0_CH0_DONE,
    },
    Vector {
        _handler: DMA0_CH1_DONE,
    },
    Vector {
        _handler: DMA0_CH2_DONE,
    },
    Vector {
        _handler: DMA0_CH3_DONE,
    },
    Vector {
        _handler: DMA0_CH4_DONE,
    },
    Vector {
        _handler: DMA0_CH5_DONE,
    },
    Vector {
        _handler: DMA0_CH6_DONE,
    },
    Vector {
        _handler: DMA0_CH7_DONE,
    },
    Vector {
        _handler: DMA0_CH8_DONE,
    },
    Vector {
        _handler: DMA0_CH9_DONE,
    },
    Vector {
        _handler: DMA0_CH10_DONE,
    },
    Vector {
        _handler: DMA0_CH11_DONE,
    },
    Vector {
        _handler: DMA0_CH12_DONE,
    },
    Vector {
        _handler: DMA0_CH13_DONE,
    },
    Vector {
        _handler: DMA0_CH14_DONE,
    },
    Vector {
        _handler: DMA0_CH15_DONE,
    },
    Vector {
        _handler: SPORT_A_EVT,
    },
    Vector {
        _handler: SPORT_B_EVT,
    },
    Vector {
        _handler: CRYPT_EVT,
    },
    Vector {
        _handler: DMA0_CH24_DONE,
    },
    Vector { _handler: TMR2_EVT },
    Vector {
        _handler: CLKG_XTAL_OSC_EVT,
    },
    Vector { _handler: SPI1_EVT },
    Vector {
        _handler: CLKG_PLL_EVT,
    },
    Vector { _handler: RNG0_EVT },
    Vector { _handler: BEEP_EVT },
    Vector { _handler: ADC0_EVT },
    Vector { _reserved: 0 },
    Vector { _reserved: 0 },
    Vector { _reserved: 0 },
    Vector { _reserved: 0 },
    Vector { _reserved: 0 },
    Vector { _reserved: 0 },
    Vector { _reserved: 0 },
    Vector { _reserved: 0 },
    Vector { _reserved: 0 },
    Vector {
        _handler: DMA0_CH16_DONE,
    },
    Vector {
        _handler: DMA0_CH17_DONE,
    },
    Vector {
        _handler: DMA0_CH18_DONE,
    },
    Vector {
        _handler: DMA0_CH19_DONE,
    },
    Vector {
        _handler: DMA0_CH20_DONE,
    },
    Vector {
        _handler: DMA0_CH21_DONE,
    },
    Vector {
        _handler: DMA0_CH22_DONE,
    },
    Vector {
        _handler: DMA0_CH23_DONE,
    },
];
#[doc = r"Enumeration of all the interrupts"]
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum Interrupt {
    #[doc = "0 - Event"]
    RTC1_EVT = 0,
    #[doc = "1 - External Wakeup Interrupt n"]
    XINT_EVT0 = 1,
    #[doc = "2 - External Wakeup Interrupt n"]
    XINT_EVT1 = 2,
    #[doc = "3 - External Wakeup Interrupt n"]
    XINT_EVT2 = 3,
    #[doc = "4 - External Wakeup Interrupt n"]
    XINT_EVT3 = 4,
    #[doc = "5 - Expiration"]
    WDT_EXP = 5,
    #[doc = "6 - Voltage Regulator (VREG) Overvoltage"]
    PMG0_VREG_OVR = 6,
    #[doc = "7 - Battery Voltage (VBAT) Out of Range"]
    PMG0_BATT_RANGE = 7,
    #[doc = "8 - Event"]
    RTC0_EVT = 8,
    #[doc = "9 - GPIO Interrupt A"]
    SYS_GPIO_INTA = 9,
    #[doc = "10 - GPIO Interrupt B"]
    SYS_GPIO_INTB = 10,
    #[doc = "11 - Event"]
    TMR0_EVT = 11,
    #[doc = "12 - Event"]
    TMR1_EVT = 12,
    #[doc = "14 - Event"]
    UART_EVT = 14,
    #[doc = "15 - Event"]
    SPI0_EVT = 15,
    #[doc = "16 - Event"]
    SPI2_EVT = 16,
    #[doc = "17 - Slave Event"]
    I2C_SLV_EVT = 17,
    #[doc = "18 - Master Event"]
    I2C_MST_EVT = 18,
    #[doc = "19 - Channel Error"]
    DMA_CHAN_ERR = 19,
    #[doc = "20 - Channel 0 Done"]
    DMA0_CH0_DONE = 20,
    #[doc = "21 - Channel 1 Done"]
    DMA0_CH1_DONE = 21,
    #[doc = "22 - Channel 2 Done"]
    DMA0_CH2_DONE = 22,
    #[doc = "23 - Channel 3 Done"]
    DMA0_CH3_DONE = 23,
    #[doc = "24 - Channel 4 Done"]
    DMA0_CH4_DONE = 24,
    #[doc = "25 - Channel 5 Done"]
    DMA0_CH5_DONE = 25,
    #[doc = "26 - Channel 6 Done"]
    DMA0_CH6_DONE = 26,
    #[doc = "27 - Channel 7 Done"]
    DMA0_CH7_DONE = 27,
    #[doc = "28 - Channel 8 Done"]
    DMA0_CH8_DONE = 28,
    #[doc = "29 - Channel 9 Done"]
    DMA0_CH9_DONE = 29,
    #[doc = "30 - Channel 10 Done"]
    DMA0_CH10_DONE = 30,
    #[doc = "31 - Channel 11 Done"]
    DMA0_CH11_DONE = 31,
    #[doc = "32 - Channel 12 Done"]
    DMA0_CH12_DONE = 32,
    #[doc = "33 - Channel 13 Done"]
    DMA0_CH13_DONE = 33,
    #[doc = "34 - Channel 14 Done"]
    DMA0_CH14_DONE = 34,
    #[doc = "35 - Channel 15 Done"]
    DMA0_CH15_DONE = 35,
    #[doc = "36 - Channel A Event"]
    SPORT_A_EVT = 36,
    #[doc = "37 - Channel B Event"]
    SPORT_B_EVT = 37,
    #[doc = "38 - Event"]
    CRYPT_EVT = 38,
    #[doc = "39 - Channel 24 Done"]
    DMA0_CH24_DONE = 39,
    #[doc = "40 - Event"]
    TMR2_EVT = 40,
    #[doc = "41 - Crystal Oscillator Event"]
    CLKG_XTAL_OSC_EVT = 41,
    #[doc = "42 - Event"]
    SPI1_EVT = 42,
    #[doc = "43 - PLL Event"]
    CLKG_PLL_EVT = 43,
    #[doc = "44 - Event"]
    RNG0_EVT = 44,
    #[doc = "45 - Event"]
    BEEP_EVT = 45,
    #[doc = "46 - Event"]
    ADC0_EVT = 46,
    #[doc = "56 - Channel 16 Done"]
    DMA0_CH16_DONE = 56,
    #[doc = "57 - Channel 17 Done"]
    DMA0_CH17_DONE = 57,
    #[doc = "58 - Channel 18 Done"]
    DMA0_CH18_DONE = 58,
    #[doc = "59 - Channel 19 Done"]
    DMA0_CH19_DONE = 59,
    #[doc = "60 - Channel 20 Done"]
    DMA0_CH20_DONE = 60,
    #[doc = "61 - Channel 21 Done"]
    DMA0_CH21_DONE = 61,
    #[doc = "62 - Channel 22 Done"]
    DMA0_CH22_DONE = 62,
    #[doc = "63 - Channel 23 Done"]
    DMA0_CH23_DONE = 63,
}
unsafe impl bare_metal::Nr for Interrupt {
    #[inline(always)]
    fn nr(&self) -> u8 {
        *self as u8
    }
}
#[cfg(feature = "rt")]
pub use self::Interrupt as interrupt;
pub use cortex_m::peripheral::Peripherals as CorePeripherals;
pub use cortex_m::peripheral::{CBP, CPUID, DCB, DWT, FPB, ITM, MPU, NVIC, SCB, SYST, TPIU};
#[cfg(feature = "rt")]
pub use cortex_m_rt::interrupt;
#[allow(unused_imports)]
use generic::*;
#[doc = r"Common register and bit access and modify traits"]
pub mod generic;
#[doc = "General Purpose Timer"]
pub struct TMR0 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for TMR0 {}
impl TMR0 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const tmr0::RegisterBlock {
        0x4000_0000 as *const _
    }
}
impl Deref for TMR0 {
    type Target = tmr0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*TMR0::ptr() }
    }
}
#[doc = "General Purpose Timer"]
pub mod tmr0;
#[doc = "General Purpose Timer"]
pub struct TMR1 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for TMR1 {}
impl TMR1 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const tmr0::RegisterBlock {
        0x4000_0400 as *const _
    }
}
impl Deref for TMR1 {
    type Target = tmr0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*TMR1::ptr() }
    }
}
#[doc = "General Purpose Timer"]
pub struct TMR2 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for TMR2 {}
impl TMR2 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const tmr0::RegisterBlock {
        0x4000_0800 as *const _
    }
}
impl Deref for TMR2 {
    type Target = tmr0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*TMR2::ptr() }
    }
}
#[doc = "Real-Time Clock"]
pub struct RTC0 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for RTC0 {}
impl RTC0 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const rtc0::RegisterBlock {
        0x4000_1000 as *const _
    }
}
impl Deref for RTC0 {
    type Target = rtc0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*RTC0::ptr() }
    }
}
#[doc = "Real-Time Clock"]
pub mod rtc0;
#[doc = "Real-Time Clock"]
pub struct RTC1 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for RTC1 {}
impl RTC1 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const rtc0::RegisterBlock {
        0x4000_1400 as *const _
    }
}
impl Deref for RTC1 {
    type Target = rtc0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*RTC1::ptr() }
    }
}
#[doc = "System Identification and Debug Enable"]
pub struct SYS {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for SYS {}
impl SYS {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const sys::RegisterBlock {
        0x4000_2000 as *const _
    }
}
impl Deref for SYS {
    type Target = sys::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*SYS::ptr() }
    }
}
#[doc = "System Identification and Debug Enable"]
pub mod sys;
#[doc = "Watchdog Timer"]
pub struct WDT0 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for WDT0 {}
impl WDT0 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const wdt0::RegisterBlock {
        0x4000_2c00 as *const _
    }
}
impl Deref for WDT0 {
    type Target = wdt0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*WDT0::ptr() }
    }
}
#[doc = "Watchdog Timer"]
pub mod wdt0;
#[doc = "I2C Master/Slave"]
pub struct I2C0 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for I2C0 {}
impl I2C0 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const i2c0::RegisterBlock {
        0x4000_3000 as *const _
    }
}
impl Deref for I2C0 {
    type Target = i2c0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*I2C0::ptr() }
    }
}
#[doc = "I2C Master/Slave"]
pub mod i2c0;
#[doc = "Serial Peripheral Interface"]
pub struct SPI0 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for SPI0 {}
impl SPI0 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const spi0::RegisterBlock {
        0x4000_4000 as *const _
    }
}
impl Deref for SPI0 {
    type Target = spi0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*SPI0::ptr() }
    }
}
#[doc = "Serial Peripheral Interface"]
pub mod spi0;
#[doc = "Serial Peripheral Interface"]
pub struct SPI1 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for SPI1 {}
impl SPI1 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const spi0::RegisterBlock {
        0x4000_4400 as *const _
    }
}
impl Deref for SPI1 {
    type Target = spi0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*SPI1::ptr() }
    }
}
#[doc = "Serial Peripheral Interface"]
pub struct SPI2 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for SPI2 {}
impl SPI2 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const spi0::RegisterBlock {
        0x4002_4000 as *const _
    }
}
impl Deref for SPI2 {
    type Target = spi0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*SPI2::ptr() }
    }
}
#[doc = "Unknown"]
pub struct UART0 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for UART0 {}
impl UART0 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const uart0::RegisterBlock {
        0x4000_5000 as *const _
    }
}
impl Deref for UART0 {
    type Target = uart0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*UART0::ptr() }
    }
}
#[doc = "Unknown"]
pub mod uart0;
#[doc = "Beeper Driver"]
pub struct BEEP0 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for BEEP0 {}
impl BEEP0 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const beep0::RegisterBlock {
        0x4000_5c00 as *const _
    }
}
impl Deref for BEEP0 {
    type Target = beep0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*BEEP0::ptr() }
    }
}
#[doc = "Beeper Driver"]
pub mod beep0;
#[doc = "Unknown"]
pub struct ADC0 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for ADC0 {}
impl ADC0 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const adc0::RegisterBlock {
        0x4000_7000 as *const _
    }
}
impl Deref for ADC0 {
    type Target = adc0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*ADC0::ptr() }
    }
}
#[doc = "Unknown"]
pub mod adc0;
#[doc = "DMA"]
pub struct DMA0 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for DMA0 {}
impl DMA0 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const dma0::RegisterBlock {
        0x4001_0000 as *const _
    }
}
impl Deref for DMA0 {
    type Target = dma0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*DMA0::ptr() }
    }
}
#[doc = "DMA"]
pub mod dma0;
#[doc = "Flash Controller"]
pub struct FLCC0 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for FLCC0 {}
impl FLCC0 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const flcc0::RegisterBlock {
        0x4001_8000 as *const _
    }
}
impl Deref for FLCC0 {
    type Target = flcc0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*FLCC0::ptr() }
    }
}
#[doc = "Flash Controller"]
pub mod flcc0;
#[doc = "Cache Controller"]
pub struct FLCC0_CACHE {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for FLCC0_CACHE {}
impl FLCC0_CACHE {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const flcc0_cache::RegisterBlock {
        0x4001_8058 as *const _
    }
}
impl Deref for FLCC0_CACHE {
    type Target = flcc0_cache::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*FLCC0_CACHE::ptr() }
    }
}
#[doc = "Cache Controller"]
pub mod flcc0_cache;
#[doc = "Unknown"]
pub struct GPIO0 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for GPIO0 {}
impl GPIO0 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const gpio0::RegisterBlock {
        0x4002_0000 as *const _
    }
}
impl Deref for GPIO0 {
    type Target = gpio0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*GPIO0::ptr() }
    }
}
#[doc = "Unknown"]
pub mod gpio0;
#[doc = "Unknown"]
pub struct GPIO1 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for GPIO1 {}
impl GPIO1 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const gpio0::RegisterBlock {
        0x4002_0040 as *const _
    }
}
impl Deref for GPIO1 {
    type Target = gpio0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*GPIO1::ptr() }
    }
}
#[doc = "Unknown"]
pub struct GPIO2 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for GPIO2 {}
impl GPIO2 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const gpio0::RegisterBlock {
        0x4002_0080 as *const _
    }
}
impl Deref for GPIO2 {
    type Target = gpio0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*GPIO2::ptr() }
    }
}
#[doc = "Serial Port"]
pub struct SPORT0 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for SPORT0 {}
impl SPORT0 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const sport0::RegisterBlock {
        0x4003_8000 as *const _
    }
}
impl Deref for SPORT0 {
    type Target = sport0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*SPORT0::ptr() }
    }
}
#[doc = "Serial Port"]
pub mod sport0;
#[doc = "CRC Accelerator"]
pub struct CRC0 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for CRC0 {}
impl CRC0 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const crc0::RegisterBlock {
        0x4004_0000 as *const _
    }
}
impl Deref for CRC0 {
    type Target = crc0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*CRC0::ptr() }
    }
}
#[doc = "CRC Accelerator"]
pub mod crc0;
#[doc = "Random Number Generator"]
pub struct RNG0 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for RNG0 {}
impl RNG0 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const rng0::RegisterBlock {
        0x4004_0400 as *const _
    }
}
impl Deref for RNG0 {
    type Target = rng0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*RNG0::ptr() }
    }
}
#[doc = "Random Number Generator"]
pub mod rng0;
#[doc = "Register Map for the Crypto Block"]
pub struct CRYPT0 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for CRYPT0 {}
impl CRYPT0 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const crypt0::RegisterBlock {
        0x4004_4000 as *const _
    }
}
impl Deref for CRYPT0 {
    type Target = crypt0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*CRYPT0::ptr() }
    }
}
#[doc = "Register Map for the Crypto Block"]
pub mod crypt0;
#[doc = "Power Management"]
pub struct PMG0 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for PMG0 {}
impl PMG0 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const pmg0::RegisterBlock {
        0x4004_c000 as *const _
    }
}
impl Deref for PMG0 {
    type Target = pmg0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*PMG0::ptr() }
    }
}
#[doc = "Power Management"]
pub mod pmg0;
#[doc = "External interrupt configuration"]
pub struct XINT0 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for XINT0 {}
impl XINT0 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const xint0::RegisterBlock {
        0x4004_c080 as *const _
    }
}
impl Deref for XINT0 {
    type Target = xint0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*XINT0::ptr() }
    }
}
#[doc = "External interrupt configuration"]
pub mod xint0;
#[doc = "Clocking"]
pub struct CLKG0_OSC {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for CLKG0_OSC {}
impl CLKG0_OSC {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const clkg0_osc::RegisterBlock {
        0x4004_c100 as *const _
    }
}
impl Deref for CLKG0_OSC {
    type Target = clkg0_osc::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*CLKG0_OSC::ptr() }
    }
}
#[doc = "Clocking"]
pub mod clkg0_osc;
#[doc = "Power Management"]
pub struct PMG0_TST {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for PMG0_TST {}
impl PMG0_TST {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const pmg0_tst::RegisterBlock {
        0x4004_c200 as *const _
    }
}
impl Deref for PMG0_TST {
    type Target = pmg0_tst::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*PMG0_TST::ptr() }
    }
}
#[doc = "Power Management"]
pub mod pmg0_tst;
#[doc = "Clocking"]
pub struct CLKG0_CLK {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for CLKG0_CLK {}
impl CLKG0_CLK {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const clkg0_clk::RegisterBlock {
        0x4004_c300 as *const _
    }
}
impl Deref for CLKG0_CLK {
    type Target = clkg0_clk::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*CLKG0_CLK::ptr() }
    }
}
#[doc = "Clocking"]
pub mod clkg0_clk;
#[doc = "Bus matrix"]
pub struct BUSM0 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for BUSM0 {}
impl BUSM0 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const busm0::RegisterBlock {
        0x4004_c800 as *const _
    }
}
impl Deref for BUSM0 {
    type Target = busm0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*BUSM0::ptr() }
    }
}
#[doc = "Bus matrix"]
pub mod busm0;
#[doc = "Cortex-M3 Interrupt Controller"]
pub struct NVIC0 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for NVIC0 {}
impl NVIC0 {
    #[doc = r"Returns a pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const nvic0::RegisterBlock {
        0xe000_e000 as *const _
    }
}
impl Deref for NVIC0 {
    type Target = nvic0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*NVIC0::ptr() }
    }
}
#[doc = "Cortex-M3 Interrupt Controller"]
pub mod nvic0;
#[no_mangle]
static mut DEVICE_PERIPHERALS: bool = false;
#[doc = r"All the peripherals"]
#[allow(non_snake_case)]
pub struct Peripherals {
    #[doc = "TMR0"]
    pub TMR0: TMR0,
    #[doc = "TMR1"]
    pub TMR1: TMR1,
    #[doc = "TMR2"]
    pub TMR2: TMR2,
    #[doc = "RTC0"]
    pub RTC0: RTC0,
    #[doc = "RTC1"]
    pub RTC1: RTC1,
    #[doc = "SYS"]
    pub SYS: SYS,
    #[doc = "WDT0"]
    pub WDT0: WDT0,
    #[doc = "I2C0"]
    pub I2C0: I2C0,
    #[doc = "SPI0"]
    pub SPI0: SPI0,
    #[doc = "SPI1"]
    pub SPI1: SPI1,
    #[doc = "SPI2"]
    pub SPI2: SPI2,
    #[doc = "UART0"]
    pub UART0: UART0,
    #[doc = "BEEP0"]
    pub BEEP0: BEEP0,
    #[doc = "ADC0"]
    pub ADC0: ADC0,
    #[doc = "DMA0"]
    pub DMA0: DMA0,
    #[doc = "FLCC0"]
    pub FLCC0: FLCC0,
    #[doc = "FLCC0_CACHE"]
    pub FLCC0_CACHE: FLCC0_CACHE,
    #[doc = "GPIO0"]
    pub GPIO0: GPIO0,
    #[doc = "GPIO1"]
    pub GPIO1: GPIO1,
    #[doc = "GPIO2"]
    pub GPIO2: GPIO2,
    #[doc = "SPORT0"]
    pub SPORT0: SPORT0,
    #[doc = "CRC0"]
    pub CRC0: CRC0,
    #[doc = "RNG0"]
    pub RNG0: RNG0,
    #[doc = "CRYPT0"]
    pub CRYPT0: CRYPT0,
    #[doc = "PMG0"]
    pub PMG0: PMG0,
    #[doc = "XINT0"]
    pub XINT0: XINT0,
    #[doc = "CLKG0_OSC"]
    pub CLKG0_OSC: CLKG0_OSC,
    #[doc = "PMG0_TST"]
    pub PMG0_TST: PMG0_TST,
    #[doc = "CLKG0_CLK"]
    pub CLKG0_CLK: CLKG0_CLK,
    #[doc = "BUSM0"]
    pub BUSM0: BUSM0,
    #[doc = "NVIC0"]
    pub NVIC0: NVIC0,
}
impl Peripherals {
    #[doc = r"Returns all the peripherals *once*"]
    #[inline]
    pub fn take() -> Option<Self> {
        cortex_m::interrupt::free(|_| {
            if unsafe { DEVICE_PERIPHERALS } {
                None
            } else {
                Some(unsafe { Peripherals::steal() })
            }
        })
    }
    #[doc = r"Unchecked version of `Peripherals::take`"]
    #[inline]
    pub unsafe fn steal() -> Self {
        DEVICE_PERIPHERALS = true;
        Peripherals {
            TMR0: TMR0 {
                _marker: PhantomData,
            },
            TMR1: TMR1 {
                _marker: PhantomData,
            },
            TMR2: TMR2 {
                _marker: PhantomData,
            },
            RTC0: RTC0 {
                _marker: PhantomData,
            },
            RTC1: RTC1 {
                _marker: PhantomData,
            },
            SYS: SYS {
                _marker: PhantomData,
            },
            WDT0: WDT0 {
                _marker: PhantomData,
            },
            I2C0: I2C0 {
                _marker: PhantomData,
            },
            SPI0: SPI0 {
                _marker: PhantomData,
            },
            SPI1: SPI1 {
                _marker: PhantomData,
            },
            SPI2: SPI2 {
                _marker: PhantomData,
            },
            UART0: UART0 {
                _marker: PhantomData,
            },
            BEEP0: BEEP0 {
                _marker: PhantomData,
            },
            ADC0: ADC0 {
                _marker: PhantomData,
            },
            DMA0: DMA0 {
                _marker: PhantomData,
            },
            FLCC0: FLCC0 {
                _marker: PhantomData,
            },
            FLCC0_CACHE: FLCC0_CACHE {
                _marker: PhantomData,
            },
            GPIO0: GPIO0 {
                _marker: PhantomData,
            },
            GPIO1: GPIO1 {
                _marker: PhantomData,
            },
            GPIO2: GPIO2 {
                _marker: PhantomData,
            },
            SPORT0: SPORT0 {
                _marker: PhantomData,
            },
            CRC0: CRC0 {
                _marker: PhantomData,
            },
            RNG0: RNG0 {
                _marker: PhantomData,
            },
            CRYPT0: CRYPT0 {
                _marker: PhantomData,
            },
            PMG0: PMG0 {
                _marker: PhantomData,
            },
            XINT0: XINT0 {
                _marker: PhantomData,
            },
            CLKG0_OSC: CLKG0_OSC {
                _marker: PhantomData,
            },
            PMG0_TST: PMG0_TST {
                _marker: PhantomData,
            },
            CLKG0_CLK: CLKG0_CLK {
                _marker: PhantomData,
            },
            BUSM0: BUSM0 {
                _marker: PhantomData,
            },
            NVIC0: NVIC0 {
                _marker: PhantomData,
            },
        }
    }
}
