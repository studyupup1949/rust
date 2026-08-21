#[doc = "Writer for register RESTART"]
pub type W = crate::W<u16, super::RESTART>;
#[doc = "Register RESTART `reset()`'s with value 0"]
impl crate::ResetValue for super::RESTART {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Write proxy for field `CLRWORD`"]
pub struct CLRWORD_W<'a> {
    w: &'a mut W,
}
impl<'a> CLRWORD_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u16) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xffff) | ((value as u16) & 0xffff);
        self.w
    }
}
impl W {
    #[doc = "Bits 0:15 - Clear Watchdog"]
    #[inline(always)]
    pub fn clrword(&mut self) -> CLRWORD_W {
        CLRWORD_W { w: self }
    }
}
