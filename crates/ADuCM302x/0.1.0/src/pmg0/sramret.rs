#[doc = "Reader of register SRAMRET"]
pub type R = crate::R<u32, super::SRAMRET>;
#[doc = "Writer for register SRAMRET"]
pub type W = crate::W<u32, super::SRAMRET>;
#[doc = "Register SRAMRET `reset()`'s with value 0"]
impl crate::ResetValue for super::SRAMRET {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `BNK1EN`"]
pub type BNK1EN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `BNK1EN`"]
pub struct BNK1EN_W<'a> {
    w: &'a mut W,
}
impl<'a> BNK1EN_W<'a> {
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
        self.w.bits = (self.w.bits & !0x01) | ((value as u32) & 0x01);
        self.w
    }
}
#[doc = "Reader of field `BNK2EN`"]
pub type BNK2EN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `BNK2EN`"]
pub struct BNK2EN_W<'a> {
    w: &'a mut W,
}
impl<'a> BNK2EN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 1)) | (((value as u32) & 0x01) << 1);
        self.w
    }
}
impl R {
    #[doc = "Bit 0 - Enable Retention Bank 1 (8kB)"]
    #[inline(always)]
    pub fn bnk1en(&self) -> BNK1EN_R {
        BNK1EN_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Enable Retention Bank 2 (16kB)"]
    #[inline(always)]
    pub fn bnk2en(&self) -> BNK2EN_R {
        BNK2EN_R::new(((self.bits >> 1) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Enable Retention Bank 1 (8kB)"]
    #[inline(always)]
    pub fn bnk1en(&mut self) -> BNK1EN_W {
        BNK1EN_W { w: self }
    }
    #[doc = "Bit 1 - Enable Retention Bank 2 (16kB)"]
    #[inline(always)]
    pub fn bnk2en(&mut self) -> BNK2EN_W {
        BNK2EN_W { w: self }
    }
}
