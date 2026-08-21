#[doc = "Reader of register TFC"]
pub type R = crate::R<u16, super::TFC>;
#[doc = "Reader of field `TFC`"]
pub type TFC_R = crate::R<u8, u8>;
impl R {
    #[doc = "Bits 0:4 - Current Tx FIFO Data Bytes"]
    #[inline(always)]
    pub fn tfc(&self) -> TFC_R {
        TFC_R::new((self.bits & 0x1f) as u8)
    }
}
