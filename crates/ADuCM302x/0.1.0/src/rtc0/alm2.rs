#[doc = "Reader of register ALM2"]
pub type R = crate::R<u16, super::ALM2>;
#[doc = "Writer for register ALM2"]
pub type W = crate::W<u16, super::ALM2>;
#[doc = "Register ALM2 `reset()`'s with value 0"]
impl crate::ResetValue for super::ALM2 {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `VALUE`"]
pub type VALUE_R = crate::R<u16, u16>;
#[doc = "Write proxy for field `VALUE`"]
pub struct VALUE_W<'a> {
    w: &'a mut W,
}
impl<'a> VALUE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u16) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x7fff) | ((value as u16) & 0x7fff);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:14 - Fractional Bits of the Alarm Target Time"]
    #[inline(always)]
    pub fn value(&self) -> VALUE_R {
        VALUE_R::new((self.bits & 0x7fff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:14 - Fractional Bits of the Alarm Target Time"]
    #[inline(always)]
    pub fn value(&mut self) -> VALUE_W {
        VALUE_W { w: self }
    }
}
