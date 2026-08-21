#[doc = "Reader of register AVG_CFG"]
pub type R = crate::R<u16, super::AVG_CFG>;
#[doc = "Writer for register AVG_CFG"]
pub type W = crate::W<u16, super::AVG_CFG>;
#[doc = "Register AVG_CFG `reset()`'s with value 0x4008"]
impl crate::ResetValue for super::AVG_CFG {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x4008
    }
}
#[doc = "Reader of field `FACTOR`"]
pub type FACTOR_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `FACTOR`"]
pub struct FACTOR_W<'a> {
    w: &'a mut W,
}
impl<'a> FACTOR_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xff) | ((value as u16) & 0xff);
        self.w
    }
}
#[doc = "Reader of field `OS`"]
pub type OS_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `OS`"]
pub struct OS_W<'a> {
    w: &'a mut W,
}
impl<'a> OS_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 14)) | (((value as u16) & 0x01) << 14);
        self.w
    }
}
#[doc = "Reader of field `EN`"]
pub type EN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `EN`"]
pub struct EN_W<'a> {
    w: &'a mut W,
}
impl<'a> EN_W<'a> {
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
    #[doc = "Bits 0:7 - Averaging Factor"]
    #[inline(always)]
    pub fn factor(&self) -> FACTOR_R {
        FACTOR_R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bit 14 - Enable Oversampling"]
    #[inline(always)]
    pub fn os(&self) -> OS_R {
        OS_R::new(((self.bits >> 14) & 0x01) != 0)
    }
    #[doc = "Bit 15 - Enable Averaging on Channels Enabled in Enable Register"]
    #[inline(always)]
    pub fn en(&self) -> EN_R {
        EN_R::new(((self.bits >> 15) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bits 0:7 - Averaging Factor"]
    #[inline(always)]
    pub fn factor(&mut self) -> FACTOR_W {
        FACTOR_W { w: self }
    }
    #[doc = "Bit 14 - Enable Oversampling"]
    #[inline(always)]
    pub fn os(&mut self) -> OS_W {
        OS_W { w: self }
    }
    #[doc = "Bit 15 - Enable Averaging on Channels Enabled in Enable Register"]
    #[inline(always)]
    pub fn en(&mut self) -> EN_W {
        EN_W { w: self }
    }
}
