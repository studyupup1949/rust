#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    _reserved0: [u8; 4usize],
    #[doc = "0x04 - Interrupt Control Type"]
    pub intnum: INTNUM,
    _reserved1: [u8; 8usize],
    #[doc = "0x10 - Systick Control and Status"]
    pub stksta: STKSTA,
    #[doc = "0x14 - Systick Reload Value"]
    pub stkld: STKLD,
    #[doc = "0x18 - Systick Current Value"]
    pub stkval: STKVAL,
    #[doc = "0x1c - Systick Calibration Value"]
    pub stkcal: STKCAL,
    _reserved5: [u8; 224usize],
    #[doc = "0x100 - IRQ0..31 Set_Enable"]
    pub intsete0: INTSETE0,
    #[doc = "0x104 - IRQ32..63 Set_Enable"]
    pub intsete1: INTSETE1,
    _reserved7: [u8; 120usize],
    #[doc = "0x180 - IRQ0..31 Clear_Enable"]
    pub intclre0: INTCLRE0,
    #[doc = "0x184 - IRQ32..63 Clear_Enable"]
    pub intclre1: INTCLRE1,
    _reserved9: [u8; 120usize],
    #[doc = "0x200 - IRQ0..31 Set_Pending"]
    pub intsetp0: INTSETP0,
    #[doc = "0x204 - IRQ32..63 Set_Pending"]
    pub intsetp1: INTSETP1,
    _reserved11: [u8; 120usize],
    #[doc = "0x280 - IRQ0..31 Clear_Pending"]
    pub intclrp0: INTCLRP0,
    #[doc = "0x284 - IRQ32..63 Clear_Pending"]
    pub intclrp1: INTCLRP1,
    _reserved13: [u8; 120usize],
    #[doc = "0x300 - IRQ0..31 Active Bit"]
    pub intact0: INTACT0,
    #[doc = "0x304 - IRQ32..63 Active Bit"]
    pub intact1: INTACT1,
    _reserved15: [u8; 248usize],
    #[doc = "0x400 - IRQ0..3 Priority"]
    pub intpri0: INTPRI0,
    #[doc = "0x404 - IRQ4..7 Priority"]
    pub intpri1: INTPRI1,
    #[doc = "0x408 - IRQ8..11 Priority"]
    pub intpri2: INTPRI2,
    #[doc = "0x40c - IRQ12..15 Priority"]
    pub intpri3: INTPRI3,
    #[doc = "0x410 - IRQ16..19 Priority"]
    pub intpri4: INTPRI4,
    #[doc = "0x414 - IRQ20..23 Priority"]
    pub intpri5: INTPRI5,
    #[doc = "0x418 - IRQ24..27 Priority"]
    pub intpri6: INTPRI6,
    #[doc = "0x41c - IRQ28..31 Priority"]
    pub intpri7: INTPRI7,
    #[doc = "0x420 - IRQ32..35 Priority"]
    pub intpri8: INTPRI8,
    #[doc = "0x424 - IRQ36..39 Priority"]
    pub intpri9: INTPRI9,
    #[doc = "0x428 - IRQ40..43 Priority"]
    pub intpri10: INTPRI10,
    _reserved26: [u8; 2260usize],
    #[doc = "0xd00 - CPUID Base"]
    pub intcpid: INTCPID,
    #[doc = "0xd04 - Interrupt Control State"]
    pub intsta: INTSTA,
    #[doc = "0xd08 - Vector Table Offset"]
    pub intvec: INTVEC,
    #[doc = "0xd0c - Application Interrupt/Reset Control"]
    pub intairc: INTAIRC,
    #[doc = "0xd10 - System Control"]
    pub intcon0: INTCON0,
    _reserved31: [u8; 2usize],
    #[doc = "0xd14 - Configuration Control"]
    pub intcon1: INTCON1,
    #[doc = "0xd18 - System Handlers 4-7 Priority"]
    pub intshprio0: INTSHPRIO0,
    #[doc = "0xd1c - System Handlers 8-11 Priority"]
    pub intshprio1: INTSHPRIO1,
    #[doc = "0xd20 - System Handlers 12-15 Priority"]
    pub intshprio3: INTSHPRIO3,
    #[doc = "0xd24 - System Handler Control and State"]
    pub intshcsr: INTSHCSR,
    #[doc = "0xd28 - Configurable Fault Status"]
    pub intcfsr: INTCFSR,
    #[doc = "0xd2c - Hard Fault Status"]
    pub inthfsr: INTHFSR,
    #[doc = "0xd30 - Debug Fault Status"]
    pub intdfsr: INTDFSR,
    #[doc = "0xd34 - Mem Manage Address"]
    pub intmmar: INTMMAR,
    #[doc = "0xd38 - Bus Fault Address"]
    pub intbfar: INTBFAR,
    #[doc = "0xd3c - Auxiliary Fault Status"]
    pub intafsr: INTAFSR,
    #[doc = "0xd40 - Processor Feature Register 0"]
    pub intpfr0: INTPFR0,
    #[doc = "0xd44 - Processor Feature Register 1"]
    pub intpfr1: INTPFR1,
    #[doc = "0xd48 - Debug Feature Register 0"]
    pub intdfr0: INTDFR0,
    #[doc = "0xd4c - Auxiliary Feature Register 0"]
    pub intafr0: INTAFR0,
    #[doc = "0xd50 - Memory Model Feature Register 0"]
    pub intmmfr0: INTMMFR0,
    #[doc = "0xd54 - Memory Model Feature Register 1"]
    pub intmmfr1: INTMMFR1,
    #[doc = "0xd58 - Memory Model Feature Register 2"]
    pub intmmfr2: INTMMFR2,
    #[doc = "0xd5c - Memory Model Feature Register 3"]
    pub intmmfr3: INTMMFR3,
    #[doc = "0xd60 - ISA Feature Register 0"]
    pub intisar0: INTISAR0,
    #[doc = "0xd64 - ISA Feature Register 1"]
    pub intisar1: INTISAR1,
    #[doc = "0xd68 - ISA Feature Register 2"]
    pub intisar2: INTISAR2,
    #[doc = "0xd6c - ISA Feature Register 3"]
    pub intisar3: INTISAR3,
    #[doc = "0xd70 - ISA Feature Register 4"]
    pub intisar4: INTISAR4,
    _reserved55: [u8; 396usize],
    #[doc = "0xf00 - Software Trigger Interrupt Register"]
    pub inttrgi: INTTRGI,
    _reserved56: [u8; 204usize],
    #[doc = "0xfd0 - Peripheral Identification Register 4"]
    pub intpid4: INTPID4,
    #[doc = "0xfd4 - Peripheral Identification Register 5"]
    pub intpid5: INTPID5,
    #[doc = "0xfd8 - Peripheral Identification Register 6"]
    pub intpid6: INTPID6,
    #[doc = "0xfdc - Peripheral Identification Register 7"]
    pub intpid7: INTPID7,
    #[doc = "0xfe0 - Peripheral Identification Bits7:0"]
    pub intpid0: INTPID0,
    #[doc = "0xfe4 - Peripheral Identification Bits15:8"]
    pub intpid1: INTPID1,
    #[doc = "0xfe8 - Peripheral Identification Bits16:23"]
    pub intpid2: INTPID2,
    #[doc = "0xfec - Peripheral Identification Bits24:31"]
    pub intpid3: INTPID3,
    #[doc = "0xff0 - Component Identification Bits7:0"]
    pub intcid0: INTCID0,
    #[doc = "0xff4 - Component Identification Bits15:8"]
    pub intcid1: INTCID1,
    #[doc = "0xff8 - Component Identification Bits16:23"]
    pub intcid2: INTCID2,
    #[doc = "0xffc - Component Identification Bits24:31"]
    pub intcid3: INTCID3,
}
#[doc = "Interrupt Control Type\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intnum](intnum) module"]
pub type INTNUM = crate::Reg<u32, _INTNUM>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTNUM;
#[doc = "`read()` method returns [intnum::R](intnum::R) reader structure"]
impl crate::Readable for INTNUM {}
#[doc = "`write(|w| ..)` method takes [intnum::W](intnum::W) writer structure"]
impl crate::Writable for INTNUM {}
#[doc = "Interrupt Control Type"]
pub mod intnum;
#[doc = "Systick Control and Status\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [stksta](stksta) module"]
pub type STKSTA = crate::Reg<u32, _STKSTA>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _STKSTA;
#[doc = "`read()` method returns [stksta::R](stksta::R) reader structure"]
impl crate::Readable for STKSTA {}
#[doc = "`write(|w| ..)` method takes [stksta::W](stksta::W) writer structure"]
impl crate::Writable for STKSTA {}
#[doc = "Systick Control and Status"]
pub mod stksta;
#[doc = "Systick Reload Value\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [stkld](stkld) module"]
pub type STKLD = crate::Reg<u32, _STKLD>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _STKLD;
#[doc = "`read()` method returns [stkld::R](stkld::R) reader structure"]
impl crate::Readable for STKLD {}
#[doc = "`write(|w| ..)` method takes [stkld::W](stkld::W) writer structure"]
impl crate::Writable for STKLD {}
#[doc = "Systick Reload Value"]
pub mod stkld;
#[doc = "Systick Current Value\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [stkval](stkval) module"]
pub type STKVAL = crate::Reg<u32, _STKVAL>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _STKVAL;
#[doc = "`read()` method returns [stkval::R](stkval::R) reader structure"]
impl crate::Readable for STKVAL {}
#[doc = "`write(|w| ..)` method takes [stkval::W](stkval::W) writer structure"]
impl crate::Writable for STKVAL {}
#[doc = "Systick Current Value"]
pub mod stkval;
#[doc = "Systick Calibration Value\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [stkcal](stkcal) module"]
pub type STKCAL = crate::Reg<u32, _STKCAL>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _STKCAL;
#[doc = "`read()` method returns [stkcal::R](stkcal::R) reader structure"]
impl crate::Readable for STKCAL {}
#[doc = "`write(|w| ..)` method takes [stkcal::W](stkcal::W) writer structure"]
impl crate::Writable for STKCAL {}
#[doc = "Systick Calibration Value"]
pub mod stkcal;
#[doc = "IRQ0..31 Set_Enable\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intsete0](intsete0) module"]
pub type INTSETE0 = crate::Reg<u32, _INTSETE0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTSETE0;
#[doc = "`read()` method returns [intsete0::R](intsete0::R) reader structure"]
impl crate::Readable for INTSETE0 {}
#[doc = "`write(|w| ..)` method takes [intsete0::W](intsete0::W) writer structure"]
impl crate::Writable for INTSETE0 {}
#[doc = "IRQ0..31 Set_Enable"]
pub mod intsete0;
#[doc = "IRQ32..63 Set_Enable\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intsete1](intsete1) module"]
pub type INTSETE1 = crate::Reg<u32, _INTSETE1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTSETE1;
#[doc = "`read()` method returns [intsete1::R](intsete1::R) reader structure"]
impl crate::Readable for INTSETE1 {}
#[doc = "`write(|w| ..)` method takes [intsete1::W](intsete1::W) writer structure"]
impl crate::Writable for INTSETE1 {}
#[doc = "IRQ32..63 Set_Enable"]
pub mod intsete1;
#[doc = "IRQ0..31 Clear_Enable\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intclre0](intclre0) module"]
pub type INTCLRE0 = crate::Reg<u32, _INTCLRE0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTCLRE0;
#[doc = "`read()` method returns [intclre0::R](intclre0::R) reader structure"]
impl crate::Readable for INTCLRE0 {}
#[doc = "`write(|w| ..)` method takes [intclre0::W](intclre0::W) writer structure"]
impl crate::Writable for INTCLRE0 {}
#[doc = "IRQ0..31 Clear_Enable"]
pub mod intclre0;
#[doc = "IRQ32..63 Clear_Enable\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intclre1](intclre1) module"]
pub type INTCLRE1 = crate::Reg<u32, _INTCLRE1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTCLRE1;
#[doc = "`read()` method returns [intclre1::R](intclre1::R) reader structure"]
impl crate::Readable for INTCLRE1 {}
#[doc = "`write(|w| ..)` method takes [intclre1::W](intclre1::W) writer structure"]
impl crate::Writable for INTCLRE1 {}
#[doc = "IRQ32..63 Clear_Enable"]
pub mod intclre1;
#[doc = "IRQ0..31 Set_Pending\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intsetp0](intsetp0) module"]
pub type INTSETP0 = crate::Reg<u32, _INTSETP0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTSETP0;
#[doc = "`read()` method returns [intsetp0::R](intsetp0::R) reader structure"]
impl crate::Readable for INTSETP0 {}
#[doc = "`write(|w| ..)` method takes [intsetp0::W](intsetp0::W) writer structure"]
impl crate::Writable for INTSETP0 {}
#[doc = "IRQ0..31 Set_Pending"]
pub mod intsetp0;
#[doc = "IRQ32..63 Set_Pending\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intsetp1](intsetp1) module"]
pub type INTSETP1 = crate::Reg<u32, _INTSETP1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTSETP1;
#[doc = "`read()` method returns [intsetp1::R](intsetp1::R) reader structure"]
impl crate::Readable for INTSETP1 {}
#[doc = "`write(|w| ..)` method takes [intsetp1::W](intsetp1::W) writer structure"]
impl crate::Writable for INTSETP1 {}
#[doc = "IRQ32..63 Set_Pending"]
pub mod intsetp1;
#[doc = "IRQ0..31 Clear_Pending\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intclrp0](intclrp0) module"]
pub type INTCLRP0 = crate::Reg<u32, _INTCLRP0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTCLRP0;
#[doc = "`read()` method returns [intclrp0::R](intclrp0::R) reader structure"]
impl crate::Readable for INTCLRP0 {}
#[doc = "`write(|w| ..)` method takes [intclrp0::W](intclrp0::W) writer structure"]
impl crate::Writable for INTCLRP0 {}
#[doc = "IRQ0..31 Clear_Pending"]
pub mod intclrp0;
#[doc = "IRQ32..63 Clear_Pending\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intclrp1](intclrp1) module"]
pub type INTCLRP1 = crate::Reg<u32, _INTCLRP1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTCLRP1;
#[doc = "`read()` method returns [intclrp1::R](intclrp1::R) reader structure"]
impl crate::Readable for INTCLRP1 {}
#[doc = "`write(|w| ..)` method takes [intclrp1::W](intclrp1::W) writer structure"]
impl crate::Writable for INTCLRP1 {}
#[doc = "IRQ32..63 Clear_Pending"]
pub mod intclrp1;
#[doc = "IRQ0..31 Active Bit\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intact0](intact0) module"]
pub type INTACT0 = crate::Reg<u32, _INTACT0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTACT0;
#[doc = "`read()` method returns [intact0::R](intact0::R) reader structure"]
impl crate::Readable for INTACT0 {}
#[doc = "`write(|w| ..)` method takes [intact0::W](intact0::W) writer structure"]
impl crate::Writable for INTACT0 {}
#[doc = "IRQ0..31 Active Bit"]
pub mod intact0;
#[doc = "IRQ32..63 Active Bit\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intact1](intact1) module"]
pub type INTACT1 = crate::Reg<u32, _INTACT1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTACT1;
#[doc = "`read()` method returns [intact1::R](intact1::R) reader structure"]
impl crate::Readable for INTACT1 {}
#[doc = "`write(|w| ..)` method takes [intact1::W](intact1::W) writer structure"]
impl crate::Writable for INTACT1 {}
#[doc = "IRQ32..63 Active Bit"]
pub mod intact1;
#[doc = "IRQ0..3 Priority\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intpri0](intpri0) module"]
pub type INTPRI0 = crate::Reg<u32, _INTPRI0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTPRI0;
#[doc = "`read()` method returns [intpri0::R](intpri0::R) reader structure"]
impl crate::Readable for INTPRI0 {}
#[doc = "`write(|w| ..)` method takes [intpri0::W](intpri0::W) writer structure"]
impl crate::Writable for INTPRI0 {}
#[doc = "IRQ0..3 Priority"]
pub mod intpri0;
#[doc = "IRQ4..7 Priority\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intpri1](intpri1) module"]
pub type INTPRI1 = crate::Reg<u32, _INTPRI1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTPRI1;
#[doc = "`read()` method returns [intpri1::R](intpri1::R) reader structure"]
impl crate::Readable for INTPRI1 {}
#[doc = "`write(|w| ..)` method takes [intpri1::W](intpri1::W) writer structure"]
impl crate::Writable for INTPRI1 {}
#[doc = "IRQ4..7 Priority"]
pub mod intpri1;
#[doc = "IRQ8..11 Priority\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intpri2](intpri2) module"]
pub type INTPRI2 = crate::Reg<u32, _INTPRI2>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTPRI2;
#[doc = "`read()` method returns [intpri2::R](intpri2::R) reader structure"]
impl crate::Readable for INTPRI2 {}
#[doc = "`write(|w| ..)` method takes [intpri2::W](intpri2::W) writer structure"]
impl crate::Writable for INTPRI2 {}
#[doc = "IRQ8..11 Priority"]
pub mod intpri2;
#[doc = "IRQ12..15 Priority\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intpri3](intpri3) module"]
pub type INTPRI3 = crate::Reg<u32, _INTPRI3>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTPRI3;
#[doc = "`read()` method returns [intpri3::R](intpri3::R) reader structure"]
impl crate::Readable for INTPRI3 {}
#[doc = "`write(|w| ..)` method takes [intpri3::W](intpri3::W) writer structure"]
impl crate::Writable for INTPRI3 {}
#[doc = "IRQ12..15 Priority"]
pub mod intpri3;
#[doc = "IRQ16..19 Priority\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intpri4](intpri4) module"]
pub type INTPRI4 = crate::Reg<u32, _INTPRI4>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTPRI4;
#[doc = "`read()` method returns [intpri4::R](intpri4::R) reader structure"]
impl crate::Readable for INTPRI4 {}
#[doc = "`write(|w| ..)` method takes [intpri4::W](intpri4::W) writer structure"]
impl crate::Writable for INTPRI4 {}
#[doc = "IRQ16..19 Priority"]
pub mod intpri4;
#[doc = "IRQ20..23 Priority\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intpri5](intpri5) module"]
pub type INTPRI5 = crate::Reg<u32, _INTPRI5>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTPRI5;
#[doc = "`read()` method returns [intpri5::R](intpri5::R) reader structure"]
impl crate::Readable for INTPRI5 {}
#[doc = "`write(|w| ..)` method takes [intpri5::W](intpri5::W) writer structure"]
impl crate::Writable for INTPRI5 {}
#[doc = "IRQ20..23 Priority"]
pub mod intpri5;
#[doc = "IRQ24..27 Priority\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intpri6](intpri6) module"]
pub type INTPRI6 = crate::Reg<u32, _INTPRI6>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTPRI6;
#[doc = "`read()` method returns [intpri6::R](intpri6::R) reader structure"]
impl crate::Readable for INTPRI6 {}
#[doc = "`write(|w| ..)` method takes [intpri6::W](intpri6::W) writer structure"]
impl crate::Writable for INTPRI6 {}
#[doc = "IRQ24..27 Priority"]
pub mod intpri6;
#[doc = "IRQ28..31 Priority\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intpri7](intpri7) module"]
pub type INTPRI7 = crate::Reg<u32, _INTPRI7>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTPRI7;
#[doc = "`read()` method returns [intpri7::R](intpri7::R) reader structure"]
impl crate::Readable for INTPRI7 {}
#[doc = "`write(|w| ..)` method takes [intpri7::W](intpri7::W) writer structure"]
impl crate::Writable for INTPRI7 {}
#[doc = "IRQ28..31 Priority"]
pub mod intpri7;
#[doc = "IRQ32..35 Priority\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intpri8](intpri8) module"]
pub type INTPRI8 = crate::Reg<u32, _INTPRI8>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTPRI8;
#[doc = "`read()` method returns [intpri8::R](intpri8::R) reader structure"]
impl crate::Readable for INTPRI8 {}
#[doc = "`write(|w| ..)` method takes [intpri8::W](intpri8::W) writer structure"]
impl crate::Writable for INTPRI8 {}
#[doc = "IRQ32..35 Priority"]
pub mod intpri8;
#[doc = "IRQ36..39 Priority\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intpri9](intpri9) module"]
pub type INTPRI9 = crate::Reg<u32, _INTPRI9>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTPRI9;
#[doc = "`read()` method returns [intpri9::R](intpri9::R) reader structure"]
impl crate::Readable for INTPRI9 {}
#[doc = "`write(|w| ..)` method takes [intpri9::W](intpri9::W) writer structure"]
impl crate::Writable for INTPRI9 {}
#[doc = "IRQ36..39 Priority"]
pub mod intpri9;
#[doc = "IRQ40..43 Priority\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intpri10](intpri10) module"]
pub type INTPRI10 = crate::Reg<u32, _INTPRI10>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTPRI10;
#[doc = "`read()` method returns [intpri10::R](intpri10::R) reader structure"]
impl crate::Readable for INTPRI10 {}
#[doc = "`write(|w| ..)` method takes [intpri10::W](intpri10::W) writer structure"]
impl crate::Writable for INTPRI10 {}
#[doc = "IRQ40..43 Priority"]
pub mod intpri10;
#[doc = "CPUID Base\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intcpid](intcpid) module"]
pub type INTCPID = crate::Reg<u32, _INTCPID>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTCPID;
#[doc = "`read()` method returns [intcpid::R](intcpid::R) reader structure"]
impl crate::Readable for INTCPID {}
#[doc = "`write(|w| ..)` method takes [intcpid::W](intcpid::W) writer structure"]
impl crate::Writable for INTCPID {}
#[doc = "CPUID Base"]
pub mod intcpid;
#[doc = "Interrupt Control State\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intsta](intsta) module"]
pub type INTSTA = crate::Reg<u32, _INTSTA>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTSTA;
#[doc = "`read()` method returns [intsta::R](intsta::R) reader structure"]
impl crate::Readable for INTSTA {}
#[doc = "`write(|w| ..)` method takes [intsta::W](intsta::W) writer structure"]
impl crate::Writable for INTSTA {}
#[doc = "Interrupt Control State"]
pub mod intsta;
#[doc = "Vector Table Offset\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intvec](intvec) module"]
pub type INTVEC = crate::Reg<u32, _INTVEC>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTVEC;
#[doc = "`read()` method returns [intvec::R](intvec::R) reader structure"]
impl crate::Readable for INTVEC {}
#[doc = "`write(|w| ..)` method takes [intvec::W](intvec::W) writer structure"]
impl crate::Writable for INTVEC {}
#[doc = "Vector Table Offset"]
pub mod intvec;
#[doc = "Application Interrupt/Reset Control\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intairc](intairc) module"]
pub type INTAIRC = crate::Reg<u32, _INTAIRC>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTAIRC;
#[doc = "`read()` method returns [intairc::R](intairc::R) reader structure"]
impl crate::Readable for INTAIRC {}
#[doc = "`write(|w| ..)` method takes [intairc::W](intairc::W) writer structure"]
impl crate::Writable for INTAIRC {}
#[doc = "Application Interrupt/Reset Control"]
pub mod intairc;
#[doc = "System Control\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intcon0](intcon0) module"]
pub type INTCON0 = crate::Reg<u16, _INTCON0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTCON0;
#[doc = "`read()` method returns [intcon0::R](intcon0::R) reader structure"]
impl crate::Readable for INTCON0 {}
#[doc = "`write(|w| ..)` method takes [intcon0::W](intcon0::W) writer structure"]
impl crate::Writable for INTCON0 {}
#[doc = "System Control"]
pub mod intcon0;
#[doc = "Configuration Control\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intcon1](intcon1) module"]
pub type INTCON1 = crate::Reg<u32, _INTCON1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTCON1;
#[doc = "`read()` method returns [intcon1::R](intcon1::R) reader structure"]
impl crate::Readable for INTCON1 {}
#[doc = "`write(|w| ..)` method takes [intcon1::W](intcon1::W) writer structure"]
impl crate::Writable for INTCON1 {}
#[doc = "Configuration Control"]
pub mod intcon1;
#[doc = "System Handlers 4-7 Priority\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intshprio0](intshprio0) module"]
pub type INTSHPRIO0 = crate::Reg<u32, _INTSHPRIO0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTSHPRIO0;
#[doc = "`read()` method returns [intshprio0::R](intshprio0::R) reader structure"]
impl crate::Readable for INTSHPRIO0 {}
#[doc = "`write(|w| ..)` method takes [intshprio0::W](intshprio0::W) writer structure"]
impl crate::Writable for INTSHPRIO0 {}
#[doc = "System Handlers 4-7 Priority"]
pub mod intshprio0;
#[doc = "System Handlers 8-11 Priority\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intshprio1](intshprio1) module"]
pub type INTSHPRIO1 = crate::Reg<u32, _INTSHPRIO1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTSHPRIO1;
#[doc = "`read()` method returns [intshprio1::R](intshprio1::R) reader structure"]
impl crate::Readable for INTSHPRIO1 {}
#[doc = "`write(|w| ..)` method takes [intshprio1::W](intshprio1::W) writer structure"]
impl crate::Writable for INTSHPRIO1 {}
#[doc = "System Handlers 8-11 Priority"]
pub mod intshprio1;
#[doc = "System Handlers 12-15 Priority\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intshprio3](intshprio3) module"]
pub type INTSHPRIO3 = crate::Reg<u32, _INTSHPRIO3>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTSHPRIO3;
#[doc = "`read()` method returns [intshprio3::R](intshprio3::R) reader structure"]
impl crate::Readable for INTSHPRIO3 {}
#[doc = "`write(|w| ..)` method takes [intshprio3::W](intshprio3::W) writer structure"]
impl crate::Writable for INTSHPRIO3 {}
#[doc = "System Handlers 12-15 Priority"]
pub mod intshprio3;
#[doc = "System Handler Control and State\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intshcsr](intshcsr) module"]
pub type INTSHCSR = crate::Reg<u32, _INTSHCSR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTSHCSR;
#[doc = "`read()` method returns [intshcsr::R](intshcsr::R) reader structure"]
impl crate::Readable for INTSHCSR {}
#[doc = "`write(|w| ..)` method takes [intshcsr::W](intshcsr::W) writer structure"]
impl crate::Writable for INTSHCSR {}
#[doc = "System Handler Control and State"]
pub mod intshcsr;
#[doc = "Configurable Fault Status\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intcfsr](intcfsr) module"]
pub type INTCFSR = crate::Reg<u32, _INTCFSR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTCFSR;
#[doc = "`read()` method returns [intcfsr::R](intcfsr::R) reader structure"]
impl crate::Readable for INTCFSR {}
#[doc = "`write(|w| ..)` method takes [intcfsr::W](intcfsr::W) writer structure"]
impl crate::Writable for INTCFSR {}
#[doc = "Configurable Fault Status"]
pub mod intcfsr;
#[doc = "Hard Fault Status\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [inthfsr](inthfsr) module"]
pub type INTHFSR = crate::Reg<u32, _INTHFSR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTHFSR;
#[doc = "`read()` method returns [inthfsr::R](inthfsr::R) reader structure"]
impl crate::Readable for INTHFSR {}
#[doc = "`write(|w| ..)` method takes [inthfsr::W](inthfsr::W) writer structure"]
impl crate::Writable for INTHFSR {}
#[doc = "Hard Fault Status"]
pub mod inthfsr;
#[doc = "Debug Fault Status\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intdfsr](intdfsr) module"]
pub type INTDFSR = crate::Reg<u32, _INTDFSR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTDFSR;
#[doc = "`read()` method returns [intdfsr::R](intdfsr::R) reader structure"]
impl crate::Readable for INTDFSR {}
#[doc = "`write(|w| ..)` method takes [intdfsr::W](intdfsr::W) writer structure"]
impl crate::Writable for INTDFSR {}
#[doc = "Debug Fault Status"]
pub mod intdfsr;
#[doc = "Mem Manage Address\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intmmar](intmmar) module"]
pub type INTMMAR = crate::Reg<u32, _INTMMAR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTMMAR;
#[doc = "`read()` method returns [intmmar::R](intmmar::R) reader structure"]
impl crate::Readable for INTMMAR {}
#[doc = "`write(|w| ..)` method takes [intmmar::W](intmmar::W) writer structure"]
impl crate::Writable for INTMMAR {}
#[doc = "Mem Manage Address"]
pub mod intmmar;
#[doc = "Bus Fault Address\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intbfar](intbfar) module"]
pub type INTBFAR = crate::Reg<u32, _INTBFAR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTBFAR;
#[doc = "`read()` method returns [intbfar::R](intbfar::R) reader structure"]
impl crate::Readable for INTBFAR {}
#[doc = "`write(|w| ..)` method takes [intbfar::W](intbfar::W) writer structure"]
impl crate::Writable for INTBFAR {}
#[doc = "Bus Fault Address"]
pub mod intbfar;
#[doc = "Auxiliary Fault Status\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intafsr](intafsr) module"]
pub type INTAFSR = crate::Reg<u32, _INTAFSR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTAFSR;
#[doc = "`read()` method returns [intafsr::R](intafsr::R) reader structure"]
impl crate::Readable for INTAFSR {}
#[doc = "`write(|w| ..)` method takes [intafsr::W](intafsr::W) writer structure"]
impl crate::Writable for INTAFSR {}
#[doc = "Auxiliary Fault Status"]
pub mod intafsr;
#[doc = "Processor Feature Register 0\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intpfr0](intpfr0) module"]
pub type INTPFR0 = crate::Reg<u32, _INTPFR0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTPFR0;
#[doc = "`read()` method returns [intpfr0::R](intpfr0::R) reader structure"]
impl crate::Readable for INTPFR0 {}
#[doc = "`write(|w| ..)` method takes [intpfr0::W](intpfr0::W) writer structure"]
impl crate::Writable for INTPFR0 {}
#[doc = "Processor Feature Register 0"]
pub mod intpfr0;
#[doc = "Processor Feature Register 1\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intpfr1](intpfr1) module"]
pub type INTPFR1 = crate::Reg<u32, _INTPFR1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTPFR1;
#[doc = "`read()` method returns [intpfr1::R](intpfr1::R) reader structure"]
impl crate::Readable for INTPFR1 {}
#[doc = "`write(|w| ..)` method takes [intpfr1::W](intpfr1::W) writer structure"]
impl crate::Writable for INTPFR1 {}
#[doc = "Processor Feature Register 1"]
pub mod intpfr1;
#[doc = "Debug Feature Register 0\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intdfr0](intdfr0) module"]
pub type INTDFR0 = crate::Reg<u32, _INTDFR0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTDFR0;
#[doc = "`read()` method returns [intdfr0::R](intdfr0::R) reader structure"]
impl crate::Readable for INTDFR0 {}
#[doc = "`write(|w| ..)` method takes [intdfr0::W](intdfr0::W) writer structure"]
impl crate::Writable for INTDFR0 {}
#[doc = "Debug Feature Register 0"]
pub mod intdfr0;
#[doc = "Auxiliary Feature Register 0\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intafr0](intafr0) module"]
pub type INTAFR0 = crate::Reg<u32, _INTAFR0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTAFR0;
#[doc = "`read()` method returns [intafr0::R](intafr0::R) reader structure"]
impl crate::Readable for INTAFR0 {}
#[doc = "`write(|w| ..)` method takes [intafr0::W](intafr0::W) writer structure"]
impl crate::Writable for INTAFR0 {}
#[doc = "Auxiliary Feature Register 0"]
pub mod intafr0;
#[doc = "Memory Model Feature Register 0\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intmmfr0](intmmfr0) module"]
pub type INTMMFR0 = crate::Reg<u32, _INTMMFR0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTMMFR0;
#[doc = "`read()` method returns [intmmfr0::R](intmmfr0::R) reader structure"]
impl crate::Readable for INTMMFR0 {}
#[doc = "`write(|w| ..)` method takes [intmmfr0::W](intmmfr0::W) writer structure"]
impl crate::Writable for INTMMFR0 {}
#[doc = "Memory Model Feature Register 0"]
pub mod intmmfr0;
#[doc = "Memory Model Feature Register 1\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intmmfr1](intmmfr1) module"]
pub type INTMMFR1 = crate::Reg<u32, _INTMMFR1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTMMFR1;
#[doc = "`read()` method returns [intmmfr1::R](intmmfr1::R) reader structure"]
impl crate::Readable for INTMMFR1 {}
#[doc = "`write(|w| ..)` method takes [intmmfr1::W](intmmfr1::W) writer structure"]
impl crate::Writable for INTMMFR1 {}
#[doc = "Memory Model Feature Register 1"]
pub mod intmmfr1;
#[doc = "Memory Model Feature Register 2\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intmmfr2](intmmfr2) module"]
pub type INTMMFR2 = crate::Reg<u32, _INTMMFR2>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTMMFR2;
#[doc = "`read()` method returns [intmmfr2::R](intmmfr2::R) reader structure"]
impl crate::Readable for INTMMFR2 {}
#[doc = "`write(|w| ..)` method takes [intmmfr2::W](intmmfr2::W) writer structure"]
impl crate::Writable for INTMMFR2 {}
#[doc = "Memory Model Feature Register 2"]
pub mod intmmfr2;
#[doc = "Memory Model Feature Register 3\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intmmfr3](intmmfr3) module"]
pub type INTMMFR3 = crate::Reg<u32, _INTMMFR3>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTMMFR3;
#[doc = "`read()` method returns [intmmfr3::R](intmmfr3::R) reader structure"]
impl crate::Readable for INTMMFR3 {}
#[doc = "`write(|w| ..)` method takes [intmmfr3::W](intmmfr3::W) writer structure"]
impl crate::Writable for INTMMFR3 {}
#[doc = "Memory Model Feature Register 3"]
pub mod intmmfr3;
#[doc = "ISA Feature Register 0\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intisar0](intisar0) module"]
pub type INTISAR0 = crate::Reg<u32, _INTISAR0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTISAR0;
#[doc = "`read()` method returns [intisar0::R](intisar0::R) reader structure"]
impl crate::Readable for INTISAR0 {}
#[doc = "`write(|w| ..)` method takes [intisar0::W](intisar0::W) writer structure"]
impl crate::Writable for INTISAR0 {}
#[doc = "ISA Feature Register 0"]
pub mod intisar0;
#[doc = "ISA Feature Register 1\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intisar1](intisar1) module"]
pub type INTISAR1 = crate::Reg<u32, _INTISAR1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTISAR1;
#[doc = "`read()` method returns [intisar1::R](intisar1::R) reader structure"]
impl crate::Readable for INTISAR1 {}
#[doc = "`write(|w| ..)` method takes [intisar1::W](intisar1::W) writer structure"]
impl crate::Writable for INTISAR1 {}
#[doc = "ISA Feature Register 1"]
pub mod intisar1;
#[doc = "ISA Feature Register 2\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intisar2](intisar2) module"]
pub type INTISAR2 = crate::Reg<u32, _INTISAR2>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTISAR2;
#[doc = "`read()` method returns [intisar2::R](intisar2::R) reader structure"]
impl crate::Readable for INTISAR2 {}
#[doc = "`write(|w| ..)` method takes [intisar2::W](intisar2::W) writer structure"]
impl crate::Writable for INTISAR2 {}
#[doc = "ISA Feature Register 2"]
pub mod intisar2;
#[doc = "ISA Feature Register 3\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intisar3](intisar3) module"]
pub type INTISAR3 = crate::Reg<u32, _INTISAR3>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTISAR3;
#[doc = "`read()` method returns [intisar3::R](intisar3::R) reader structure"]
impl crate::Readable for INTISAR3 {}
#[doc = "`write(|w| ..)` method takes [intisar3::W](intisar3::W) writer structure"]
impl crate::Writable for INTISAR3 {}
#[doc = "ISA Feature Register 3"]
pub mod intisar3;
#[doc = "ISA Feature Register 4\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intisar4](intisar4) module"]
pub type INTISAR4 = crate::Reg<u32, _INTISAR4>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTISAR4;
#[doc = "`read()` method returns [intisar4::R](intisar4::R) reader structure"]
impl crate::Readable for INTISAR4 {}
#[doc = "`write(|w| ..)` method takes [intisar4::W](intisar4::W) writer structure"]
impl crate::Writable for INTISAR4 {}
#[doc = "ISA Feature Register 4"]
pub mod intisar4;
#[doc = "Software Trigger Interrupt Register\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [inttrgi](inttrgi) module"]
pub type INTTRGI = crate::Reg<u32, _INTTRGI>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTTRGI;
#[doc = "`read()` method returns [inttrgi::R](inttrgi::R) reader structure"]
impl crate::Readable for INTTRGI {}
#[doc = "`write(|w| ..)` method takes [inttrgi::W](inttrgi::W) writer structure"]
impl crate::Writable for INTTRGI {}
#[doc = "Software Trigger Interrupt Register"]
pub mod inttrgi;
#[doc = "Peripheral Identification Register 4\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intpid4](intpid4) module"]
pub type INTPID4 = crate::Reg<u32, _INTPID4>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTPID4;
#[doc = "`read()` method returns [intpid4::R](intpid4::R) reader structure"]
impl crate::Readable for INTPID4 {}
#[doc = "`write(|w| ..)` method takes [intpid4::W](intpid4::W) writer structure"]
impl crate::Writable for INTPID4 {}
#[doc = "Peripheral Identification Register 4"]
pub mod intpid4;
#[doc = "Peripheral Identification Register 5\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intpid5](intpid5) module"]
pub type INTPID5 = crate::Reg<u32, _INTPID5>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTPID5;
#[doc = "`read()` method returns [intpid5::R](intpid5::R) reader structure"]
impl crate::Readable for INTPID5 {}
#[doc = "`write(|w| ..)` method takes [intpid5::W](intpid5::W) writer structure"]
impl crate::Writable for INTPID5 {}
#[doc = "Peripheral Identification Register 5"]
pub mod intpid5;
#[doc = "Peripheral Identification Register 6\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intpid6](intpid6) module"]
pub type INTPID6 = crate::Reg<u32, _INTPID6>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTPID6;
#[doc = "`read()` method returns [intpid6::R](intpid6::R) reader structure"]
impl crate::Readable for INTPID6 {}
#[doc = "`write(|w| ..)` method takes [intpid6::W](intpid6::W) writer structure"]
impl crate::Writable for INTPID6 {}
#[doc = "Peripheral Identification Register 6"]
pub mod intpid6;
#[doc = "Peripheral Identification Register 7\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intpid7](intpid7) module"]
pub type INTPID7 = crate::Reg<u32, _INTPID7>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTPID7;
#[doc = "`read()` method returns [intpid7::R](intpid7::R) reader structure"]
impl crate::Readable for INTPID7 {}
#[doc = "`write(|w| ..)` method takes [intpid7::W](intpid7::W) writer structure"]
impl crate::Writable for INTPID7 {}
#[doc = "Peripheral Identification Register 7"]
pub mod intpid7;
#[doc = "Peripheral Identification Bits7:0\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intpid0](intpid0) module"]
pub type INTPID0 = crate::Reg<u32, _INTPID0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTPID0;
#[doc = "`read()` method returns [intpid0::R](intpid0::R) reader structure"]
impl crate::Readable for INTPID0 {}
#[doc = "`write(|w| ..)` method takes [intpid0::W](intpid0::W) writer structure"]
impl crate::Writable for INTPID0 {}
#[doc = "Peripheral Identification Bits7:0"]
pub mod intpid0;
#[doc = "Peripheral Identification Bits15:8\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intpid1](intpid1) module"]
pub type INTPID1 = crate::Reg<u32, _INTPID1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTPID1;
#[doc = "`read()` method returns [intpid1::R](intpid1::R) reader structure"]
impl crate::Readable for INTPID1 {}
#[doc = "`write(|w| ..)` method takes [intpid1::W](intpid1::W) writer structure"]
impl crate::Writable for INTPID1 {}
#[doc = "Peripheral Identification Bits15:8"]
pub mod intpid1;
#[doc = "Peripheral Identification Bits16:23\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intpid2](intpid2) module"]
pub type INTPID2 = crate::Reg<u32, _INTPID2>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTPID2;
#[doc = "`read()` method returns [intpid2::R](intpid2::R) reader structure"]
impl crate::Readable for INTPID2 {}
#[doc = "`write(|w| ..)` method takes [intpid2::W](intpid2::W) writer structure"]
impl crate::Writable for INTPID2 {}
#[doc = "Peripheral Identification Bits16:23"]
pub mod intpid2;
#[doc = "Peripheral Identification Bits24:31\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intpid3](intpid3) module"]
pub type INTPID3 = crate::Reg<u32, _INTPID3>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTPID3;
#[doc = "`read()` method returns [intpid3::R](intpid3::R) reader structure"]
impl crate::Readable for INTPID3 {}
#[doc = "`write(|w| ..)` method takes [intpid3::W](intpid3::W) writer structure"]
impl crate::Writable for INTPID3 {}
#[doc = "Peripheral Identification Bits24:31"]
pub mod intpid3;
#[doc = "Component Identification Bits7:0\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intcid0](intcid0) module"]
pub type INTCID0 = crate::Reg<u32, _INTCID0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTCID0;
#[doc = "`read()` method returns [intcid0::R](intcid0::R) reader structure"]
impl crate::Readable for INTCID0 {}
#[doc = "`write(|w| ..)` method takes [intcid0::W](intcid0::W) writer structure"]
impl crate::Writable for INTCID0 {}
#[doc = "Component Identification Bits7:0"]
pub mod intcid0;
#[doc = "Component Identification Bits15:8\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intcid1](intcid1) module"]
pub type INTCID1 = crate::Reg<u32, _INTCID1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTCID1;
#[doc = "`read()` method returns [intcid1::R](intcid1::R) reader structure"]
impl crate::Readable for INTCID1 {}
#[doc = "`write(|w| ..)` method takes [intcid1::W](intcid1::W) writer structure"]
impl crate::Writable for INTCID1 {}
#[doc = "Component Identification Bits15:8"]
pub mod intcid1;
#[doc = "Component Identification Bits16:23\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intcid2](intcid2) module"]
pub type INTCID2 = crate::Reg<u32, _INTCID2>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTCID2;
#[doc = "`read()` method returns [intcid2::R](intcid2::R) reader structure"]
impl crate::Readable for INTCID2 {}
#[doc = "`write(|w| ..)` method takes [intcid2::W](intcid2::W) writer structure"]
impl crate::Writable for INTCID2 {}
#[doc = "Component Identification Bits16:23"]
pub mod intcid2;
#[doc = "Component Identification Bits24:31\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [intcid3](intcid3) module"]
pub type INTCID3 = crate::Reg<u32, _INTCID3>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _INTCID3;
#[doc = "`read()` method returns [intcid3::R](intcid3::R) reader structure"]
impl crate::Readable for INTCID3 {}
#[doc = "`write(|w| ..)` method takes [intcid3::W](intcid3::W) writer structure"]
impl crate::Writable for INTCID3 {}
#[doc = "Component Identification Bits24:31"]
pub mod intcid3;
