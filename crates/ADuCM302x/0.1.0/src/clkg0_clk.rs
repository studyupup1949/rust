#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - Miscellaneous Clock Settings"]
    pub ctl0: CTL0,
    #[doc = "0x04 - Clock Dividers"]
    pub ctl1: CTL1,
    _reserved2: [u8; 4usize],
    #[doc = "0x0c - System PLL"]
    pub ctl3: CTL3,
    _reserved3: [u8; 4usize],
    #[doc = "0x14 - User Clock Gating Control"]
    pub ctl5: CTL5,
    #[doc = "0x18 - Clocking Status"]
    pub stat0: STAT0,
}
#[doc = "Miscellaneous Clock Settings\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ctl0](ctl0) module"]
pub type CTL0 = crate::Reg<u32, _CTL0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CTL0;
#[doc = "`read()` method returns [ctl0::R](ctl0::R) reader structure"]
impl crate::Readable for CTL0 {}
#[doc = "`write(|w| ..)` method takes [ctl0::W](ctl0::W) writer structure"]
impl crate::Writable for CTL0 {}
#[doc = "Miscellaneous Clock Settings"]
pub mod ctl0;
#[doc = "Clock Dividers\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ctl1](ctl1) module"]
pub type CTL1 = crate::Reg<u32, _CTL1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CTL1;
#[doc = "`read()` method returns [ctl1::R](ctl1::R) reader structure"]
impl crate::Readable for CTL1 {}
#[doc = "`write(|w| ..)` method takes [ctl1::W](ctl1::W) writer structure"]
impl crate::Writable for CTL1 {}
#[doc = "Clock Dividers"]
pub mod ctl1;
#[doc = "System PLL\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ctl3](ctl3) module"]
pub type CTL3 = crate::Reg<u32, _CTL3>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CTL3;
#[doc = "`read()` method returns [ctl3::R](ctl3::R) reader structure"]
impl crate::Readable for CTL3 {}
#[doc = "`write(|w| ..)` method takes [ctl3::W](ctl3::W) writer structure"]
impl crate::Writable for CTL3 {}
#[doc = "System PLL"]
pub mod ctl3;
#[doc = "User Clock Gating Control\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ctl5](ctl5) module"]
pub type CTL5 = crate::Reg<u32, _CTL5>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CTL5;
#[doc = "`read()` method returns [ctl5::R](ctl5::R) reader structure"]
impl crate::Readable for CTL5 {}
#[doc = "`write(|w| ..)` method takes [ctl5::W](ctl5::W) writer structure"]
impl crate::Writable for CTL5 {}
#[doc = "User Clock Gating Control"]
pub mod ctl5;
#[doc = "Clocking Status\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [stat0](stat0) module"]
pub type STAT0 = crate::Reg<u32, _STAT0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _STAT0;
#[doc = "`read()` method returns [stat0::R](stat0::R) reader structure"]
impl crate::Readable for STAT0 {}
#[doc = "`write(|w| ..)` method takes [stat0::W](stat0::W) writer structure"]
impl crate::Writable for STAT0 {}
#[doc = "Clocking Status"]
pub mod stat0;
