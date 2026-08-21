#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - Load Value"]
    pub load: LOAD,
    _reserved1: [u8; 2usize],
    #[doc = "0x04 - Current Count Value"]
    pub ccnt: CCNT,
    _reserved2: [u8; 2usize],
    #[doc = "0x08 - Control"]
    pub ctl: CTL,
    _reserved3: [u8; 2usize],
    #[doc = "0x0c - Clear Interrupt"]
    pub restart: RESTART,
    _reserved4: [u8; 10usize],
    #[doc = "0x18 - Status"]
    pub stat: STAT,
}
#[doc = "Load Value\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [load](load) module"]
pub type LOAD = crate::Reg<u16, _LOAD>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _LOAD;
#[doc = "`read()` method returns [load::R](load::R) reader structure"]
impl crate::Readable for LOAD {}
#[doc = "`write(|w| ..)` method takes [load::W](load::W) writer structure"]
impl crate::Writable for LOAD {}
#[doc = "Load Value"]
pub mod load;
#[doc = "Current Count Value\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ccnt](ccnt) module"]
pub type CCNT = crate::Reg<u16, _CCNT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CCNT;
#[doc = "`read()` method returns [ccnt::R](ccnt::R) reader structure"]
impl crate::Readable for CCNT {}
#[doc = "Current Count Value"]
pub mod ccnt;
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
#[doc = "Clear Interrupt\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [restart](restart) module"]
pub type RESTART = crate::Reg<u16, _RESTART>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _RESTART;
#[doc = "`write(|w| ..)` method takes [restart::W](restart::W) writer structure"]
impl crate::Writable for RESTART {}
#[doc = "Clear Interrupt"]
pub mod restart;
#[doc = "Status\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [stat](stat) module"]
pub type STAT = crate::Reg<u16, _STAT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _STAT;
#[doc = "`read()` method returns [stat::R](stat::R) reader structure"]
impl crate::Readable for STAT {}
#[doc = "Status"]
pub mod stat;
