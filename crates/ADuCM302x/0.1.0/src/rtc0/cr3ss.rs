#[doc = "Reader of register CR3SS"]
pub type R = crate::R<u16, super::CR3SS>;
#[doc = "Writer for register CR3SS"]
pub type W = crate::W<u16, super::CR3SS>;
#[doc = "Register CR3SS `reset()`'s with value 0"]
impl crate::ResetValue for super::CR3SS {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `SS1EN`"]
pub type SS1EN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SS1EN`"]
pub struct SS1EN_W<'a> {
    w: &'a mut W,
}
impl<'a> SS1EN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 1)) | (((value as u16) & 0x01) << 1);
        self.w
    }
}
#[doc = "Reader of field `SS1IRQEN`"]
pub type SS1IRQEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SS1IRQEN`"]
pub struct SS1IRQEN_W<'a> {
    w: &'a mut W,
}
impl<'a> SS1IRQEN_W<'a> {
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
    #[doc = "Bit 1 - Enable for SensorStrobe Channel 1"]
    #[inline(always)]
    pub fn ss1en(&self) -> SS1EN_R {
        SS1EN_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 9 - Interrupt Enable for SensorStrobe Channel 1"]
    #[inline(always)]
    pub fn ss1irqen(&self) -> SS1IRQEN_R {
        SS1IRQEN_R::new(((self.bits >> 9) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 1 - Enable for SensorStrobe Channel 1"]
    #[inline(always)]
    pub fn ss1en(&mut self) -> SS1EN_W {
        SS1EN_W { w: self }
    }
    #[doc = "Bit 9 - Interrupt Enable for SensorStrobe Channel 1"]
    #[inline(always)]
    pub fn ss1irqen(&mut self) -> SS1IRQEN_W {
        SS1IRQEN_W { w: self }
    }
}
