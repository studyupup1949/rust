#[doc = "Reader of register SR3"]
pub type R = crate::R<u16, super::SR3>;
#[doc = "Writer for register SR3"]
pub type W = crate::W<u16, super::SR3>;
#[doc = "Register SR3 `reset()`'s with value 0"]
impl crate::ResetValue for super::SR3 {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `IC0IRQ`"]
pub type IC0IRQ_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IC0IRQ`"]
pub struct IC0IRQ_W<'a> {
    w: &'a mut W,
}
impl<'a> IC0IRQ_W<'a> {
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
#[doc = "Reader of field `IC2IRQ`"]
pub type IC2IRQ_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IC2IRQ`"]
pub struct IC2IRQ_W<'a> {
    w: &'a mut W,
}
impl<'a> IC2IRQ_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 2)) | (((value as u16) & 0x01) << 2);
        self.w
    }
}
#[doc = "Reader of field `IC3IRQ`"]
pub type IC3IRQ_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IC3IRQ`"]
pub struct IC3IRQ_W<'a> {
    w: &'a mut W,
}
impl<'a> IC3IRQ_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 3)) | (((value as u16) & 0x01) << 3);
        self.w
    }
}
#[doc = "Reader of field `IC4IRQ`"]
pub type IC4IRQ_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IC4IRQ`"]
pub struct IC4IRQ_W<'a> {
    w: &'a mut W,
}
impl<'a> IC4IRQ_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 4)) | (((value as u16) & 0x01) << 4);
        self.w
    }
}
#[doc = "Reader of field `ALMINTMIR`"]
pub type ALMINTMIR_R = crate::R<bool, bool>;
#[doc = "Reader of field `SS1IRQ`"]
pub type SS1IRQ_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SS1IRQ`"]
pub struct SS1IRQ_W<'a> {
    w: &'a mut W,
}
impl<'a> SS1IRQ_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 9)) | (((value as u16) & 0x01) << 9);
        self.w
    }
}
impl R {
    #[doc = "Bit 0 - Sticky Interrupt Source for the RTC Input Capture Channel 0"]
    #[inline(always)]
    pub fn ic0irq(&self) -> IC0IRQ_R {
        IC0IRQ_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 2 - Sticky Interrupt Source for the RTC Input Capture Channel 2"]
    #[inline(always)]
    pub fn ic2irq(&self) -> IC2IRQ_R {
        IC2IRQ_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Sticky Interrupt Source for the RTC Input Capture Channel 3"]
    #[inline(always)]
    pub fn ic3irq(&self) -> IC3IRQ_R {
        IC3IRQ_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Sticky Interrupt Source for the RTC Input Capture Channel 4"]
    #[inline(always)]
    pub fn ic4irq(&self) -> IC4IRQ_R {
        IC4IRQ_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bit 8 - Read-only Mirror of the ALMINT Interrupt Source in SR0 Register"]
    #[inline(always)]
    pub fn almintmir(&self) -> ALMINTMIR_R {
        ALMINTMIR_R::new(((self.bits >> 8) & 0x01) != 0)
    }
    #[doc = "Bit 9 - Sticky Interrupt Source for SensorStrobe Channel 1"]
    #[inline(always)]
    pub fn ss1irq(&self) -> SS1IRQ_R {
        SS1IRQ_R::new(((self.bits >> 9) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Sticky Interrupt Source for the RTC Input Capture Channel 0"]
    #[inline(always)]
    pub fn ic0irq(&mut self) -> IC0IRQ_W {
        IC0IRQ_W { w: self }
    }
    #[doc = "Bit 2 - Sticky Interrupt Source for the RTC Input Capture Channel 2"]
    #[inline(always)]
    pub fn ic2irq(&mut self) -> IC2IRQ_W {
        IC2IRQ_W { w: self }
    }
    #[doc = "Bit 3 - Sticky Interrupt Source for the RTC Input Capture Channel 3"]
    #[inline(always)]
    pub fn ic3irq(&mut self) -> IC3IRQ_W {
        IC3IRQ_W { w: self }
    }
    #[doc = "Bit 4 - Sticky Interrupt Source for the RTC Input Capture Channel 4"]
    #[inline(always)]
    pub fn ic4irq(&mut self) -> IC4IRQ_W {
        IC4IRQ_W { w: self }
    }
    #[doc = "Bit 9 - Sticky Interrupt Source for SensorStrobe Channel 1"]
    #[inline(always)]
    pub fn ss1irq(&mut self) -> SS1IRQ_W {
        SS1IRQ_W { w: self }
    }
}
