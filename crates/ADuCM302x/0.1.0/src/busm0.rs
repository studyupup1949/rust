#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - Arbitration Priority Configuration for FLASH and SRAM0"]
    pub arbit0: ARBIT0,
    #[doc = "0x04 - Arbitration Priority Configuration for SRAM1 and SIP"]
    pub arbit1: ARBIT1,
    #[doc = "0x08 - Arbitration Priority Configuration for APB32 and APB16"]
    pub arbit2: ARBIT2,
    #[doc = "0x0c - Arbitration Priority Configuration for APB16 priority for core and for DMA1"]
    pub arbit3: ARBIT3,
}
#[doc = "Arbitration Priority Configuration for FLASH and SRAM0\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [arbit0](arbit0) module"]
pub type ARBIT0 = crate::Reg<u32, _ARBIT0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ARBIT0;
#[doc = "`read()` method returns [arbit0::R](arbit0::R) reader structure"]
impl crate::Readable for ARBIT0 {}
#[doc = "`write(|w| ..)` method takes [arbit0::W](arbit0::W) writer structure"]
impl crate::Writable for ARBIT0 {}
#[doc = "Arbitration Priority Configuration for FLASH and SRAM0"]
pub mod arbit0;
#[doc = "Arbitration Priority Configuration for SRAM1 and SIP\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [arbit1](arbit1) module"]
pub type ARBIT1 = crate::Reg<u32, _ARBIT1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ARBIT1;
#[doc = "`read()` method returns [arbit1::R](arbit1::R) reader structure"]
impl crate::Readable for ARBIT1 {}
#[doc = "`write(|w| ..)` method takes [arbit1::W](arbit1::W) writer structure"]
impl crate::Writable for ARBIT1 {}
#[doc = "Arbitration Priority Configuration for SRAM1 and SIP"]
pub mod arbit1;
#[doc = "Arbitration Priority Configuration for APB32 and APB16\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [arbit2](arbit2) module"]
pub type ARBIT2 = crate::Reg<u32, _ARBIT2>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ARBIT2;
#[doc = "`read()` method returns [arbit2::R](arbit2::R) reader structure"]
impl crate::Readable for ARBIT2 {}
#[doc = "`write(|w| ..)` method takes [arbit2::W](arbit2::W) writer structure"]
impl crate::Writable for ARBIT2 {}
#[doc = "Arbitration Priority Configuration for APB32 and APB16"]
pub mod arbit2;
#[doc = "Arbitration Priority Configuration for APB16 priority for core and for DMA1\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [arbit3](arbit3) module"]
pub type ARBIT3 = crate::Reg<u32, _ARBIT3>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ARBIT3;
#[doc = "`read()` method returns [arbit3::R](arbit3::R) reader structure"]
impl crate::Readable for ARBIT3 {}
#[doc = "`write(|w| ..)` method takes [arbit3::W](arbit3::W) writer structure"]
impl crate::Writable for ARBIT3 {}
#[doc = "Arbitration Priority Configuration for APB16 priority for core and for DMA1"]
pub mod arbit3;
