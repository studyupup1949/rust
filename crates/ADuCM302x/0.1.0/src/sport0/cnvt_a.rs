#[doc = "Reader of register CNVT_A"]
pub type R = crate::R<u32, super::CNVT_A>;
#[doc = "Writer for register CNVT_A"]
pub type W = crate::W<u32, super::CNVT_A>;
#[doc = "Register CNVT_A `reset()`'s with value 0"]
impl crate::ResetValue for super::CNVT_A {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `WID`"]
pub type WID_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `WID`"]
pub struct WID_W<'a> {
    w: &'a mut W,
}
impl<'a> WID_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x0f) | ((value as u32) & 0x0f);
        self.w
    }
}
#[doc = "Reader of field `POL`"]
pub type POL_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `POL`"]
pub struct POL_W<'a> {
    w: &'a mut W,
}
impl<'a> POL_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 8)) | (((value as u32) & 0x01) << 8);
        self.w
    }
}
#[doc = "Reader of field `CNVT2FS`"]
pub type CNVT2FS_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `CNVT2FS`"]
pub struct CNVT2FS_W<'a> {
    w: &'a mut W,
}
impl<'a> CNVT2FS_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0xff << 16)) | (((value as u32) & 0xff) << 16);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:3 - SPT_CNVT Signal Width: Half SPORT a"]
    #[inline(always)]
    pub fn wid(&self) -> WID_R {
        WID_R::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bit 8 - Polarity of the SPT_CNVT Signal"]
    #[inline(always)]
    pub fn pol(&self) -> POL_R {
        POL_R::new(((self.bits >> 8) & 0x01) != 0)
    }
    #[doc = "Bits 16:23 - SPT_CNVT to FS Duration: Half SPORT a"]
    #[inline(always)]
    pub fn cnvt2fs(&self) -> CNVT2FS_R {
        CNVT2FS_R::new(((self.bits >> 16) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - SPT_CNVT Signal Width: Half SPORT a"]
    #[inline(always)]
    pub fn wid(&mut self) -> WID_W {
        WID_W { w: self }
    }
    #[doc = "Bit 8 - Polarity of the SPT_CNVT Signal"]
    #[inline(always)]
    pub fn pol(&mut self) -> POL_W {
        POL_W { w: self }
    }
    #[doc = "Bits 16:23 - SPT_CNVT to FS Duration: Half SPORT a"]
    #[inline(always)]
    pub fn cnvt2fs(&mut self) -> CNVT2FS_W {
        CNVT2FS_W { w: self }
    }
}
