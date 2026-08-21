#[doc = "Reader of register INTCON0"]
pub type R = crate::R<u16, super::INTCON0>;
#[doc = "Writer for register INTCON0"]
pub type W = crate::W<u16, super::INTCON0>;
#[doc = "Register INTCON0 `reset()`'s with value 0"]
impl crate::ResetValue for super::INTCON0 {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `SLEEPONEXIT`"]
pub type SLEEPONEXIT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SLEEPONEXIT`"]
pub struct SLEEPONEXIT_W<'a> {
    w: &'a mut W,
}
impl<'a> SLEEPONEXIT_W<'a> {
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
#[doc = "Reader of field `SLEEPDEEP`"]
pub type SLEEPDEEP_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SLEEPDEEP`"]
pub struct SLEEPDEEP_W<'a> {
    w: &'a mut W,
}
impl<'a> SLEEPDEEP_W<'a> {
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
impl R {
    #[doc = "Bit 1 - Sleeps the core on exit from an ISR"]
    #[inline(always)]
    pub fn sleeponexit(&self) -> SLEEPONEXIT_R {
        SLEEPONEXIT_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - deep sleep flag for HIBERNATE mode"]
    #[inline(always)]
    pub fn sleepdeep(&self) -> SLEEPDEEP_R {
        SLEEPDEEP_R::new(((self.bits >> 2) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 1 - Sleeps the core on exit from an ISR"]
    #[inline(always)]
    pub fn sleeponexit(&mut self) -> SLEEPONEXIT_W {
        SLEEPONEXIT_W { w: self }
    }
    #[doc = "Bit 2 - deep sleep flag for HIBERNATE mode"]
    #[inline(always)]
    pub fn sleepdeep(&mut self) -> SLEEPDEEP_W {
        SLEEPDEEP_W { w: self }
    }
}
