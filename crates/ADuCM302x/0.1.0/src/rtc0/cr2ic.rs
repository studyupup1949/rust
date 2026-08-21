#[doc = "Reader of register CR2IC"]
pub type R = crate::R<u16, super::CR2IC>;
#[doc = "Writer for register CR2IC"]
pub type W = crate::W<u16, super::CR2IC>;
#[doc = "Register CR2IC `reset()`'s with value 0x83a0"]
impl crate::ResetValue for super::CR2IC {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x83a0
    }
}
#[doc = "Reader of field `IC0EN`"]
pub type IC0EN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IC0EN`"]
pub struct IC0EN_W<'a> {
    w: &'a mut W,
}
impl<'a> IC0EN_W<'a> {
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
#[doc = "Reader of field `IC2EN`"]
pub type IC2EN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IC2EN`"]
pub struct IC2EN_W<'a> {
    w: &'a mut W,
}
impl<'a> IC2EN_W<'a> {
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
#[doc = "Reader of field `IC3EN`"]
pub type IC3EN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IC3EN`"]
pub struct IC3EN_W<'a> {
    w: &'a mut W,
}
impl<'a> IC3EN_W<'a> {
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
#[doc = "Reader of field `IC4EN`"]
pub type IC4EN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IC4EN`"]
pub struct IC4EN_W<'a> {
    w: &'a mut W,
}
impl<'a> IC4EN_W<'a> {
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
#[doc = "Reader of field `IC0LH`"]
pub type IC0LH_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IC0LH`"]
pub struct IC0LH_W<'a> {
    w: &'a mut W,
}
impl<'a> IC0LH_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 5)) | (((value as u16) & 0x01) << 5);
        self.w
    }
}
#[doc = "Reader of field `IC2LH`"]
pub type IC2LH_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IC2LH`"]
pub struct IC2LH_W<'a> {
    w: &'a mut W,
}
impl<'a> IC2LH_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 7)) | (((value as u16) & 0x01) << 7);
        self.w
    }
}
#[doc = "Reader of field `IC3LH`"]
pub type IC3LH_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IC3LH`"]
pub struct IC3LH_W<'a> {
    w: &'a mut W,
}
impl<'a> IC3LH_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 8)) | (((value as u16) & 0x01) << 8);
        self.w
    }
}
#[doc = "Reader of field `IC4LH`"]
pub type IC4LH_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IC4LH`"]
pub struct IC4LH_W<'a> {
    w: &'a mut W,
}
impl<'a> IC4LH_W<'a> {
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
#[doc = "Reader of field `IC0IRQEN`"]
pub type IC0IRQEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IC0IRQEN`"]
pub struct IC0IRQEN_W<'a> {
    w: &'a mut W,
}
impl<'a> IC0IRQEN_W<'a> {
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
#[doc = "Reader of field `IC2IRQEN`"]
pub type IC2IRQEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IC2IRQEN`"]
pub struct IC2IRQEN_W<'a> {
    w: &'a mut W,
}
impl<'a> IC2IRQEN_W<'a> {
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
#[doc = "Reader of field `IC3IRQEN`"]
pub type IC3IRQEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IC3IRQEN`"]
pub struct IC3IRQEN_W<'a> {
    w: &'a mut W,
}
impl<'a> IC3IRQEN_W<'a> {
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
#[doc = "Reader of field `IC4IRQEN`"]
pub type IC4IRQEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IC4IRQEN`"]
pub struct IC4IRQEN_W<'a> {
    w: &'a mut W,
}
impl<'a> IC4IRQEN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 14)) | (((value as u16) & 0x01) << 14);
        self.w
    }
}
#[doc = "Reader of field `ICOWUSEN`"]
pub type ICOWUSEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ICOWUSEN`"]
pub struct ICOWUSEN_W<'a> {
    w: &'a mut W,
}
impl<'a> ICOWUSEN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 15)) | (((value as u16) & 0x01) << 15);
        self.w
    }
}
impl R {
    #[doc = "Bit 0 - Enable for the RTC Input Capture Channel 0"]
    #[inline(always)]
    pub fn ic0en(&self) -> IC0EN_R {
        IC0EN_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 2 - Enable for the RTC Input Capture Channel 2"]
    #[inline(always)]
    pub fn ic2en(&self) -> IC2EN_R {
        IC2EN_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Enable for the RTC Input Capture Channel 3"]
    #[inline(always)]
    pub fn ic3en(&self) -> IC3EN_R {
        IC3EN_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Enable for the RTC Input Capture Channel 4"]
    #[inline(always)]
    pub fn ic4en(&self) -> IC4EN_R {
        IC4EN_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bit 5 - Polarity of the Active-Going Capture Edge for the RTC Input Capture Channel 0"]
    #[inline(always)]
    pub fn ic0lh(&self) -> IC0LH_R {
        IC0LH_R::new(((self.bits >> 5) & 0x01) != 0)
    }
    #[doc = "Bit 7 - Polarity of the Active-going Capture Edge for the Input Capture Channel 2"]
    #[inline(always)]
    pub fn ic2lh(&self) -> IC2LH_R {
        IC2LH_R::new(((self.bits >> 7) & 0x01) != 0)
    }
    #[doc = "Bit 8 - Polarity of the Active-going Capture Edge for the Input Capture Channel 3"]
    #[inline(always)]
    pub fn ic3lh(&self) -> IC3LH_R {
        IC3LH_R::new(((self.bits >> 8) & 0x01) != 0)
    }
    #[doc = "Bit 9 - Polarity of the Active-going Capture Edge for the Input Capture Channel 4"]
    #[inline(always)]
    pub fn ic4lh(&self) -> IC4LH_R {
        IC4LH_R::new(((self.bits >> 9) & 0x01) != 0)
    }
    #[doc = "Bit 10 - Interrupt Enable for the RTC Input Capture Channel 0"]
    #[inline(always)]
    pub fn ic0irqen(&self) -> IC0IRQEN_R {
        IC0IRQEN_R::new(((self.bits >> 10) & 0x01) != 0)
    }
    #[doc = "Bit 12 - Interrupt Enable for the RTC Input Capture Channel 2"]
    #[inline(always)]
    pub fn ic2irqen(&self) -> IC2IRQEN_R {
        IC2IRQEN_R::new(((self.bits >> 12) & 0x01) != 0)
    }
    #[doc = "Bit 13 - Interrupt Enable for the RTC Input Capture Channel 3"]
    #[inline(always)]
    pub fn ic3irqen(&self) -> IC3IRQEN_R {
        IC3IRQEN_R::new(((self.bits >> 13) & 0x01) != 0)
    }
    #[doc = "Bit 14 - Interrupt Enable for the RTC Input Capture Channel 4"]
    #[inline(always)]
    pub fn ic4irqen(&self) -> IC4IRQEN_R {
        IC4IRQEN_R::new(((self.bits >> 14) & 0x01) != 0)
    }
    #[doc = "Bit 15 - Enable Overwrite of Unread Snapshots for All Input Capture Channels"]
    #[inline(always)]
    pub fn icowusen(&self) -> ICOWUSEN_R {
        ICOWUSEN_R::new(((self.bits >> 15) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Enable for the RTC Input Capture Channel 0"]
    #[inline(always)]
    pub fn ic0en(&mut self) -> IC0EN_W {
        IC0EN_W { w: self }
    }
    #[doc = "Bit 2 - Enable for the RTC Input Capture Channel 2"]
    #[inline(always)]
    pub fn ic2en(&mut self) -> IC2EN_W {
        IC2EN_W { w: self }
    }
    #[doc = "Bit 3 - Enable for the RTC Input Capture Channel 3"]
    #[inline(always)]
    pub fn ic3en(&mut self) -> IC3EN_W {
        IC3EN_W { w: self }
    }
    #[doc = "Bit 4 - Enable for the RTC Input Capture Channel 4"]
    #[inline(always)]
    pub fn ic4en(&mut self) -> IC4EN_W {
        IC4EN_W { w: self }
    }
    #[doc = "Bit 5 - Polarity of the Active-Going Capture Edge for the RTC Input Capture Channel 0"]
    #[inline(always)]
    pub fn ic0lh(&mut self) -> IC0LH_W {
        IC0LH_W { w: self }
    }
    #[doc = "Bit 7 - Polarity of the Active-going Capture Edge for the Input Capture Channel 2"]
    #[inline(always)]
    pub fn ic2lh(&mut self) -> IC2LH_W {
        IC2LH_W { w: self }
    }
    #[doc = "Bit 8 - Polarity of the Active-going Capture Edge for the Input Capture Channel 3"]
    #[inline(always)]
    pub fn ic3lh(&mut self) -> IC3LH_W {
        IC3LH_W { w: self }
    }
    #[doc = "Bit 9 - Polarity of the Active-going Capture Edge for the Input Capture Channel 4"]
    #[inline(always)]
    pub fn ic4lh(&mut self) -> IC4LH_W {
        IC4LH_W { w: self }
    }
    #[doc = "Bit 10 - Interrupt Enable for the RTC Input Capture Channel 0"]
    #[inline(always)]
    pub fn ic0irqen(&mut self) -> IC0IRQEN_W {
        IC0IRQEN_W { w: self }
    }
    #[doc = "Bit 12 - Interrupt Enable for the RTC Input Capture Channel 2"]
    #[inline(always)]
    pub fn ic2irqen(&mut self) -> IC2IRQEN_W {
        IC2IRQEN_W { w: self }
    }
    #[doc = "Bit 13 - Interrupt Enable for the RTC Input Capture Channel 3"]
    #[inline(always)]
    pub fn ic3irqen(&mut self) -> IC3IRQEN_W {
        IC3IRQEN_W { w: self }
    }
    #[doc = "Bit 14 - Interrupt Enable for the RTC Input Capture Channel 4"]
    #[inline(always)]
    pub fn ic4irqen(&mut self) -> IC4IRQEN_W {
        IC4IRQEN_W { w: self }
    }
    #[doc = "Bit 15 - Enable Overwrite of Unread Snapshots for All Input Capture Channels"]
    #[inline(always)]
    pub fn icowusen(&mut self) -> ICOWUSEN_W {
        ICOWUSEN_W { w: self }
    }
}
