#[doc = "Reader of register IRQ_EN"]
pub type R = crate::R<u16, super::IRQ_EN>;
#[doc = "Writer for register IRQ_EN"]
pub type W = crate::W<u16, super::IRQ_EN>;
#[doc = "Register IRQ_EN `reset()`'s with value 0"]
impl crate::ResetValue for super::IRQ_EN {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `CNVDONE`"]
pub type CNVDONE_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `CNVDONE`"]
pub struct CNVDONE_W<'a> {
    w: &'a mut W,
}
impl<'a> CNVDONE_W<'a> {
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
        self.w.bits = (self.w.bits & !0x01) | ((value as u16) & 0x01);
        self.w
    }
}
#[doc = "Reader of field `CALDONE`"]
pub type CALDONE_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `CALDONE`"]
pub struct CALDONE_W<'a> {
    w: &'a mut W,
}
impl<'a> CALDONE_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 10)) | (((value as u16) & 0x01) << 10);
        self.w
    }
}
#[doc = "Reader of field `OVF`"]
pub type OVF_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `OVF`"]
pub struct OVF_W<'a> {
    w: &'a mut W,
}
impl<'a> OVF_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 11)) | (((value as u16) & 0x01) << 11);
        self.w
    }
}
#[doc = "Reader of field `ALERT`"]
pub type ALERT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ALERT`"]
pub struct ALERT_W<'a> {
    w: &'a mut W,
}
impl<'a> ALERT_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 12)) | (((value as u16) & 0x01) << 12);
        self.w
    }
}
#[doc = "Reader of field `RDY`"]
pub type RDY_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `RDY`"]
pub struct RDY_W<'a> {
    w: &'a mut W,
}
impl<'a> RDY_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 13)) | (((value as u16) & 0x01) << 13);
        self.w
    }
}
impl R {
    #[doc = "Bit 0 - Enable Conversion Done Interrupt"]
    #[inline(always)]
    pub fn cnvdone(&self) -> CNVDONE_R {
        CNVDONE_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 10 - Enable Interrupt for Calibration Done"]
    #[inline(always)]
    pub fn caldone(&self) -> CALDONE_R {
        CALDONE_R::new(((self.bits >> 10) & 0x01) != 0)
    }
    #[doc = "Bit 11 - Enable Overflow Interrupt"]
    #[inline(always)]
    pub fn ovf(&self) -> OVF_R {
        OVF_R::new(((self.bits >> 11) & 0x01) != 0)
    }
    #[doc = "Bit 12 - Interrupt on Crossing Lower or Higher Limit Enable"]
    #[inline(always)]
    pub fn alert(&self) -> ALERT_R {
        ALERT_R::new(((self.bits >> 12) & 0x01) != 0)
    }
    #[doc = "Bit 13 - Set to Enable Interrupt When ADC is Ready to Convert"]
    #[inline(always)]
    pub fn rdy(&self) -> RDY_R {
        RDY_R::new(((self.bits >> 13) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Enable Conversion Done Interrupt"]
    #[inline(always)]
    pub fn cnvdone(&mut self) -> CNVDONE_W {
        CNVDONE_W { w: self }
    }
    #[doc = "Bit 10 - Enable Interrupt for Calibration Done"]
    #[inline(always)]
    pub fn caldone(&mut self) -> CALDONE_W {
        CALDONE_W { w: self }
    }
    #[doc = "Bit 11 - Enable Overflow Interrupt"]
    #[inline(always)]
    pub fn ovf(&mut self) -> OVF_W {
        OVF_W { w: self }
    }
    #[doc = "Bit 12 - Interrupt on Crossing Lower or Higher Limit Enable"]
    #[inline(always)]
    pub fn alert(&mut self) -> ALERT_W {
        ALERT_W { w: self }
    }
    #[doc = "Bit 13 - Set to Enable Interrupt When ADC is Ready to Convert"]
    #[inline(always)]
    pub fn rdy(&mut self) -> RDY_W {
        RDY_W { w: self }
    }
}
