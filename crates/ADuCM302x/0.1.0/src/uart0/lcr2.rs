#[doc = "Reader of register LCR2"]
pub type R = crate::R<u16, super::LCR2>;
#[doc = "Writer for register LCR2"]
pub type W = crate::W<u16, super::LCR2>;
#[doc = "Register LCR2 `reset()`'s with value 0x02"]
impl crate::ResetValue for super::LCR2 {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x02
    }
}
#[doc = "Reader of field `OSR`"]
pub type OSR_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `OSR`"]
pub struct OSR_W<'a> {
    w: &'a mut W,
}
impl<'a> OSR_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x03) | ((value as u16) & 0x03);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:1 - Over Sample Rate"]
    #[inline(always)]
    pub fn osr(&self) -> OSR_R {
        OSR_R::new((self.bits & 0x03) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Over Sample Rate"]
    #[inline(always)]
    pub fn osr(&mut self) -> OSR_W {
        OSR_W { w: self }
    }
}
