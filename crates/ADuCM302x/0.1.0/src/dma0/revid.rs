#[doc = "Reader of register REVID"]
pub type R = crate::R<u32, super::REVID>;
#[doc = "Reader of field `VALUE`"]
pub type VALUE_R = crate::R<u8, u8>;
impl R {
    #[doc = "Bits 0:7 - DMA Controller Revision ID"]
    #[inline(always)]
    pub fn value(&self) -> VALUE_R {
        VALUE_R::new((self.bits & 0xff) as u8)
    }
}
