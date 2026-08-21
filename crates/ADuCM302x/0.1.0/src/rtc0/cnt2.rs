#[doc = "Reader of register CNT2"]
pub type R = crate::R<u16, super::CNT2>;
#[doc = "Reader of field `VALUE`"]
pub type VALUE_R = crate::R<u16, u16>;
impl R {
    #[doc = "Bits 0:14 - Fractional Bits of the RTC Real-Time Count"]
    #[inline(always)]
    pub fn value(&self) -> VALUE_R {
        VALUE_R::new((self.bits & 0x7fff) as u16)
    }
}
