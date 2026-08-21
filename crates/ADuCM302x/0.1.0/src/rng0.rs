#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - RNG Control Register"]
    pub ctl: CTL,
    _reserved1: [u8; 2usize],
    #[doc = "0x04 - RNG Sample Length Register"]
    pub len: LEN,
    _reserved2: [u8; 2usize],
    #[doc = "0x08 - RNG Status Register"]
    pub stat: STAT,
    _reserved3: [u8; 2usize],
    #[doc = "0x0c - RNG Data Register"]
    pub data: DATA,
    #[doc = "0x10 - Oscillator Count"]
    pub osccnt: OSCCNT,
    #[doc = "0x14 - Oscillator Difference"]
    pub oscdiff: [OSCDIFF; 4],
}
#[doc = "RNG Control Register\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ctl](ctl) module"]
pub type CTL = crate::Reg<u16, _CTL>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CTL;
#[doc = "`read()` method returns [ctl::R](ctl::R) reader structure"]
impl crate::Readable for CTL {}
#[doc = "`write(|w| ..)` method takes [ctl::W](ctl::W) writer structure"]
impl crate::Writable for CTL {}
#[doc = "RNG Control Register"]
pub mod ctl;
#[doc = "RNG Sample Length Register\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [len](len) module"]
pub type LEN = crate::Reg<u16, _LEN>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _LEN;
#[doc = "`read()` method returns [len::R](len::R) reader structure"]
impl crate::Readable for LEN {}
#[doc = "`write(|w| ..)` method takes [len::W](len::W) writer structure"]
impl crate::Writable for LEN {}
#[doc = "RNG Sample Length Register"]
pub mod len;
#[doc = "RNG Status Register\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [stat](stat) module"]
pub type STAT = crate::Reg<u16, _STAT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _STAT;
#[doc = "`read()` method returns [stat::R](stat::R) reader structure"]
impl crate::Readable for STAT {}
#[doc = "`write(|w| ..)` method takes [stat::W](stat::W) writer structure"]
impl crate::Writable for STAT {}
#[doc = "RNG Status Register"]
pub mod stat;
#[doc = "RNG Data Register\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [data](data) module"]
pub type DATA = crate::Reg<u32, _DATA>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _DATA;
#[doc = "`read()` method returns [data::R](data::R) reader structure"]
impl crate::Readable for DATA {}
#[doc = "RNG Data Register"]
pub mod data;
#[doc = "Oscillator Count\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [osccnt](osccnt) module"]
pub type OSCCNT = crate::Reg<u32, _OSCCNT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _OSCCNT;
#[doc = "`read()` method returns [osccnt::R](osccnt::R) reader structure"]
impl crate::Readable for OSCCNT {}
#[doc = "Oscillator Count"]
pub mod osccnt;
#[doc = "Oscillator Difference\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [oscdiff](oscdiff) module"]
pub type OSCDIFF = crate::Reg<u8, _OSCDIFF>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _OSCDIFF;
#[doc = "`read()` method returns [oscdiff::R](oscdiff::R) reader structure"]
impl crate::Readable for OSCDIFF {}
#[doc = "Oscillator Difference"]
pub mod oscdiff;
