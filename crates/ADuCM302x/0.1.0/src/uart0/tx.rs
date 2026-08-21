#[doc = "Writer for register TX"]
pub type W = crate::W<u16, super::TX>;
#[doc = "Register TX `reset()`'s with value 0"]
impl crate::ResetValue for super::TX {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Write proxy for field `THR`"]
pub struct THR_W<'a> {
    w: &'a mut W,
}
impl<'a> THR_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xff) | ((value as u16) & 0xff);
        self.w
    }
}
impl W {
    #[doc = "Bits 0:7 - Transmit Holding Register"]
    #[inline(always)]
    pub fn thr(&mut self) -> THR_W {
        THR_W { w: self }
    }
}
