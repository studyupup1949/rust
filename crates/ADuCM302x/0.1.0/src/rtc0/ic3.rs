#[doc = "Reader of register IC3"]
pub type R = crate::R<u16, super::IC3>;
#[doc = "Reader of field `IC3`"]
pub type IC3_R = crate::R<u16, u16>;
impl R {
    #[doc = "Bits 0:15 - RTC Input Capture Channel 3"]
    #[inline(always)]
    pub fn ic3(&self) -> IC3_R {
        IC3_R::new((self.bits & 0xffff) as u16)
    }
}
