#[doc = "Writer for register CLRINT"]
pub type W = crate::W<u16, super::CLRINT>;
#[doc = "Register CLRINT `reset()`'s with value 0"]
impl crate::ResetValue for super::CLRINT {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Write proxy for field `TIMEOUT`"]
pub struct TIMEOUT_W<'a> {
    w: &'a mut W,
}
impl<'a> TIMEOUT_W<'a> {
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
#[doc = "Write proxy for field `EVTCAPT`"]
pub struct EVTCAPT_W<'a> {
    w: &'a mut W,
}
impl<'a> EVTCAPT_W<'a> {
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
impl W {
    #[doc = "Bit 0 - Clear Timeout Interrupt"]
    #[inline(always)]
    pub fn timeout(&mut self) -> TIMEOUT_W {
        TIMEOUT_W { w: self }
    }
    #[doc = "Bit 1 - Clear Captured Event Interrupt"]
    #[inline(always)]
    pub fn evtcapt(&mut self) -> EVTCAPT_W {
        EVTCAPT_W { w: self }
    }
}
