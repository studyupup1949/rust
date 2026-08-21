#[doc = "Reader of register SSMSK"]
pub type R = crate::R<u16, super::SSMSK>;
#[doc = "Writer for register SSMSK"]
pub type W = crate::W<u16, super::SSMSK>;
#[doc = "Register SSMSK `reset()`'s with value 0"]
impl crate::ResetValue for super::SSMSK {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `SSMSK`"]
pub type SSMSK_R = crate::R<u16, u16>;
#[doc = "Write proxy for field `SSMSK`"]
pub struct SSMSK_W<'a> {
    w: &'a mut W,
}
impl<'a> SSMSK_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u16) -> &'a mut W {
        self.w.bits = (self.w.bits & !0xffff) | ((value as u16) & 0xffff);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:15 - Thermometer-Encoded Masks for SensorStrobe Channels"]
    #[inline(always)]
    pub fn ssmsk(&self) -> SSMSK_R {
        SSMSK_R::new((self.bits & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:15 - Thermometer-Encoded Masks for SensorStrobe Channels"]
    #[inline(always)]
    pub fn ssmsk(&mut self) -> SSMSK_W {
        SSMSK_W { w: self }
    }
}
