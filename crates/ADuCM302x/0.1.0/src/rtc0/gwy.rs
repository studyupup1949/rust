#[doc = "Writer for register GWY"]
pub type W = crate::W<u16, super::GWY>;
#[doc = "Register GWY `reset()`'s with value 0"]
impl crate::ResetValue for super::GWY {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Write proxy for field `SWKEY`"]
pub struct SWKEY_W<'a> {
    w: &'a mut W,
}
impl<'a> SWKEY_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u16) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xffff) | ((value as u16) & 0xffff);
        self.w
    }
}
impl W {
    #[doc = "Bits 0:15 - Software-keyed Command Issued by the CPU"]
    #[inline(always)]
    pub fn swkey(&mut self) -> SWKEY_W {
        SWKEY_W { w: self }
    }
}
