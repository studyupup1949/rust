#[doc = "Reader of register CTL1"]
pub type R = crate::R<u32, super::CTL1>;
#[doc = "Writer for register CTL1"]
pub type W = crate::W<u32, super::CTL1>;
#[doc = "Register CTL1 `reset()`'s with value 0x0010_0404"]
impl crate::ResetValue for super::CTL1 {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x0010_0404
    }
}
#[doc = "Reader of field `HCLKDIVCNT`"]
pub type HCLKDIVCNT_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `HCLKDIVCNT`"]
pub struct HCLKDIVCNT_W<'a> {
    w: &'a mut W,
}
impl<'a> HCLKDIVCNT_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x3f) | ((value as u32) & 0x3f);
        self.w
    }
}
#[doc = "Reader of field `PCLKDIVCNT`"]
pub type PCLKDIVCNT_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `PCLKDIVCNT`"]
pub struct PCLKDIVCNT_W<'a> {
    w: &'a mut W,
}
impl<'a> PCLKDIVCNT_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x3f << 8)) | (((value as u32) & 0x3f) << 8);
        self.w
    }
}
#[doc = "Reader of field `ACLKDIVCNT`"]
pub type ACLKDIVCNT_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `ACLKDIVCNT`"]
pub struct ACLKDIVCNT_W<'a> {
    w: &'a mut W,
}
impl<'a> ACLKDIVCNT_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0xff << 16)) | (((value as u32) & 0xff) << 16);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:5 - HCLK Divide Count"]
    #[inline(always)]
    pub fn hclkdivcnt(&self) -> HCLKDIVCNT_R {
        HCLKDIVCNT_R::new((self.bits & 0x3f) as u8)
    }
    #[doc = "Bits 8:13 - PCLK Divide Count"]
    #[inline(always)]
    pub fn pclkdivcnt(&self) -> PCLKDIVCNT_R {
        PCLKDIVCNT_R::new(((self.bits >> 8) & 0x3f) as u8)
    }
    #[doc = "Bits 16:23 - ACLK Divide Count"]
    #[inline(always)]
    pub fn aclkdivcnt(&self) -> ACLKDIVCNT_R {
        ACLKDIVCNT_R::new(((self.bits >> 16) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:5 - HCLK Divide Count"]
    #[inline(always)]
    pub fn hclkdivcnt(&mut self) -> HCLKDIVCNT_W {
        HCLKDIVCNT_W { w: self }
    }
    #[doc = "Bits 8:13 - PCLK Divide Count"]
    #[inline(always)]
    pub fn pclkdivcnt(&mut self) -> PCLKDIVCNT_W {
        PCLKDIVCNT_W { w: self }
    }
    #[doc = "Bits 16:23 - ACLK Divide Count"]
    #[inline(always)]
    pub fn aclkdivcnt(&mut self) -> ACLKDIVCNT_W {
        ACLKDIVCNT_W { w: self }
    }
}
