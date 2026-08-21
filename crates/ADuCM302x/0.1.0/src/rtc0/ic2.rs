#[doc = "Reader of register IC2"]
pub type R = crate::R<u16, super::IC2>;
#[doc = "Reader of field `IC2`"]
pub type IC2_R = crate::R<u16, u16>;
impl R {
    #[doc = "Bits 0:15 - RTC Input Capture Channel 2"]
    #[inline(always)]
    pub fn ic2(&self) -> IC2_R {
        IC2_R::new((self.bits & 0xffff) as u16)
    }
}
