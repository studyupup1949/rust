#[doc = "Reader of register CS_CTL"]
pub type R = crate::R<u16, super::CS_CTL>;
#[doc = "Writer for register CS_CTL"]
pub type W = crate::W<u16, super::CS_CTL>;
#[doc = "Register CS_CTL `reset()`'s with value 0x01"]
impl crate::ResetValue for super::CS_CTL {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x01
    }
}
#[doc = "Reader of field `SEL`"]
pub type SEL_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `SEL`"]
pub struct SEL_W<'a> {
    w: &'a mut W,
}
impl<'a> SEL_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x0f) | ((value as u16) & 0x0f);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:3 - Chip Select Control"]
    #[inline(always)]
    pub fn sel(&self) -> SEL_R {
        SEL_R::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - Chip Select Control"]
    #[inline(always)]
    pub fn sel(&mut self) -> SEL_W {
        SEL_W { w: self }
    }
}
