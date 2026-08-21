#[doc = "Reader of register RX"]
pub type R = crate::R<u16, super::RX>;
#[doc = "Reader of field `RBR`"]
pub type RBR_R = crate::R<u8, u8>;
impl R {
    #[doc = "Bits 0:7 - Receive Buffer Register"]
    #[inline(always)]
    pub fn rbr(&self) -> RBR_R {
        RBR_R::new((self.bits & 0xff) as u8)
    }
}
