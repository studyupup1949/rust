#[doc = "Writer for register SHCTL"]
pub type W = crate::W<u16, super::SHCTL>;
#[doc = "Register SHCTL `reset()`'s with value 0"]
impl crate::ResetValue for super::SHCTL {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Write proxy for field `RST`"]
pub struct RST_W<'a> {
    w: &'a mut W,
}
impl<'a> RST_W<'a> {
    #[doc = r"Sets the field bit"]
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x01) | ((value as u16) & 0x01);
        self.w
    }
}
impl W {
    #[doc = "Bit 0 - Reset START STOP Detect Circuit"]
    #[inline(always)]
    pub fn rst(&mut self) -> RST_W {
        RST_W { w: self }
    }
}
