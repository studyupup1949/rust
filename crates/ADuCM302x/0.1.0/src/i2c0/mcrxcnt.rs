#[doc = "Reader of register MCRXCNT"]
pub type R = crate::R<u16, super::MCRXCNT>;
#[doc = "Reader of field `VALUE`"]
pub type VALUE_R = crate::R<u8, u8>;
impl R {
    #[doc = "Bits 0:7 - Current Receive Count"]
    #[inline(always)]
    pub fn value(&self) -> VALUE_R {
        VALUE_R::new((self.bits & 0xff) as u8)
    }
}
