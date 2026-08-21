#[doc = "Reader of register CTL0"]
pub type R = crate::R<u32, super::CTL0>;
#[doc = "Writer for register CTL0"]
pub type W = crate::W<u32, super::CTL0>;
#[doc = "Register CTL0 `reset()`'s with value 0x78"]
impl crate::ResetValue for super::CTL0 {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x78
    }
}
#[doc = "Reader of field `CLKMUX`"]
pub type CLKMUX_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `CLKMUX`"]
pub struct CLKMUX_W<'a> {
    w: &'a mut W,
}
impl<'a> CLKMUX_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x03) | ((value as u32) & 0x03);
        self.w
    }
}
#[doc = "Reader of field `CLKOUT`"]
pub type CLKOUT_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `CLKOUT`"]
pub struct CLKOUT_W<'a> {
    w: &'a mut W,
}
impl<'a> CLKOUT_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x0f << 3)) | (((value as u32) & 0x0f) << 3);
        self.w
    }
}
#[doc = "Reader of field `RCLKMUX`"]
pub type RCLKMUX_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `RCLKMUX`"]
pub struct RCLKMUX_W<'a> {
    w: &'a mut W,
}
impl<'a> RCLKMUX_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 8)) | (((value as u32) & 0x03) << 8);
        self.w
    }
}
#[doc = "Reader of field `SPLLIPSEL`"]
pub type SPLLIPSEL_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SPLLIPSEL`"]
pub struct SPLLIPSEL_W<'a> {
    w: &'a mut W,
}
impl<'a> SPLLIPSEL_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 11)) | (((value as u32) & 0x01) << 11);
        self.w
    }
}
#[doc = "Reader of field `LFXTALIE`"]
pub type LFXTALIE_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `LFXTALIE`"]
pub struct LFXTALIE_W<'a> {
    w: &'a mut W,
}
impl<'a> LFXTALIE_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 14)) | (((value as u32) & 0x01) << 14);
        self.w
    }
}
#[doc = "Reader of field `HFXTALIE`"]
pub type HFXTALIE_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `HFXTALIE`"]
pub struct HFXTALIE_W<'a> {
    w: &'a mut W,
}
impl<'a> HFXTALIE_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 15)) | (((value as u32) & 0x01) << 15);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:1 - Clock Mux Select"]
    #[inline(always)]
    pub fn clkmux(&self) -> CLKMUX_R {
        CLKMUX_R::new((self.bits & 0x03) as u8)
    }
    #[doc = "Bits 3:6 - GPIO Clock Out Select (for Debug)"]
    #[inline(always)]
    pub fn clkout(&self) -> CLKOUT_R {
        CLKOUT_R::new(((self.bits >> 3) & 0x0f) as u8)
    }
    #[doc = "Bits 8:9 - Flash Reference Clock and HP Buck Source Mux"]
    #[inline(always)]
    pub fn rclkmux(&self) -> RCLKMUX_R {
        RCLKMUX_R::new(((self.bits >> 8) & 0x03) as u8)
    }
    #[doc = "Bit 11 - SPLL Source Select Mux"]
    #[inline(always)]
    pub fn spllipsel(&self) -> SPLLIPSEL_R {
        SPLLIPSEL_R::new(((self.bits >> 11) & 0x01) != 0)
    }
    #[doc = "Bit 14 - Low Frequency Crystal Interrupt Enable"]
    #[inline(always)]
    pub fn lfxtalie(&self) -> LFXTALIE_R {
        LFXTALIE_R::new(((self.bits >> 14) & 0x01) != 0)
    }
    #[doc = "Bit 15 - High Frequency Crystal Interrupt Enable"]
    #[inline(always)]
    pub fn hfxtalie(&self) -> HFXTALIE_R {
        HFXTALIE_R::new(((self.bits >> 15) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bits 0:1 - Clock Mux Select"]
    #[inline(always)]
    pub fn clkmux(&mut self) -> CLKMUX_W {
        CLKMUX_W { w: self }
    }
    #[doc = "Bits 3:6 - GPIO Clock Out Select (for Debug)"]
    #[inline(always)]
    pub fn clkout(&mut self) -> CLKOUT_W {
        CLKOUT_W { w: self }
    }
    #[doc = "Bits 8:9 - Flash Reference Clock and HP Buck Source Mux"]
    #[inline(always)]
    pub fn rclkmux(&mut self) -> RCLKMUX_W {
        RCLKMUX_W { w: self }
    }
    #[doc = "Bit 11 - SPLL Source Select Mux"]
    #[inline(always)]
    pub fn spllipsel(&mut self) -> SPLLIPSEL_W {
        SPLLIPSEL_W { w: self }
    }
    #[doc = "Bit 14 - Low Frequency Crystal Interrupt Enable"]
    #[inline(always)]
    pub fn lfxtalie(&mut self) -> LFXTALIE_W {
        LFXTALIE_W { w: self }
    }
    #[doc = "Bit 15 - High Frequency Crystal Interrupt Enable"]
    #[inline(always)]
    pub fn hfxtalie(&mut self) -> HFXTALIE_W {
        HFXTALIE_W { w: self }
    }
}
