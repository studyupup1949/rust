#[doc = "Reader of register TCTL"]
pub type R = crate::R<u16, super::TCTL>;
#[doc = "Writer for register TCTL"]
pub type W = crate::W<u16, super::TCTL>;
#[doc = "Register TCTL `reset()`'s with value 0x05"]
impl crate::ResetValue for super::TCTL {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x05
    }
}
#[doc = "Reader of field `THDATIN`"]
pub type THDATIN_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `THDATIN`"]
pub struct THDATIN_W<'a> {
    w: &'a mut W,
}
impl<'a> THDATIN_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x1f) | ((value as u16) & 0x1f);
        self.w
    }
}
#[doc = "Reader of field `FILTEROFF`"]
pub type FILTEROFF_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `FILTEROFF`"]
pub struct FILTEROFF_W<'a> {
    w: &'a mut W,
}
impl<'a> FILTEROFF_W<'a> {
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
    #[doc = "Bits 0:4 - Data in Hold Start"]
    #[inline(always)]
    pub fn thdatin(&self) -> THDATIN_R {
        THDATIN_R::new((self.bits & 0x1f) as u8)
    }
    #[doc = "Bit 8 - Input Filter Control"]
    #[inline(always)]
    pub fn filteroff(&self) -> FILTEROFF_R {
        FILTEROFF_R::new(((self.bits >> 8) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bits 0:4 - Data in Hold Start"]
    #[inline(always)]
    pub fn thdatin(&mut self) -> THDATIN_W {
        THDATIN_W { w: self }
    }
    #[doc = "Bit 8 - Input Filter Control"]
    #[inline(always)]
    pub fn filteroff(&mut self) -> FILTEROFF_W {
        FILTEROFF_W { w: self }
    }
}
