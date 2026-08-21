#[doc = "Reader of register CNT"]
pub type R = crate::R<u16, super::CNT>;
#[doc = "Writer for register CNT"]
pub type W = crate::W<u16, super::CNT>;
#[doc = "Register CNT `reset()`'s with value 0"]
impl crate::ResetValue for super::CNT {
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
        self.w.bits = (self.w.bits & !0x3fff) | ((value as u16) & 0x3fff);
        self.w
    }
}
#[doc = "Reader of field `FRAMECONT`"]
pub type FRAMECONT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `FRAMECONT`"]
pub struct FRAMECONT_W<'a> {
    w: &'a mut W,
}
impl<'a> FRAMECONT_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 15)) | (((value as u16) & 0x01) << 15);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:13 - Transfer Byte Count"]
    #[inline(always)]
    pub fn value(&self) -> VALUE_R {
        VALUE_R::new((self.bits & 0x3fff) as u16)
    }
    #[doc = "Bit 15 - Continue Frame"]
    #[inline(always)]
    pub fn framecont(&self) -> FRAMECONT_R {
        FRAMECONT_R::new(((self.bits >> 15) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bits 0:13 - Transfer Byte Count"]
    #[inline(always)]
    pub fn value(&mut self) -> VALUE_W {
        VALUE_W { w: self }
    }
    #[doc = "Bit 15 - Continue Frame"]
    #[inline(always)]
    pub fn framecont(&mut self) -> FRAMECONT_W {
        FRAMECONT_W { w: self }
    }
}
