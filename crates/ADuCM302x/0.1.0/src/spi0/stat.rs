#[doc = "Reader of register STAT"]
pub type R = crate::R<u16, super::STAT>;
#[doc = "Reader of field `IRQ`"]
pub type IRQ_R = crate::R<bool, bool>;
#[doc = "Reader of field `XFRDONE`"]
pub type XFRDONE_R = crate::R<bool, bool>;
#[doc = "Reader of field `TXEMPTY`"]
pub type TXEMPTY_R = crate::R<bool, bool>;
#[doc = "Reader of field `TXDONE`"]
pub type TXDONE_R = crate::R<bool, bool>;
#[doc = "Reader of field `TXUNDR`"]
pub type TXUNDR_R = crate::R<bool, bool>;
#[doc = "Reader of field `TXIRQ`"]
pub type TXIRQ_R = crate::R<bool, bool>;
#[doc = "Reader of field `RXIRQ`"]
pub type RXIRQ_R = crate::R<bool, bool>;
#[doc = "Reader of field `RXOVR`"]
pub type RXOVR_R = crate::R<bool, bool>;
#[doc = "Reader of field `CS`"]
pub type CS_R = crate::R<bool, bool>;
#[doc = "Reader of field `CSERR`"]
pub type CSERR_R = crate::R<bool, bool>;
#[doc = "Reader of field `CSRISE`"]
pub type CSRISE_R = crate::R<bool, bool>;
#[doc = "Reader of field `CSFALL`"]
pub type CSFALL_R = crate::R<bool, bool>;
#[doc = "Reader of field `RDY`"]
pub type RDY_R = crate::R<bool, bool>;
impl R {
    #[doc = "Bit 0 - SPI Interrupt Status"]
    #[inline(always)]
    pub fn irq(&self) -> IRQ_R {
        IRQ_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - SPI Transfer Completion"]
    #[inline(always)]
    pub fn xfrdone(&self) -> XFRDONE_R {
        XFRDONE_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - SPI Tx FIFO Empty Interrupt"]
    #[inline(always)]
    pub fn txempty(&self) -> TXEMPTY_R {
        TXEMPTY_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - SPI Tx Done in Read Command Mode"]
    #[inline(always)]
    pub fn txdone(&self) -> TXDONE_R {
        TXDONE_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - SPI Tx FIFO Underflow"]
    #[inline(always)]
    pub fn txundr(&self) -> TXUNDR_R {
        TXUNDR_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bit 5 - SPI Tx IRQ"]
    #[inline(always)]
    pub fn txirq(&self) -> TXIRQ_R {
        TXIRQ_R::new(((self.bits >> 5) & 0x01) != 0)
    }
    #[doc = "Bit 6 - SPI Rx IRQ"]
    #[inline(always)]
    pub fn rxirq(&self) -> RXIRQ_R {
        RXIRQ_R::new(((self.bits >> 6) & 0x01) != 0)
    }
    #[doc = "Bit 7 - SPI Rx FIFO Overflow"]
    #[inline(always)]
    pub fn rxovr(&self) -> RXOVR_R {
        RXOVR_R::new(((self.bits >> 7) & 0x01) != 0)
    }
    #[doc = "Bit 11 - CS Status"]
    #[inline(always)]
    pub fn cs(&self) -> CS_R {
        CS_R::new(((self.bits >> 11) & 0x01) != 0)
    }
    #[doc = "Bit 12 - Detected a CS Error Condition in Slave Mode"]
    #[inline(always)]
    pub fn cserr(&self) -> CSERR_R {
        CSERR_R::new(((self.bits >> 12) & 0x01) != 0)
    }
    #[doc = "Bit 13 - Detected a Rising Edge on CS, in Slave CON Mode"]
    #[inline(always)]
    pub fn csrise(&self) -> CSRISE_R {
        CSRISE_R::new(((self.bits >> 13) & 0x01) != 0)
    }
    #[doc = "Bit 14 - Detected a Falling Edge on CS, in Slave CON Mode"]
    #[inline(always)]
    pub fn csfall(&self) -> CSFALL_R {
        CSFALL_R::new(((self.bits >> 14) & 0x01) != 0)
    }
    #[doc = "Bit 15 - Detected an Edge on Ready Indicator for Flow Control"]
    #[inline(always)]
    pub fn rdy(&self) -> RDY_R {
        RDY_R::new(((self.bits >> 15) & 0x01) != 0)
    }
}
