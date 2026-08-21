#[doc = "Reader of register CTL3"]
pub type R = crate::R<u32, super::CTL3>;
#[doc = "Writer for register CTL3"]
pub type W = crate::W<u32, super::CTL3>;
#[doc = "Register CTL3 `reset()`'s with value 0x691a"]
impl crate::ResetValue for super::CTL3 {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x691a
    }
}
#[doc = "Reader of field `SPLLNSEL`"]
pub type SPLLNSEL_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `SPLLNSEL`"]
pub struct SPLLNSEL_W<'a> {
    w: &'a mut W,
}
impl<'a> SPLLNSEL_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x1f) | ((value as u32) & 0x1f);
        self.w
    }
}
#[doc = "Reader of field `SPLLDIV2`"]
pub type SPLLDIV2_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SPLLDIV2`"]
pub struct SPLLDIV2_W<'a> {
    w: &'a mut W,
}
impl<'a> SPLLDIV2_W<'a> {
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
#[doc = "Reader of field `SPLLEN`"]
pub type SPLLEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SPLLEN`"]
pub struct SPLLEN_W<'a> {
    w: &'a mut W,
}
impl<'a> SPLLEN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 9)) | (((value as u32) & 0x01) << 9);
        self.w
    }
}
#[doc = "Reader of field `SPLLIE`"]
pub type SPLLIE_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SPLLIE`"]
pub struct SPLLIE_W<'a> {
    w: &'a mut W,
}
impl<'a> SPLLIE_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 10)) | (((value as u32) & 0x01) << 10);
        self.w
    }
}
#[doc = "Reader of field `SPLLMSEL`"]
pub type SPLLMSEL_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `SPLLMSEL`"]
pub struct SPLLMSEL_W<'a> {
    w: &'a mut W,
}
impl<'a> SPLLMSEL_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x0f << 11)) | (((value as u32) & 0x0f) << 11);
        self.w
    }
}
#[doc = "Reader of field `SPLLMUL2`"]
pub type SPLLMUL2_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SPLLMUL2`"]
pub struct SPLLMUL2_W<'a> {
    w: &'a mut W,
}
impl<'a> SPLLMUL2_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 16)) | (((value as u32) & 0x01) << 16);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:4 - System PLL N Multiplier"]
    #[inline(always)]
    pub fn spllnsel(&self) -> SPLLNSEL_R {
        SPLLNSEL_R::new((self.bits & 0x1f) as u8)
    }
    #[doc = "Bit 8 - System PLL Division by 2"]
    #[inline(always)]
    pub fn splldiv2(&self) -> SPLLDIV2_R {
        SPLLDIV2_R::new(((self.bits >> 8) & 0x01) != 0)
    }
    #[doc = "Bit 9 - System PLL Enable"]
    #[inline(always)]
    pub fn spllen(&self) -> SPLLEN_R {
        SPLLEN_R::new(((self.bits >> 9) & 0x01) != 0)
    }
    #[doc = "Bit 10 - System PLL Interrupt Enable"]
    #[inline(always)]
    pub fn spllie(&self) -> SPLLIE_R {
        SPLLIE_R::new(((self.bits >> 10) & 0x01) != 0)
    }
    #[doc = "Bits 11:14 - System PLL M Divider"]
    #[inline(always)]
    pub fn spllmsel(&self) -> SPLLMSEL_R {
        SPLLMSEL_R::new(((self.bits >> 11) & 0x0f) as u8)
    }
    #[doc = "Bit 16 - System PLL Multiply by 2"]
    #[inline(always)]
    pub fn spllmul2(&self) -> SPLLMUL2_R {
        SPLLMUL2_R::new(((self.bits >> 16) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bits 0:4 - System PLL N Multiplier"]
    #[inline(always)]
    pub fn spllnsel(&mut self) -> SPLLNSEL_W {
        SPLLNSEL_W { w: self }
    }
    #[doc = "Bit 8 - System PLL Division by 2"]
    #[inline(always)]
    pub fn splldiv2(&mut self) -> SPLLDIV2_W {
        SPLLDIV2_W { w: self }
    }
    #[doc = "Bit 9 - System PLL Enable"]
    #[inline(always)]
    pub fn spllen(&mut self) -> SPLLEN_W {
        SPLLEN_W { w: self }
    }
    #[doc = "Bit 10 - System PLL Interrupt Enable"]
    #[inline(always)]
    pub fn spllie(&mut self) -> SPLLIE_W {
        SPLLIE_W { w: self }
    }
    #[doc = "Bits 11:14 - System PLL M Divider"]
    #[inline(always)]
    pub fn spllmsel(&mut self) -> SPLLMSEL_W {
        SPLLMSEL_W { w: self }
    }
    #[doc = "Bit 16 - System PLL Multiply by 2"]
    #[inline(always)]
    pub fn spllmul2(&mut self) -> SPLLMUL2_W {
        SPLLMUL2_W { w: self }
    }
}
