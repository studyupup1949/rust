#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - CRC Control"]
    pub ctl: CTL,
    #[doc = "0x04 - Input Data Word"]
    pub ipdata: IPDATA,
    #[doc = "0x08 - CRC Result"]
    pub result: RESULT,
    #[doc = "0x0c - Programmable CRC Polynomial"]
    pub poly: POLY,
    _reserved_4_ipbits: [u8; 8usize],
}
impl RegisterBlock {
    #[doc = "0x10 - Input Data Byte"]
    #[inline(always)]
    pub fn ipbyte(&self) -> &IPBYTE {
        unsafe { &*(((self as *const Self) as *const u8).add(16usize) as *const IPBYTE) }
    }
    #[doc = "0x10 - Input Data Byte"]
    #[inline(always)]
    pub fn ipbyte_mut(&self) -> &mut IPBYTE {
        unsafe { &mut *(((self as *const Self) as *mut u8).add(16usize) as *mut IPBYTE) }
    }
    #[doc = "0x10 - Input Data Bits"]
    #[inline(always)]
    pub fn ipbits(&self) -> &[IPBITS; 8] {
        unsafe { &*(((self as *const Self) as *const u8).add(16usize) as *const [IPBITS; 8]) }
    }
    #[doc = "0x10 - Input Data Bits"]
    #[inline(always)]
    pub fn ipbits_mut(&self) -> &mut [IPBITS; 8] {
        unsafe { &mut *(((self as *const Self) as *mut u8).add(16usize) as *mut [IPBITS; 8]) }
    }
}
#[doc = "CRC Control\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ctl](ctl) module"]
pub type CTL = crate::Reg<u32, _CTL>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CTL;
#[doc = "`read()` method returns [ctl::R](ctl::R) reader structure"]
impl crate::Readable for CTL {}
#[doc = "`write(|w| ..)` method takes [ctl::W](ctl::W) writer structure"]
impl crate::Writable for CTL {}
#[doc = "CRC Control"]
pub mod ctl;
#[doc = "Input Data Word\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ipdata](ipdata) module"]
pub type IPDATA = crate::Reg<u32, _IPDATA>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _IPDATA;
#[doc = "`write(|w| ..)` method takes [ipdata::W](ipdata::W) writer structure"]
impl crate::Writable for IPDATA {}
#[doc = "Input Data Word"]
pub mod ipdata;
#[doc = "CRC Result\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [result](result) module"]
pub type RESULT = crate::Reg<u32, _RESULT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _RESULT;
#[doc = "`read()` method returns [result::R](result::R) reader structure"]
impl crate::Readable for RESULT {}
#[doc = "`write(|w| ..)` method takes [result::W](result::W) writer structure"]
impl crate::Writable for RESULT {}
#[doc = "CRC Result"]
pub mod result;
#[doc = "Programmable CRC Polynomial\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [poly](poly) module"]
pub type POLY = crate::Reg<u32, _POLY>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _POLY;
#[doc = "`read()` method returns [poly::R](poly::R) reader structure"]
impl crate::Readable for POLY {}
#[doc = "`write(|w| ..)` method takes [poly::W](poly::W) writer structure"]
impl crate::Writable for POLY {}
#[doc = "Programmable CRC Polynomial"]
pub mod poly;
#[doc = "Input Data Bits\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ipbits](ipbits) module"]
pub type IPBITS = crate::Reg<u8, _IPBITS>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _IPBITS;
#[doc = "`write(|w| ..)` method takes [ipbits::W](ipbits::W) writer structure"]
impl crate::Writable for IPBITS {}
#[doc = "Input Data Bits"]
pub mod ipbits;
#[doc = "Input Data Byte\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ipbyte](ipbyte) module"]
pub type IPBYTE = crate::Reg<u8, _IPBYTE>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _IPBYTE;
#[doc = "`write(|w| ..)` method takes [ipbyte::W](ipbyte::W) writer structure"]
impl crate::Writable for IPBYTE {}
#[doc = "Input Data Byte"]
pub mod ipbyte;
