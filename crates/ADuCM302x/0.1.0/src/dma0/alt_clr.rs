#[doc = "Writer for register ALT_CLR"]
pub type W = crate::W<u32, super::ALT_CLR>;
#[doc = "Register ALT_CLR `reset()`'s with value 0"]
impl crate::ResetValue for super::ALT_CLR {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Write proxy for field `CHAN`"]
pub struct CHAN_W<'a> {
    w: &'a mut W,
}
impl<'a> CHAN_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u32) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x01ff_ffff) | ((value as u32) & 0x01ff_ffff);
        self.w
    }
}
impl W {
    #[doc = "Bits 0:24 - Select Primary Data Structure"]
    #[inline(always)]
    pub fn chan(&mut self) -> CHAN_W {
        CHAN_W { w: self }
    }
}
