#[doc = "Reader of register CNV_TIME"]
pub type R = crate::R<u16, super::CNV_TIME>;
#[doc = "Writer for register CNV_TIME"]
pub type W = crate::W<u16, super::CNV_TIME>;
#[doc = "Register CNV_TIME `reset()`'s with value 0"]
impl crate::ResetValue for super::CNV_TIME {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `SAMPTIME`"]
pub type SAMPTIME_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `SAMPTIME`"]
pub struct SAMPTIME_W<'a> {
    w: &'a mut W,
}
impl<'a> SAMPTIME_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xff) | ((value as u16) & 0xff);
        self.w
    }
}
#[doc = "Reader of field `DLY`"]
pub type DLY_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `DLY`"]
pub struct DLY_W<'a> {
    w: &'a mut W,
}
impl<'a> DLY_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0xff << 8)) | (((value as u16) & 0xff) << 8);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:7 - Sampling Time"]
    #[inline(always)]
    pub fn samptime(&self) -> SAMPTIME_R {
        SAMPTIME_R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Delay Between Two Consecutive Conversions"]
    #[inline(always)]
    pub fn dly(&self) -> DLY_R {
        DLY_R::new(((self.bits >> 8) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Sampling Time"]
    #[inline(always)]
    pub fn samptime(&mut self) -> SAMPTIME_W {
        SAMPTIME_W { w: self }
    }
    #[doc = "Bits 8:15 - Delay Between Two Consecutive Conversions"]
    #[inline(always)]
    pub fn dly(&mut self) -> DLY_W {
        DLY_W { w: self }
    }
}
