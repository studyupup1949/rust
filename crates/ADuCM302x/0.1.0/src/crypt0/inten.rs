#[doc = "Reader of register INTEN"]
pub type R = crate::R<u32, super::INTEN>;
#[doc = "Writer for register INTEN"]
pub type W = crate::W<u32, super::INTEN>;
#[doc = "Register INTEN `reset()`'s with value 0"]
impl crate::ResetValue for super::INTEN {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `INRDYEN`"]
pub type INRDYEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `INRDYEN`"]
pub struct INRDYEN_W<'a> {
    w: &'a mut W,
}
impl<'a> INRDYEN_W<'a> {
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
#[doc = "Reader of field `OUTRDYEN`"]
pub type OUTRDYEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `OUTRDYEN`"]
pub struct OUTRDYEN_W<'a> {
    w: &'a mut W,
}
impl<'a> OUTRDYEN_W<'a> {
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
#[doc = "Reader of field `INOVREN`"]
pub type INOVREN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `INOVREN`"]
pub struct INOVREN_W<'a> {
    w: &'a mut W,
}
impl<'a> INOVREN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 2)) | (((value as u32) & 0x01) << 2);
        self.w
    }
}
#[doc = "Reader of field `SHADONEN`"]
pub type SHADONEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SHADONEN`"]
pub struct SHADONEN_W<'a> {
    w: &'a mut W,
}
impl<'a> SHADONEN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 5)) | (((value as u32) & 0x01) << 5);
        self.w
    }
}
impl R {
    #[doc = "Bit 0 - Enable Input Ready Interrupt"]
    #[inline(always)]
    pub fn inrdyen(&self) -> INRDYEN_R {
        INRDYEN_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Enables the Output Ready Interrupt"]
    #[inline(always)]
    pub fn outrdyen(&self) -> OUTRDYEN_R {
        OUTRDYEN_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Enable Input Overflow Interrupt"]
    #[inline(always)]
    pub fn inovren(&self) -> INOVREN_R {
        INOVREN_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 5 - Enable SHA_Done Interrupt"]
    #[inline(always)]
    pub fn shadonen(&self) -> SHADONEN_R {
        SHADONEN_R::new(((self.bits >> 5) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Enable Input Ready Interrupt"]
    #[inline(always)]
    pub fn inrdyen(&mut self) -> INRDYEN_W {
        INRDYEN_W { w: self }
    }
    #[doc = "Bit 1 - Enables the Output Ready Interrupt"]
    #[inline(always)]
    pub fn outrdyen(&mut self) -> OUTRDYEN_W {
        OUTRDYEN_W { w: self }
    }
    #[doc = "Bit 2 - Enable Input Overflow Interrupt"]
    #[inline(always)]
    pub fn inovren(&mut self) -> INOVREN_W {
        INOVREN_W { w: self }
    }
    #[doc = "Bit 5 - Enable SHA_Done Interrupt"]
    #[inline(always)]
    pub fn shadonen(&mut self) -> SHADONEN_W {
        SHADONEN_W { w: self }
    }
}
