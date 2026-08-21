#[doc = "Reader of register FIFO_STAT"]
pub type R = crate::R<u16, super::FIFO_STAT>;
#[doc = "Reader of field `TX`"]
pub type TX_R = crate::R<u8, u8>;
#[doc = "Reader of field `RX`"]
pub type RX_R = crate::R<u8, u8>;
impl R {
    #[doc = "Bits 0:3 - SPI Tx FIFO Status"]
    #[inline(always)]
    pub fn tx(&self) -> TX_R {
        TX_R::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bits 8:11 - SPI Rx FIFO Dtatus"]
    #[inline(always)]
    pub fn rx(&self) -> RX_R {
        RX_R::new(((self.bits >> 8) & 0x0f) as u8)
    }
}
