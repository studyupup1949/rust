#[doc = "Reader of register MRXCNT"]
pub type R = crate::R<u16, super::MRXCNT>;
#[doc = "Writer for register MRXCNT"]
pub type W = crate::W<u16, super::MRXCNT>;
#[doc = "Register MRXCNT `reset()`'s with value 0"]
impl crate::ResetValue for super::MRXCNT {
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
        self.w.bits = (self.w.bits & !0xff) | ((value as u16) & 0xff);
        self.w
    }
}
#[doc = "Reader of field `EXTEND`"]
pub type EXTEND_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `EXTEND`"]
pub struct EXTEND_W<'a> {
    w: &'a mut W,
}
impl<'a> EXTEND_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 8)) | (((value as u16) & 0x01) << 8);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:7 - Receive Count"]
    #[inline(always)]
    pub fn value(&self) -> VALUE_R {
        VALUE_R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bit 8 - Extended Read"]
    #[inline(always)]
    pub fn extend(&self) -> EXTEND_R {
        EXTEND_R::new(((self.bits >> 8) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bits 0:7 - Receive Count"]
    #[inline(always)]
    pub fn value(&mut self) -> VALUE_W {
        VALUE_W { w: self }
    }
    #[doc = "Bit 8 - Extended Read"]
    #[inline(always)]
    pub fn extend(&mut self) -> EXTEND_W {
        EXTEND_W { w: self }
    }
}
