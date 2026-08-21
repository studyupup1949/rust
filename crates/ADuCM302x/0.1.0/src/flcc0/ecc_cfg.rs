#[doc = "Reader of register ECC_CFG"]
pub type R = crate::R<u32, super::ECC_CFG>;
#[doc = "Writer for register ECC_CFG"]
pub type W = crate::W<u32, super::ECC_CFG>;
#[doc = "Register ECC_CFG `reset()`'s with value 0x02"]
impl crate::ResetValue for super::ECC_CFG {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x02
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
        self.w.bits = (self.w.bits & !0x01) | ((value as u32) & 0x01);
        self.w
    }
}
#[doc = "Reader of field `INFOEN`"]
pub type INFOEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `INFOEN`"]
pub struct INFOEN_W<'a> {
    w: &'a mut W,
}
impl<'a> INFOEN_W<'a> {
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
#[doc = "Reader of field `PTR`"]
pub type PTR_R = crate::R<u32, u32>;
#[doc = "Write proxy for field `PTR`"]
pub struct PTR_W<'a> {
    w: &'a mut W,
}
impl<'a> PTR_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u32) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x00ff_ffff << 8)) | (((value as u32) & 0x00ff_ffff) << 8);
        self.w
    }
}
impl R {
    #[doc = "Bit 0 - ECC Enable"]
    #[inline(always)]
    pub fn en(&self) -> EN_R {
        EN_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Info Space ECC Enable Bit"]
    #[inline(always)]
    pub fn infoen(&self) -> INFOEN_R {
        INFOEN_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bits 8:31 - ECC Start Page Pointer"]
    #[inline(always)]
    pub fn ptr(&self) -> PTR_R {
        PTR_R::new(((self.bits >> 8) & 0x00ff_ffff) as u32)
    }
}
impl W {
    #[doc = "Bit 0 - ECC Enable"]
    #[inline(always)]
    pub fn en(&mut self) -> EN_W {
        EN_W { w: self }
    }
    #[doc = "Bit 1 - Info Space ECC Enable Bit"]
    #[inline(always)]
    pub fn infoen(&mut self) -> INFOEN_W {
        INFOEN_W { w: self }
    }
    #[doc = "Bits 8:31 - ECC Start Page Pointer"]
    #[inline(always)]
    pub fn ptr(&mut self) -> PTR_W {
        PTR_W { w: self }
    }
}
