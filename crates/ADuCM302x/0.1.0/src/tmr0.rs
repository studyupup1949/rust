#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - 16-bit Load Value"]
    pub load: LOAD,
    _reserved1: [u8; 2usize],
    #[doc = "0x04 - 16-bit Timer Value"]
    pub curcnt: CURCNT,
    _reserved2: [u8; 2usize],
    #[doc = "0x08 - Control"]
    pub ctl: CTL,
    _reserved3: [u8; 2usize],
    #[doc = "0x0c - Clear Interrupt"]
    pub clrint: CLRINT,
    _reserved4: [u8; 2usize],
    #[doc = "0x10 - Capture"]
    pub capture: CAPTURE,
    _reserved5: [u8; 2usize],
    #[doc = "0x14 - 16-bit Load Value, Asynchronous"]
    pub aload: ALOAD,
    _reserved6: [u8; 2usize],
    #[doc = "0x18 - 16-bit Timer Value, Asynchronous"]
    pub acurcnt: ACURCNT,
    _reserved7: [u8; 2usize],
    #[doc = "0x1c - Status"]
    pub stat: STAT,
    _reserved8: [u8; 2usize],
    #[doc = "0x20 - PWM Control Register"]
    pub pwmctl: PWMCTL,
    _reserved9: [u8; 2usize],
    #[doc = "0x24 - PWM Match Value"]
    pub pwmmatch: PWMMATCH,
}
#[doc = "16-bit Load Value\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [load](load) module"]
pub type LOAD = crate::Reg<u16, _LOAD>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _LOAD;
#[doc = "`read()` method returns [load::R](load::R) reader structure"]
impl crate::Readable for LOAD {}
#[doc = "`write(|w| ..)` method takes [load::W](load::W) writer structure"]
impl crate::Writable for LOAD {}
#[doc = "16-bit Load Value"]
pub mod load;
#[doc = "16-bit Timer Value\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [curcnt](curcnt) module"]
pub type CURCNT = crate::Reg<u16, _CURCNT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CURCNT;
#[doc = "`read()` method returns [curcnt::R](curcnt::R) reader structure"]
impl crate::Readable for CURCNT {}
#[doc = "16-bit Timer Value"]
pub mod curcnt;
#[doc = "Control\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ctl](ctl) module"]
pub type CTL = crate::Reg<u16, _CTL>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CTL;
#[doc = "`read()` method returns [ctl::R](ctl::R) reader structure"]
impl crate::Readable for CTL {}
#[doc = "`write(|w| ..)` method takes [ctl::W](ctl::W) writer structure"]
impl crate::Writable for CTL {}
#[doc = "Control"]
pub mod ctl;
#[doc = "Clear Interrupt\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [clrint](clrint) module"]
pub type CLRINT = crate::Reg<u16, _CLRINT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CLRINT;
#[doc = "`write(|w| ..)` method takes [clrint::W](clrint::W) writer structure"]
impl crate::Writable for CLRINT {}
#[doc = "Clear Interrupt"]
pub mod clrint;
#[doc = "Capture\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [capture](capture) module"]
pub type CAPTURE = crate::Reg<u16, _CAPTURE>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CAPTURE;
#[doc = "`read()` method returns [capture::R](capture::R) reader structure"]
impl crate::Readable for CAPTURE {}
#[doc = "Capture"]
pub mod capture;
#[doc = "16-bit Load Value, Asynchronous\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [aload](aload) module"]
pub type ALOAD = crate::Reg<u16, _ALOAD>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ALOAD;
#[doc = "`read()` method returns [aload::R](aload::R) reader structure"]
impl crate::Readable for ALOAD {}
#[doc = "`write(|w| ..)` method takes [aload::W](aload::W) writer structure"]
impl crate::Writable for ALOAD {}
#[doc = "16-bit Load Value, Asynchronous"]
pub mod aload;
#[doc = "16-bit Timer Value, Asynchronous\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [acurcnt](acurcnt) module"]
pub type ACURCNT = crate::Reg<u16, _ACURCNT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ACURCNT;
#[doc = "`read()` method returns [acurcnt::R](acurcnt::R) reader structure"]
impl crate::Readable for ACURCNT {}
#[doc = "16-bit Timer Value, Asynchronous"]
pub mod acurcnt;
#[doc = "Status\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [stat](stat) module"]
pub type STAT = crate::Reg<u16, _STAT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _STAT;
#[doc = "`read()` method returns [stat::R](stat::R) reader structure"]
impl crate::Readable for STAT {}
#[doc = "Status"]
pub mod stat;
#[doc = "PWM Control Register\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [pwmctl](pwmctl) module"]
pub type PWMCTL = crate::Reg<u16, _PWMCTL>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _PWMCTL;
#[doc = "`read()` method returns [pwmctl::R](pwmctl::R) reader structure"]
impl crate::Readable for PWMCTL {}
#[doc = "`write(|w| ..)` method takes [pwmctl::W](pwmctl::W) writer structure"]
impl crate::Writable for PWMCTL {}
#[doc = "PWM Control Register"]
pub mod pwmctl;
#[doc = "PWM Match Value\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [pwmmatch](pwmmatch) module"]
pub type PWMMATCH = crate::Reg<u16, _PWMMATCH>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _PWMMATCH;
#[doc = "`read()` method returns [pwmmatch::R](pwmmatch::R) reader structure"]
impl crate::Readable for PWMMATCH {}
#[doc = "`write(|w| ..)` method takes [pwmmatch::W](pwmmatch::W) writer structure"]
impl crate::Writable for PWMMATCH {}
#[doc = "PWM Match Value"]
pub mod pwmmatch;
