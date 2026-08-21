#[doc = "Reader of register RX"]
pub type R = crate::R<u16, super::RX>;
#[doc = "Reader of field `BYTE1`"]
pub type BYTE1_R = crate::R<u8, u8>;
#[doc = "Reader of field `BYTE2`"]
pub type BYTE2_R = crate::R<u8, u8>;
impl R {
    #[doc = "Bits 0:7 - 8-bit Receive Buffer"]
    #[inline(always)]
    pub fn byte1(&self) -> BYTE1_R {
        BYTE1_R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - 8-bit Receive Buffer, Used Only in DMA Modes"]
    #[inline(always)]
    pub fn byte2(&self) -> BYTE2_R {
        BYTE2_R::new(((self.bits >> 8) & 0xff) as u8)
    }
}
