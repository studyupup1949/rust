#[doc = "Reader of register CR4SS"]
pub type R = crate::R<u16, super::CR4SS>;
#[doc = "Writer for register CR4SS"]
pub type W = crate::W<u16, super::CR4SS>;
#[doc = "Register CR4SS `reset()`'s with value 0"]
impl crate::ResetValue for super::CR4SS {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Enable for Thermometer-Code Masking of the SensorStrobe Channel 1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SS1MSKEN_A {
    #[doc = "0: Do not apply a mask to SensorStrobe Channel 1 Register"]
    NO_MSK = 0,
    #[doc = "1: Apply thermometer decoded mask"]
    THERM_MSK = 1,
}
impl From<SS1MSKEN_A> for bool {
    #[inline(always)]
    fn from(variant: SS1MSKEN_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `SS1MSKEN`"]
pub type SS1MSKEN_R = crate::R<bool, SS1MSKEN_A>;
impl SS1MSKEN_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> SS1MSKEN_A {
        match self.bits {
            false => SS1MSKEN_A::NO_MSK,
            true => SS1MSKEN_A::THERM_MSK,
        }
    }
    #[doc = "Checks if the value of the field is `NO_MSK`"]
    #[inline(always)]
    pub fn is_no_msk(&self) -> bool {
        *self == SS1MSKEN_A::NO_MSK
    }
    #[doc = "Checks if the value of the field is `THERM_MSK`"]
    #[inline(always)]
    pub fn is_therm_msk(&self) -> bool {
        *self == SS1MSKEN_A::THERM_MSK
    }
}
#[doc = "Write proxy for field `SS1MSKEN`"]
pub struct SS1MSKEN_W<'a> {
    w: &'a mut W,
}
impl<'a> SS1MSKEN_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: SS1MSKEN_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Do not apply a mask to SensorStrobe Channel 1 Register"]
    #[inline(always)]
    pub fn no_msk(self) -> &'a mut W {
        self.variant(SS1MSKEN_A::NO_MSK)
    }
    #[doc = "Apply thermometer decoded mask"]
    #[inline(always)]
    pub fn therm_msk(self) -> &'a mut W {
        self.variant(SS1MSKEN_A::THERM_MSK)
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
#[doc = "Reader of field `SS1ARLEN`"]
pub type SS1ARLEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SS1ARLEN`"]
pub struct SS1ARLEN_W<'a> {
    w: &'a mut W,
}
impl<'a> SS1ARLEN_W<'a> {
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
    #[doc = "Bit 1 - Enable for Thermometer-Code Masking of the SensorStrobe Channel 1"]
    #[inline(always)]
    pub fn ss1msken(&self) -> SS1MSKEN_R {
        SS1MSKEN_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 9 - Enable for Auto-Reloading When SensorStrobe Match Occurs"]
    #[inline(always)]
    pub fn ss1arlen(&self) -> SS1ARLEN_R {
        SS1ARLEN_R::new(((self.bits >> 9) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 1 - Enable for Thermometer-Code Masking of the SensorStrobe Channel 1"]
    #[inline(always)]
    pub fn ss1msken(&mut self) -> SS1MSKEN_W {
        SS1MSKEN_W { w: self }
    }
    #[doc = "Bit 9 - Enable for Auto-Reloading When SensorStrobe Match Occurs"]
    #[inline(always)]
    pub fn ss1arlen(&mut self) -> SS1ARLEN_W {
        SS1ARLEN_W { w: self }
    }
}
