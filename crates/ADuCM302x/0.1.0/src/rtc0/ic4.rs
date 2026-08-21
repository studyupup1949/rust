#[doc = "Reader of register IC4"]
pub type R = crate::R<u16, super::IC4>;
#[doc = "Reader of field `IC4`"]
pub type IC4_R = crate::R<u16, u16>;
impl R {
    #[doc = "Bits 0:15 - RTC Input Capture Channel 4"]
    #[inline(always)]
    pub fn ic4(&self) -> IC4_R {
        IC4_R::new((self.bits & 0xffff) as u16)
    }
}
