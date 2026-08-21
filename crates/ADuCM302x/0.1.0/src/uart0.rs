#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    _reserved_0_rx: [u8; 2usize],
    _reserved1: [u8; 2usize],
    #[doc = "0x04 - Interrupt Enable"]
    pub ien: IEN,
    _reserved2: [u8; 2usize],
    #[doc = "0x08 - Interrupt ID"]
    pub iir: IIR,
    _reserved3: [u8; 2usize],
    #[doc = "0x0c - Line Control"]
    pub lcr: LCR,
    _reserved4: [u8; 2usize],
    #[doc = "0x10 - Modem Control"]
    pub mcr: MCR,
    _reserved5: [u8; 2usize],
    #[doc = "0x14 - Line Status"]
    pub lsr: LSR,
    _reserved6: [u8; 2usize],
    #[doc = "0x18 - Modem Status"]
    pub msr: MSR,
    _reserved7: [u8; 2usize],
    #[doc = "0x1c - Scratch Buffer"]
    pub scr: SCR,
    _reserved8: [u8; 2usize],
    #[doc = "0x20 - FIFO Control"]
    pub fcr: FCR,
    _reserved9: [u8; 2usize],
    #[doc = "0x24 - Fractional Baud Rate"]
    pub fbr: FBR,
    _reserved10: [u8; 2usize],
    #[doc = "0x28 - Baud Rate Divider"]
    pub div: DIV,
    _reserved11: [u8; 2usize],
    #[doc = "0x2c - Second Line Control"]
    pub lcr2: LCR2,
    _reserved12: [u8; 2usize],
    #[doc = "0x30 - UART Control Register"]
    pub ctl: CTL,
    _reserved13: [u8; 2usize],
    #[doc = "0x34 - RX FIFO Byte Count"]
    pub rfc: RFC,
    _reserved14: [u8; 2usize],
    #[doc = "0x38 - TX FIFO Byte Count"]
    pub tfc: TFC,
    _reserved15: [u8; 2usize],
    #[doc = "0x3c - RS485 Half-duplex Control"]
    pub rsc: RSC,
    _reserved16: [u8; 2usize],
    #[doc = "0x40 - Auto Baud Control"]
    pub acr: ACR,
    _reserved17: [u8; 2usize],
    #[doc = "0x44 - Auto Baud Status (Low)"]
    pub asrl: ASRL,
    _reserved18: [u8; 2usize],
    #[doc = "0x48 - Auto Baud Status (High)"]
    pub asrh: ASRH,
}
impl RegisterBlock {
    #[doc = "0x00 - Transmit Holding Register"]
    #[inline(always)]
    pub fn tx(&self) -> &TX {
        unsafe { &*(((self as *const Self) as *const u8).add(0usize) as *const TX) }
    }
    #[doc = "0x00 - Transmit Holding Register"]
    #[inline(always)]
    pub fn tx_mut(&self) -> &mut TX {
        unsafe { &mut *(((self as *const Self) as *mut u8).add(0usize) as *mut TX) }
    }
    #[doc = "0x00 - Receive Buffer Register"]
    #[inline(always)]
    pub fn rx(&self) -> &RX {
        unsafe { &*(((self as *const Self) as *const u8).add(0usize) as *const RX) }
    }
    #[doc = "0x00 - Receive Buffer Register"]
    #[inline(always)]
    pub fn rx_mut(&self) -> &mut RX {
        unsafe { &mut *(((self as *const Self) as *mut u8).add(0usize) as *mut RX) }
    }
}
#[doc = "Receive Buffer Register\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [rx](rx) module"]
pub type RX = crate::Reg<u16, _RX>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _RX;
#[doc = "`read()` method returns [rx::R](rx::R) reader structure"]
impl crate::Readable for RX {}
#[doc = "Receive Buffer Register"]
pub mod rx;
#[doc = "Transmit Holding Register\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [tx](tx) module"]
pub type TX = crate::Reg<u16, _TX>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _TX;
#[doc = "`write(|w| ..)` method takes [tx::W](tx::W) writer structure"]
impl crate::Writable for TX {}
#[doc = "Transmit Holding Register"]
pub mod tx;
#[doc = "Interrupt Enable\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ien](ien) module"]
pub type IEN = crate::Reg<u16, _IEN>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _IEN;
#[doc = "`read()` method returns [ien::R](ien::R) reader structure"]
impl crate::Readable for IEN {}
#[doc = "`write(|w| ..)` method takes [ien::W](ien::W) writer structure"]
impl crate::Writable for IEN {}
#[doc = "Interrupt Enable"]
pub mod ien;
#[doc = "Interrupt ID\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [iir](iir) module"]
pub type IIR = crate::Reg<u16, _IIR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _IIR;
#[doc = "`read()` method returns [iir::R](iir::R) reader structure"]
impl crate::Readable for IIR {}
#[doc = "Interrupt ID"]
pub mod iir;
#[doc = "Line Control\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [lcr](lcr) module"]
pub type LCR = crate::Reg<u16, _LCR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _LCR;
#[doc = "`read()` method returns [lcr::R](lcr::R) reader structure"]
impl crate::Readable for LCR {}
#[doc = "`write(|w| ..)` method takes [lcr::W](lcr::W) writer structure"]
impl crate::Writable for LCR {}
#[doc = "Line Control"]
pub mod lcr;
#[doc = "Modem Control\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [mcr](mcr) module"]
pub type MCR = crate::Reg<u16, _MCR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _MCR;
#[doc = "`read()` method returns [mcr::R](mcr::R) reader structure"]
impl crate::Readable for MCR {}
#[doc = "`write(|w| ..)` method takes [mcr::W](mcr::W) writer structure"]
impl crate::Writable for MCR {}
#[doc = "Modem Control"]
pub mod mcr;
#[doc = "Line Status\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [lsr](lsr) module"]
pub type LSR = crate::Reg<u16, _LSR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _LSR;
#[doc = "`read()` method returns [lsr::R](lsr::R) reader structure"]
impl crate::Readable for LSR {}
#[doc = "Line Status"]
pub mod lsr;
#[doc = "Modem Status\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [msr](msr) module"]
pub type MSR = crate::Reg<u16, _MSR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _MSR;
#[doc = "`read()` method returns [msr::R](msr::R) reader structure"]
impl crate::Readable for MSR {}
#[doc = "Modem Status"]
pub mod msr;
#[doc = "Scratch Buffer\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [scr](scr) module"]
pub type SCR = crate::Reg<u16, _SCR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SCR;
#[doc = "`read()` method returns [scr::R](scr::R) reader structure"]
impl crate::Readable for SCR {}
#[doc = "`write(|w| ..)` method takes [scr::W](scr::W) writer structure"]
impl crate::Writable for SCR {}
#[doc = "Scratch Buffer"]
pub mod scr;
#[doc = "FIFO Control\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [fcr](fcr) module"]
pub type FCR = crate::Reg<u16, _FCR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _FCR;
#[doc = "`read()` method returns [fcr::R](fcr::R) reader structure"]
impl crate::Readable for FCR {}
#[doc = "`write(|w| ..)` method takes [fcr::W](fcr::W) writer structure"]
impl crate::Writable for FCR {}
#[doc = "FIFO Control"]
pub mod fcr;
#[doc = "Fractional Baud Rate\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [fbr](fbr) module"]
pub type FBR = crate::Reg<u16, _FBR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _FBR;
#[doc = "`read()` method returns [fbr::R](fbr::R) reader structure"]
impl crate::Readable for FBR {}
#[doc = "`write(|w| ..)` method takes [fbr::W](fbr::W) writer structure"]
impl crate::Writable for FBR {}
#[doc = "Fractional Baud Rate"]
pub mod fbr;
#[doc = "Baud Rate Divider\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [div](div) module"]
pub type DIV = crate::Reg<u16, _DIV>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _DIV;
#[doc = "`read()` method returns [div::R](div::R) reader structure"]
impl crate::Readable for DIV {}
#[doc = "`write(|w| ..)` method takes [div::W](div::W) writer structure"]
impl crate::Writable for DIV {}
#[doc = "Baud Rate Divider"]
pub mod div;
#[doc = "Second Line Control\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [lcr2](lcr2) module"]
pub type LCR2 = crate::Reg<u16, _LCR2>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _LCR2;
#[doc = "`read()` method returns [lcr2::R](lcr2::R) reader structure"]
impl crate::Readable for LCR2 {}
#[doc = "`write(|w| ..)` method takes [lcr2::W](lcr2::W) writer structure"]
impl crate::Writable for LCR2 {}
#[doc = "Second Line Control"]
pub mod lcr2;
#[doc = "UART Control Register\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ctl](ctl) module"]
pub type CTL = crate::Reg<u16, _CTL>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CTL;
#[doc = "`read()` method returns [ctl::R](ctl::R) reader structure"]
impl crate::Readable for CTL {}
#[doc = "`write(|w| ..)` method takes [ctl::W](ctl::W) writer structure"]
impl crate::Writable for CTL {}
#[doc = "UART Control Register"]
pub mod ctl;
#[doc = "RX FIFO Byte Count\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [rfc](rfc) module"]
pub type RFC = crate::Reg<u16, _RFC>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _RFC;
#[doc = "`read()` method returns [rfc::R](rfc::R) reader structure"]
impl crate::Readable for RFC {}
#[doc = "RX FIFO Byte Count"]
pub mod rfc;
#[doc = "TX FIFO Byte Count\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [tfc](tfc) module"]
pub type TFC = crate::Reg<u16, _TFC>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _TFC;
#[doc = "`read()` method returns [tfc::R](tfc::R) reader structure"]
impl crate::Readable for TFC {}
#[doc = "TX FIFO Byte Count"]
pub mod tfc;
#[doc = "RS485 Half-duplex Control\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [rsc](rsc) module"]
pub type RSC = crate::Reg<u16, _RSC>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _RSC;
#[doc = "`read()` method returns [rsc::R](rsc::R) reader structure"]
impl crate::Readable for RSC {}
#[doc = "`write(|w| ..)` method takes [rsc::W](rsc::W) writer structure"]
impl crate::Writable for RSC {}
#[doc = "RS485 Half-duplex Control"]
pub mod rsc;
#[doc = "Auto Baud Control\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [acr](acr) module"]
pub type ACR = crate::Reg<u16, _ACR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ACR;
#[doc = "`read()` method returns [acr::R](acr::R) reader structure"]
impl crate::Readable for ACR {}
#[doc = "`write(|w| ..)` method takes [acr::W](acr::W) writer structure"]
impl crate::Writable for ACR {}
#[doc = "Auto Baud Control"]
pub mod acr;
#[doc = "Auto Baud Status (Low)\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [asrl](asrl) module"]
pub type ASRL = crate::Reg<u16, _ASRL>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ASRL;
#[doc = "`read()` method returns [asrl::R](asrl::R) reader structure"]
impl crate::Readable for ASRL {}
#[doc = "Auto Baud Status (Low)"]
pub mod asrl;
#[doc = "Auto Baud Status (High)\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [asrh](asrh) module"]
pub type ASRH = crate::Reg<u16, _ASRH>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ASRH;
#[doc = "`read()` method returns [asrh::R](asrh::R) reader structure"]
impl crate::Readable for ASRH {}
#[doc = "Auto Baud Status (High)"]
pub mod asrh;
