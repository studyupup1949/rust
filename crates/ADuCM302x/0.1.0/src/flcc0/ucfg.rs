#[doc = "Reader of register UCFG"]
pub type R = crate::R<u32, super::UCFG>;
#[doc = "Writer for register UCFG"]
pub type W = crate::W<u32, super::UCFG>;
#[doc = "Register UCFG `reset()`'s with value 0"]
impl crate::ResetValue for super::UCFG {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `KHDMAEN`"]
pub type KHDMAEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `KHDMAEN`"]
pub struct KHDMAEN_W<'a> {
    w: &'a mut W,
}
impl<'a> KHDMAEN_W<'a> {
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
#[doc = "Reader of field `AUTOINCEN`"]
pub type AUTOINCEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `AUTOINCEN`"]
pub struct AUTOINCEN_W<'a> {
    w: &'a mut W,
}
impl<'a> AUTOINCEN_W<'a> {
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
    #[doc = "Bit 0 - Key Hole DMA Enable"]
    #[inline(always)]
    pub fn khdmaen(&self) -> KHDMAEN_R {
        KHDMAEN_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Auto Address Increment for Key Hole Access"]
    #[inline(always)]
    pub fn autoincen(&self) -> AUTOINCEN_R {
        AUTOINCEN_R::new(((self.bits >> 1) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Key Hole DMA Enable"]
    #[inline(always)]
    pub fn khdmaen(&mut self) -> KHDMAEN_W {
        KHDMAEN_W { w: self }
    }
    #[doc = "Bit 1 - Auto Address Increment for Key Hole Access"]
    #[inline(always)]
    pub fn autoincen(&mut self) -> AUTOINCEN_W {
        AUTOINCEN_W { w: self }
    }
}
