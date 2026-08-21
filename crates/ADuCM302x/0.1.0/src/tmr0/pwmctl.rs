#[doc = "Reader of register PWMCTL"]
pub type R = crate::R<u16, super::PWMCTL>;
#[doc = "Writer for register PWMCTL"]
pub type W = crate::W<u16, super::PWMCTL>;
#[doc = "Register PWMCTL `reset()`'s with value 0"]
impl crate::ResetValue for super::PWMCTL {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "PWM Match Enabled\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MATCH_A {
    #[doc = "0: PWM in toggle mode"]
    PWM_TOGGLE = 0,
    #[doc = "1: PWM in match mode"]
    PWM_MATCH = 1,
}
impl From<MATCH_A> for bool {
    #[inline(always)]
    fn from(variant: MATCH_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `MATCH`"]
pub type MATCH_R = crate::R<bool, MATCH_A>;
impl MATCH_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> MATCH_A {
        match self.bits {
            false => MATCH_A::PWM_TOGGLE,
            true => MATCH_A::PWM_MATCH,
        }
    }
    #[doc = "Checks if the value of the field is `PWM_TOGGLE`"]
    #[inline(always)]
    pub fn is_pwm_toggle(&self) -> bool {
        *self == MATCH_A::PWM_TOGGLE
    }
    #[doc = "Checks if the value of the field is `PWM_MATCH`"]
    #[inline(always)]
    pub fn is_pwm_match(&self) -> bool {
        *self == MATCH_A::PWM_MATCH
    }
}
#[doc = "Write proxy for field `MATCH`"]
pub struct MATCH_W<'a> {
    w: &'a mut W,
}
impl<'a> MATCH_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: MATCH_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "PWM in toggle mode"]
    #[inline(always)]
    pub fn pwm_toggle(self) -> &'a mut W {
        self.variant(MATCH_A::PWM_TOGGLE)
    }
    #[doc = "PWM in match mode"]
    #[inline(always)]
    pub fn pwm_match(self) -> &'a mut W {
        self.variant(MATCH_A::PWM_MATCH)
    }
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
#[doc = "PWM Idle State\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IDLESTATE_A {
    #[doc = "0: PWM idles low"]
    IDLE_LOW = 0,
    #[doc = "1: PWM idles high"]
    IDLE_HIGH = 1,
}
impl From<IDLESTATE_A> for bool {
    #[inline(always)]
    fn from(variant: IDLESTATE_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `IDLESTATE`"]
pub type IDLESTATE_R = crate::R<bool, IDLESTATE_A>;
impl IDLESTATE_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> IDLESTATE_A {
        match self.bits {
            false => IDLESTATE_A::IDLE_LOW,
            true => IDLESTATE_A::IDLE_HIGH,
        }
    }
    #[doc = "Checks if the value of the field is `IDLE_LOW`"]
    #[inline(always)]
    pub fn is_idle_low(&self) -> bool {
        *self == IDLESTATE_A::IDLE_LOW
    }
    #[doc = "Checks if the value of the field is `IDLE_HIGH`"]
    #[inline(always)]
    pub fn is_idle_high(&self) -> bool {
        *self == IDLESTATE_A::IDLE_HIGH
    }
}
#[doc = "Write proxy for field `IDLESTATE`"]
pub struct IDLESTATE_W<'a> {
    w: &'a mut W,
}
impl<'a> IDLESTATE_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: IDLESTATE_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "PWM idles low"]
    #[inline(always)]
    pub fn idle_low(self) -> &'a mut W {
        self.variant(IDLESTATE_A::IDLE_LOW)
    }
    #[doc = "PWM idles high"]
    #[inline(always)]
    pub fn idle_high(self) -> &'a mut W {
        self.variant(IDLESTATE_A::IDLE_HIGH)
    }
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
impl R {
    #[doc = "Bit 0 - PWM Match Enabled"]
    #[inline(always)]
    pub fn match_(&self) -> MATCH_R {
        MATCH_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - PWM Idle State"]
    #[inline(always)]
    pub fn idlestate(&self) -> IDLESTATE_R {
        IDLESTATE_R::new(((self.bits >> 1) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - PWM Match Enabled"]
    #[inline(always)]
    pub fn match_(&mut self) -> MATCH_W {
        MATCH_W { w: self }
    }
    #[doc = "Bit 1 - PWM Idle State"]
    #[inline(always)]
    pub fn idlestate(&mut self) -> IDLESTATE_W {
        IDLESTATE_W { w: self }
    }
}
