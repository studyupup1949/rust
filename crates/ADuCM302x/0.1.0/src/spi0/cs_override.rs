#[doc = "Reader of register CS_OVERRIDE"]
pub type R = crate::R<u16, super::CS_OVERRIDE>;
#[doc = "Writer for register CS_OVERRIDE"]
pub type W = crate::W<u16, super::CS_OVERRIDE>;
#[doc = "Register CS_OVERRIDE `reset()`'s with value 0"]
impl crate::ResetValue for super::CS_OVERRIDE {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `CTL`"]
pub type CTL_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `CTL`"]
pub struct CTL_W<'a> {
    w: &'a mut W,
}
impl<'a> CTL_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x03) | ((value as u16) & 0x03);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:1 - CS Override Control"]
    #[inline(always)]
    pub fn ctl(&self) -> CTL_R {
        CTL_R::new((self.bits & 0x03) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - CS Override Control"]
    #[inline(always)]
    pub fn ctl(&mut self) -> CTL_W {
        CTL_W { w: self }
    }
}
