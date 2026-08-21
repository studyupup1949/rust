#[doc = "Reader of register MRX"]
pub type R = crate::R<u16, super::MRX>;
#[doc = "Reader of field `VALUE`"]
pub type VALUE_R = crate::R<u8, u8>;
impl R {
    #[doc = "Bits 0:7 - Master Receive Register"]
    #[inline(always)]
    pub fn value(&self) -> VALUE_R {
        VALUE_R::new((self.bits & 0xff) as u8)
    }
}
