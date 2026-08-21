#[doc = "Reader of register DIV"]
pub type R = crate::R<u16, super::DIV>;
#[doc = "Writer for register DIV"]
pub type W = crate::W<u16, super::DIV>;
#[doc = "Register DIV `reset()`'s with value 0"]
impl crate::ResetValue for super::DIV {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
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
        self.w.bits = (self.w.bits & !0x3f) | ((value as u16) & 0x3f);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:5 - SPI Clock Divider"]
    #[inline(always)]
    pub fn value(&self) -> VALUE_R {
        VALUE_R::new((self.bits & 0x3f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:5 - SPI Clock Divider"]
    #[inline(always)]
    pub fn value(&mut self) -> VALUE_W {
        VALUE_W { w: self }
    }
}
