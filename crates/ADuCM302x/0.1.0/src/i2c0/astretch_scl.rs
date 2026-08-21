#[doc = "Reader of register ASTRETCH_SCL"]
pub type R = crate::R<u16, super::ASTRETCH_SCL>;
#[doc = "Writer for register ASTRETCH_SCL"]
pub type W = crate::W<u16, super::ASTRETCH_SCL>;
#[doc = "Register ASTRETCH_SCL `reset()`'s with value 0"]
impl crate::ResetValue for super::ASTRETCH_SCL {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `MST`"]
pub type MST_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `MST`"]
pub struct MST_W<'a> {
    w: &'a mut W,
}
impl<'a> MST_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x0f) | ((value as u16) & 0x0f);
        self.w
    }
}
#[doc = "Reader of field `SLV`"]
pub type SLV_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `SLV`"]
pub struct SLV_W<'a> {
    w: &'a mut W,
}
impl<'a> SLV_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x0f << 4)) | (((value as u16) & 0x0f) << 4);
        self.w
    }
}
#[doc = "Reader of field `MSTTMO`"]
pub type MSTTMO_R = crate::R<bool, bool>;
#[doc = "Reader of field `SLVTMO`"]
pub type SLVTMO_R = crate::R<bool, bool>;
impl R {
    #[doc = "Bits 0:3 - Master Automatic Stretch Mode"]
    #[inline(always)]
    pub fn mst(&self) -> MST_R {
        MST_R::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bits 4:7 - Slave Automatic Stretch Mode"]
    #[inline(always)]
    pub fn slv(&self) -> SLV_R {
        SLV_R::new(((self.bits >> 4) & 0x0f) as u8)
    }
    #[doc = "Bit 8 - Master Automatic Stretch Timeout"]
    #[inline(always)]
    pub fn msttmo(&self) -> MSTTMO_R {
        MSTTMO_R::new(((self.bits >> 8) & 0x01) != 0)
    }
    #[doc = "Bit 9 - Slave Automatic Stretch Timeout"]
    #[inline(always)]
    pub fn slvtmo(&self) -> SLVTMO_R {
        SLVTMO_R::new(((self.bits >> 9) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bits 0:3 - Master Automatic Stretch Mode"]
    #[inline(always)]
    pub fn mst(&mut self) -> MST_W {
        MST_W { w: self }
    }
    #[doc = "Bits 4:7 - Slave Automatic Stretch Mode"]
    #[inline(always)]
    pub fn slv(&mut self) -> SLV_W {
        SLV_W { w: self }
    }
}
