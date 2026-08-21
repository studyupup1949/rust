#[doc = "Reader of register SRX"]
pub type R = crate::R<u16, super::SRX>;
#[doc = "Reader of field `VALUE`"]
pub type VALUE_R = crate::R<u8, u8>;
impl R {
    #[doc = "Bits 0:7 - Slave Receive Register"]
    #[inline(always)]
    pub fn value(&self) -> VALUE_R {
        VALUE_R::new((self.bits & 0xff) as u8)
    }
}
