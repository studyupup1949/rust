#[doc = "Reader of register CAL_WORD"]
pub type R = crate::R<u16, super::CAL_WORD>;
#[doc = "Writer for register CAL_WORD"]
pub type W = crate::W<u16, super::CAL_WORD>;
#[doc = "Register CAL_WORD `reset()`'s with value 0x40"]
impl crate::ResetValue for super::CAL_WORD {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x40
    }
}
#[doc = "Reader of field `VALUE`"]
pub type VALUE_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `VALUE`"]
pub struct VALUE_W<'a> {
    w: &'a mut W,
}
impl<'a> VALUE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x7f) | ((value as u16) & 0x7f);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:6 - Offset Calibration Word"]
    #[inline(always)]
    pub fn value(&self) -> VALUE_R {
        VALUE_R::new((self.bits & 0x7f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:6 - Offset Calibration Word"]
    #[inline(always)]
    pub fn value(&mut self) -> VALUE_W {
        VALUE_W { w: self }
    }
}
