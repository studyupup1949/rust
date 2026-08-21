#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - Status"]
    pub stat: STAT,
    _reserved1: [u8; 2usize],
    #[doc = "0x04 - Receive"]
    pub rx: RX,
    _reserved2: [u8; 2usize],
    #[doc = "0x08 - Transmit"]
    pub tx: TX,
    _reserved3: [u8; 2usize],
    #[doc = "0x0c - SPI Baud Rate Selection"]
    pub div: DIV,
    _reserved4: [u8; 2usize],
    #[doc = "0x10 - SPI Configuration"]
    pub ctl: CTL,
    _reserved5: [u8; 2usize],
    #[doc = "0x14 - SPI Interrupts Enable"]
    pub ien: IEN,
    _reserved6: [u8; 2usize],
    #[doc = "0x18 - Transfer Byte Count"]
    pub cnt: CNT,
    _reserved7: [u8; 2usize],
    #[doc = "0x1c - SPI DMA Enable"]
    pub dma: DMA,
    _reserved8: [u8; 2usize],
    #[doc = "0x20 - FIFO Status"]
    pub fifo_stat: FIFO_STAT,
    _reserved9: [u8; 2usize],
    #[doc = "0x24 - Read Control"]
    pub rd_ctl: RD_CTL,
    _reserved10: [u8; 2usize],
    #[doc = "0x28 - Flow Control"]
    pub flow_ctl: FLOW_CTL,
    _reserved11: [u8; 2usize],
    #[doc = "0x2c - Wait Timer for Flow Control"]
    pub wait_tmr: WAIT_TMR,
    _reserved12: [u8; 2usize],
    #[doc = "0x30 - Chip Select Control for Multi-slave Connections"]
    pub cs_ctl: CS_CTL,
    _reserved13: [u8; 2usize],
    #[doc = "0x34 - Chip Select Override"]
    pub cs_override: CS_OVERRIDE,
}
#[doc = "Status\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [stat](stat) module"]
pub type STAT = crate::Reg<u16, _STAT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _STAT;
#[doc = "`read()` method returns [stat::R](stat::R) reader structure"]
impl crate::Readable for STAT {}
#[doc = "Status"]
pub mod stat;
#[doc = "Receive\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [rx](rx) module"]
pub type RX = crate::Reg<u16, _RX>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _RX;
#[doc = "`read()` method returns [rx::R](rx::R) reader structure"]
impl crate::Readable for RX {}
#[doc = "Receive"]
pub mod rx;
#[doc = "Transmit\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [tx](tx) module"]
pub type TX = crate::Reg<u16, _TX>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _TX;
#[doc = "`write(|w| ..)` method takes [tx::W](tx::W) writer structure"]
impl crate::Writable for TX {}
#[doc = "Transmit"]
pub mod tx;
#[doc = "SPI Baud Rate Selection\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [div](div) module"]
pub type DIV = crate::Reg<u16, _DIV>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _DIV;
#[doc = "`read()` method returns [div::R](div::R) reader structure"]
impl crate::Readable for DIV {}
#[doc = "`write(|w| ..)` method takes [div::W](div::W) writer structure"]
impl crate::Writable for DIV {}
#[doc = "SPI Baud Rate Selection"]
pub mod div;
#[doc = "SPI Configuration\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ctl](ctl) module"]
pub type CTL = crate::Reg<u16, _CTL>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CTL;
#[doc = "`read()` method returns [ctl::R](ctl::R) reader structure"]
impl crate::Readable for CTL {}
#[doc = "`write(|w| ..)` method takes [ctl::W](ctl::W) writer structure"]
impl crate::Writable for CTL {}
#[doc = "SPI Configuration"]
pub mod ctl;
#[doc = "SPI Interrupts Enable\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ien](ien) module"]
pub type IEN = crate::Reg<u16, _IEN>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _IEN;
#[doc = "`read()` method returns [ien::R](ien::R) reader structure"]
impl crate::Readable for IEN {}
#[doc = "`write(|w| ..)` method takes [ien::W](ien::W) writer structure"]
impl crate::Writable for IEN {}
#[doc = "SPI Interrupts Enable"]
pub mod ien;
#[doc = "Transfer Byte Count\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [cnt](cnt) module"]
pub type CNT = crate::Reg<u16, _CNT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CNT;
#[doc = "`read()` method returns [cnt::R](cnt::R) reader structure"]
impl crate::Readable for CNT {}
#[doc = "`write(|w| ..)` method takes [cnt::W](cnt::W) writer structure"]
impl crate::Writable for CNT {}
#[doc = "Transfer Byte Count"]
pub mod cnt;
#[doc = "SPI DMA Enable\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [dma](dma) module"]
pub type DMA = crate::Reg<u16, _DMA>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _DMA;
#[doc = "`read()` method returns [dma::R](dma::R) reader structure"]
impl crate::Readable for DMA {}
#[doc = "`write(|w| ..)` method takes [dma::W](dma::W) writer structure"]
impl crate::Writable for DMA {}
#[doc = "SPI DMA Enable"]
pub mod dma;
#[doc = "FIFO Status\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [fifo_stat](fifo_stat) module"]
pub type FIFO_STAT = crate::Reg<u16, _FIFO_STAT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _FIFO_STAT;
#[doc = "`read()` method returns [fifo_stat::R](fifo_stat::R) reader structure"]
impl crate::Readable for FIFO_STAT {}
#[doc = "FIFO Status"]
pub mod fifo_stat;
#[doc = "Read Control\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [rd_ctl](rd_ctl) module"]
pub type RD_CTL = crate::Reg<u16, _RD_CTL>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _RD_CTL;
#[doc = "`read()` method returns [rd_ctl::R](rd_ctl::R) reader structure"]
impl crate::Readable for RD_CTL {}
#[doc = "`write(|w| ..)` method takes [rd_ctl::W](rd_ctl::W) writer structure"]
impl crate::Writable for RD_CTL {}
#[doc = "Read Control"]
pub mod rd_ctl;
#[doc = "Flow Control\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [flow_ctl](flow_ctl) module"]
pub type FLOW_CTL = crate::Reg<u16, _FLOW_CTL>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _FLOW_CTL;
#[doc = "`read()` method returns [flow_ctl::R](flow_ctl::R) reader structure"]
impl crate::Readable for FLOW_CTL {}
#[doc = "`write(|w| ..)` method takes [flow_ctl::W](flow_ctl::W) writer structure"]
impl crate::Writable for FLOW_CTL {}
#[doc = "Flow Control"]
pub mod flow_ctl;
#[doc = "Wait Timer for Flow Control\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [wait_tmr](wait_tmr) module"]
pub type WAIT_TMR = crate::Reg<u16, _WAIT_TMR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _WAIT_TMR;
#[doc = "`read()` method returns [wait_tmr::R](wait_tmr::R) reader structure"]
impl crate::Readable for WAIT_TMR {}
#[doc = "`write(|w| ..)` method takes [wait_tmr::W](wait_tmr::W) writer structure"]
impl crate::Writable for WAIT_TMR {}
#[doc = "Wait Timer for Flow Control"]
pub mod wait_tmr;
#[doc = "Chip Select Control for Multi-slave Connections\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [cs_ctl](cs_ctl) module"]
pub type CS_CTL = crate::Reg<u16, _CS_CTL>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CS_CTL;
#[doc = "`read()` method returns [cs_ctl::R](cs_ctl::R) reader structure"]
impl crate::Readable for CS_CTL {}
#[doc = "`write(|w| ..)` method takes [cs_ctl::W](cs_ctl::W) writer structure"]
impl crate::Writable for CS_CTL {}
#[doc = "Chip Select Control for Multi-slave Connections"]
pub mod cs_ctl;
#[doc = "Chip Select Override\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [cs_override](cs_override) module"]
pub type CS_OVERRIDE = crate::Reg<u16, _CS_OVERRIDE>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CS_OVERRIDE;
#[doc = "`read()` method returns [cs_override::R](cs_override::R) reader structure"]
impl crate::Readable for CS_OVERRIDE {}
#[doc = "`write(|w| ..)` method takes [cs_override::W](cs_override::W) writer structure"]
impl crate::Writable for CS_OVERRIDE {}
#[doc = "Chip Select Override"]
pub mod cs_override;
