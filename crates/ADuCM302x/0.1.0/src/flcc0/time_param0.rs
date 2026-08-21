#[doc = "Reader of register TIME_PARAM0"]
pub type R = crate::R<u32, super::TIME_PARAM0>;
#[doc = "Writer for register TIME_PARAM0"]
pub type W = crate::W<u32, super::TIME_PARAM0>;
#[doc = "Register TIME_PARAM0 `reset()`'s with value 0xb895_5950"]
impl crate::ResetValue for super::TIME_PARAM0 {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0xb895_5950
    }
}
#[doc = "Reader of field `DIVREFCLK`"]
pub type DIVREFCLK_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `DIVREFCLK`"]
pub struct DIVREFCLK_W<'a> {
    w: &'a mut W,
}
impl<'a> DIVREFCLK_W<'a> {
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
#[doc = "Reader of field `TNVS`"]
pub type TNVS_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `TNVS`"]
pub struct TNVS_W<'a> {
    w: &'a mut W,
}
impl<'a> TNVS_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x0f << 4)) | (((value as u32) & 0x0f) << 4);
        self.w
    }
}
#[doc = "Reader of field `TPGS`"]
pub type TPGS_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `TPGS`"]
pub struct TPGS_W<'a> {
    w: &'a mut W,
}
impl<'a> TPGS_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x0f << 8)) | (((value as u32) & 0x0f) << 8);
        self.w
    }
}
#[doc = "Reader of field `TPROG`"]
pub type TPROG_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `TPROG`"]
pub struct TPROG_W<'a> {
    w: &'a mut W,
}
impl<'a> TPROG_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x0f << 12)) | (((value as u32) & 0x0f) << 12);
        self.w
    }
}
#[doc = "Reader of field `TNVH`"]
pub type TNVH_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `TNVH`"]
pub struct TNVH_W<'a> {
    w: &'a mut W,
}
impl<'a> TNVH_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x0f << 16)) | (((value as u32) & 0x0f) << 16);
        self.w
    }
}
#[doc = "Reader of field `TRCV`"]
pub type TRCV_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `TRCV`"]
pub struct TRCV_W<'a> {
    w: &'a mut W,
}
impl<'a> TRCV_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x0f << 20)) | (((value as u32) & 0x0f) << 20);
        self.w
    }
}
#[doc = "Reader of field `TERASE`"]
pub type TERASE_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `TERASE`"]
pub struct TERASE_W<'a> {
    w: &'a mut W,
}
impl<'a> TERASE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x0f << 24)) | (((value as u32) & 0x0f) << 24);
        self.w
    }
}
#[doc = "Reader of field `TNVH1`"]
pub type TNVH1_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `TNVH1`"]
pub struct TNVH1_W<'a> {
    w: &'a mut W,
}
impl<'a> TNVH1_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x0f << 28)) | (((value as u32) & 0x0f) << 28);
        self.w
    }
}
impl R {
    #[doc = "Bit 0 - Divide Reference Clock (by 2)"]
    #[inline(always)]
    pub fn divrefclk(&self) -> DIVREFCLK_R {
        DIVREFCLK_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bits 4:7 - PROG/ERASE to NVSTR Setup Time"]
    #[inline(always)]
    pub fn tnvs(&self) -> TNVS_R {
        TNVS_R::new(((self.bits >> 4) & 0x0f) as u8)
    }
    #[doc = "Bits 8:11 - NVSTR to Program Setup Time"]
    #[inline(always)]
    pub fn tpgs(&self) -> TPGS_R {
        TPGS_R::new(((self.bits >> 8) & 0x0f) as u8)
    }
    #[doc = "Bits 12:15 - Program Time"]
    #[inline(always)]
    pub fn tprog(&self) -> TPROG_R {
        TPROG_R::new(((self.bits >> 12) & 0x0f) as u8)
    }
    #[doc = "Bits 16:19 - NVSTR Hold Time"]
    #[inline(always)]
    pub fn tnvh(&self) -> TNVH_R {
        TNVH_R::new(((self.bits >> 16) & 0x0f) as u8)
    }
    #[doc = "Bits 20:23 - Recovery Time"]
    #[inline(always)]
    pub fn trcv(&self) -> TRCV_R {
        TRCV_R::new(((self.bits >> 20) & 0x0f) as u8)
    }
    #[doc = "Bits 24:27 - Erase Time"]
    #[inline(always)]
    pub fn terase(&self) -> TERASE_R {
        TERASE_R::new(((self.bits >> 24) & 0x0f) as u8)
    }
    #[doc = "Bits 28:31 - NVSTR Hold Time During Mass Erase"]
    #[inline(always)]
    pub fn tnvh1(&self) -> TNVH1_R {
        TNVH1_R::new(((self.bits >> 28) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - Divide Reference Clock (by 2)"]
    #[inline(always)]
    pub fn divrefclk(&mut self) -> DIVREFCLK_W {
        DIVREFCLK_W { w: self }
    }
    #[doc = "Bits 4:7 - PROG/ERASE to NVSTR Setup Time"]
    #[inline(always)]
    pub fn tnvs(&mut self) -> TNVS_W {
        TNVS_W { w: self }
    }
    #[doc = "Bits 8:11 - NVSTR to Program Setup Time"]
    #[inline(always)]
    pub fn tpgs(&mut self) -> TPGS_W {
        TPGS_W { w: self }
    }
    #[doc = "Bits 12:15 - Program Time"]
    #[inline(always)]
    pub fn tprog(&mut self) -> TPROG_W {
        TPROG_W { w: self }
    }
    #[doc = "Bits 16:19 - NVSTR Hold Time"]
    #[inline(always)]
    pub fn tnvh(&mut self) -> TNVH_W {
        TNVH_W { w: self }
    }
    #[doc = "Bits 20:23 - Recovery Time"]
    #[inline(always)]
    pub fn trcv(&mut self) -> TRCV_W {
        TRCV_W { w: self }
    }
    #[doc = "Bits 24:27 - Erase Time"]
    #[inline(always)]
    pub fn terase(&mut self) -> TERASE_W {
        TERASE_W { w: self }
    }
    #[doc = "Bits 28:31 - NVSTR Hold Time During Mass Erase"]
    #[inline(always)]
    pub fn tnvh1(&mut self) -> TNVH1_W {
        TNVH1_W { w: self }
    }
}
