#[doc = "Reader of register DS"]
pub type R = crate::R<u16, super::DS>;
#[doc = "Writer for register DS"]
pub type W = crate::W<u16, super::DS>;
#[doc = "Register DS `reset()`'s with value 0"]
impl crate::ResetValue for super::DS {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Drive Strength Pin 00\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN00_A {
    #[doc = "0: Single Drive Strength"]
    SINGLE_PIN00 = 0,
    #[doc = "1: Double Drive Strength"]
    DOUBLE_PIN00 = 1,
}
impl From<PIN00_A> for bool {
    #[inline(always)]
    fn from(variant: PIN00_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `PIN00`"]
pub type PIN00_R = crate::R<bool, PIN00_A>;
impl PIN00_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PIN00_A {
        match self.bits {
            false => PIN00_A::SINGLE_PIN00,
            true => PIN00_A::DOUBLE_PIN00,
        }
    }
    #[doc = "Checks if the value of the field is `SINGLE_PIN00`"]
    #[inline(always)]
    pub fn is_single_pin00(&self) -> bool {
        *self == PIN00_A::SINGLE_PIN00
    }
    #[doc = "Checks if the value of the field is `DOUBLE_PIN00`"]
    #[inline(always)]
    pub fn is_double_pin00(&self) -> bool {
        *self == PIN00_A::DOUBLE_PIN00
    }
}
#[doc = "Write proxy for field `PIN00`"]
pub struct PIN00_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN00_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN00_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Single Drive Strength"]
    #[inline(always)]
    pub fn single_pin00(self) -> &'a mut W {
        self.variant(PIN00_A::SINGLE_PIN00)
    }
    #[doc = "Double Drive Strength"]
    #[inline(always)]
    pub fn double_pin00(self) -> &'a mut W {
        self.variant(PIN00_A::DOUBLE_PIN00)
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
#[doc = "Drive Strength Pin 01\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN01_A {
    #[doc = "0: Single Drive Strength"]
    SINGLE_PIN01 = 0,
    #[doc = "1: Double Drive Strength"]
    DOUBLE_PIN01 = 1,
}
impl From<PIN01_A> for bool {
    #[inline(always)]
    fn from(variant: PIN01_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `PIN01`"]
pub type PIN01_R = crate::R<bool, PIN01_A>;
impl PIN01_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PIN01_A {
        match self.bits {
            false => PIN01_A::SINGLE_PIN01,
            true => PIN01_A::DOUBLE_PIN01,
        }
    }
    #[doc = "Checks if the value of the field is `SINGLE_PIN01`"]
    #[inline(always)]
    pub fn is_single_pin01(&self) -> bool {
        *self == PIN01_A::SINGLE_PIN01
    }
    #[doc = "Checks if the value of the field is `DOUBLE_PIN01`"]
    #[inline(always)]
    pub fn is_double_pin01(&self) -> bool {
        *self == PIN01_A::DOUBLE_PIN01
    }
}
#[doc = "Write proxy for field `PIN01`"]
pub struct PIN01_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN01_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN01_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Single Drive Strength"]
    #[inline(always)]
    pub fn single_pin01(self) -> &'a mut W {
        self.variant(PIN01_A::SINGLE_PIN01)
    }
    #[doc = "Double Drive Strength"]
    #[inline(always)]
    pub fn double_pin01(self) -> &'a mut W {
        self.variant(PIN01_A::DOUBLE_PIN01)
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
#[doc = "Drive Strength Pin 02\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN02_A {
    #[doc = "0: Single Drive Strength"]
    SINGLE_PIN02 = 0,
    #[doc = "1: Double Drive Strength"]
    DOUBLE_PIN02 = 1,
}
impl From<PIN02_A> for bool {
    #[inline(always)]
    fn from(variant: PIN02_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `PIN02`"]
pub type PIN02_R = crate::R<bool, PIN02_A>;
impl PIN02_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PIN02_A {
        match self.bits {
            false => PIN02_A::SINGLE_PIN02,
            true => PIN02_A::DOUBLE_PIN02,
        }
    }
    #[doc = "Checks if the value of the field is `SINGLE_PIN02`"]
    #[inline(always)]
    pub fn is_single_pin02(&self) -> bool {
        *self == PIN02_A::SINGLE_PIN02
    }
    #[doc = "Checks if the value of the field is `DOUBLE_PIN02`"]
    #[inline(always)]
    pub fn is_double_pin02(&self) -> bool {
        *self == PIN02_A::DOUBLE_PIN02
    }
}
#[doc = "Write proxy for field `PIN02`"]
pub struct PIN02_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN02_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN02_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Single Drive Strength"]
    #[inline(always)]
    pub fn single_pin02(self) -> &'a mut W {
        self.variant(PIN02_A::SINGLE_PIN02)
    }
    #[doc = "Double Drive Strength"]
    #[inline(always)]
    pub fn double_pin02(self) -> &'a mut W {
        self.variant(PIN02_A::DOUBLE_PIN02)
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
        self.w.bits = (self.w.bits & !(0x01 << 2)) | (((value as u16) & 0x01) << 2);
        self.w
    }
}
#[doc = "Drive Strength Pin 03\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN03_A {
    #[doc = "0: Single Drive Strength"]
    SINGLE_PIN03 = 0,
    #[doc = "1: Double Drive Strength"]
    DOUBLE_PIN03 = 1,
}
impl From<PIN03_A> for bool {
    #[inline(always)]
    fn from(variant: PIN03_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `PIN03`"]
pub type PIN03_R = crate::R<bool, PIN03_A>;
impl PIN03_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PIN03_A {
        match self.bits {
            false => PIN03_A::SINGLE_PIN03,
            true => PIN03_A::DOUBLE_PIN03,
        }
    }
    #[doc = "Checks if the value of the field is `SINGLE_PIN03`"]
    #[inline(always)]
    pub fn is_single_pin03(&self) -> bool {
        *self == PIN03_A::SINGLE_PIN03
    }
    #[doc = "Checks if the value of the field is `DOUBLE_PIN03`"]
    #[inline(always)]
    pub fn is_double_pin03(&self) -> bool {
        *self == PIN03_A::DOUBLE_PIN03
    }
}
#[doc = "Write proxy for field `PIN03`"]
pub struct PIN03_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN03_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN03_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Single Drive Strength"]
    #[inline(always)]
    pub fn single_pin03(self) -> &'a mut W {
        self.variant(PIN03_A::SINGLE_PIN03)
    }
    #[doc = "Double Drive Strength"]
    #[inline(always)]
    pub fn double_pin03(self) -> &'a mut W {
        self.variant(PIN03_A::DOUBLE_PIN03)
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
        self.w.bits = (self.w.bits & !(0x01 << 3)) | (((value as u16) & 0x01) << 3);
        self.w
    }
}
#[doc = "Drive Strength Pin 04\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN04_A {
    #[doc = "0: Single Drive Strength"]
    SINGLE_PIN04 = 0,
    #[doc = "1: Double Drive Strength"]
    DOUBLE_PIN04 = 1,
}
impl From<PIN04_A> for bool {
    #[inline(always)]
    fn from(variant: PIN04_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `PIN04`"]
pub type PIN04_R = crate::R<bool, PIN04_A>;
impl PIN04_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PIN04_A {
        match self.bits {
            false => PIN04_A::SINGLE_PIN04,
            true => PIN04_A::DOUBLE_PIN04,
        }
    }
    #[doc = "Checks if the value of the field is `SINGLE_PIN04`"]
    #[inline(always)]
    pub fn is_single_pin04(&self) -> bool {
        *self == PIN04_A::SINGLE_PIN04
    }
    #[doc = "Checks if the value of the field is `DOUBLE_PIN04`"]
    #[inline(always)]
    pub fn is_double_pin04(&self) -> bool {
        *self == PIN04_A::DOUBLE_PIN04
    }
}
#[doc = "Write proxy for field `PIN04`"]
pub struct PIN04_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN04_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN04_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Single Drive Strength"]
    #[inline(always)]
    pub fn single_pin04(self) -> &'a mut W {
        self.variant(PIN04_A::SINGLE_PIN04)
    }
    #[doc = "Double Drive Strength"]
    #[inline(always)]
    pub fn double_pin04(self) -> &'a mut W {
        self.variant(PIN04_A::DOUBLE_PIN04)
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
        self.w.bits = (self.w.bits & !(0x01 << 4)) | (((value as u16) & 0x01) << 4);
        self.w
    }
}
#[doc = "Drive Strength Pin 05\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN05_A {
    #[doc = "0: Single Drive Strength"]
    SINGLE_PIN05 = 0,
    #[doc = "1: Double Drive Strength"]
    DOUBLE_PIN05 = 1,
}
impl From<PIN05_A> for bool {
    #[inline(always)]
    fn from(variant: PIN05_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `PIN05`"]
pub type PIN05_R = crate::R<bool, PIN05_A>;
impl PIN05_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PIN05_A {
        match self.bits {
            false => PIN05_A::SINGLE_PIN05,
            true => PIN05_A::DOUBLE_PIN05,
        }
    }
    #[doc = "Checks if the value of the field is `SINGLE_PIN05`"]
    #[inline(always)]
    pub fn is_single_pin05(&self) -> bool {
        *self == PIN05_A::SINGLE_PIN05
    }
    #[doc = "Checks if the value of the field is `DOUBLE_PIN05`"]
    #[inline(always)]
    pub fn is_double_pin05(&self) -> bool {
        *self == PIN05_A::DOUBLE_PIN05
    }
}
#[doc = "Write proxy for field `PIN05`"]
pub struct PIN05_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN05_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN05_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Single Drive Strength"]
    #[inline(always)]
    pub fn single_pin05(self) -> &'a mut W {
        self.variant(PIN05_A::SINGLE_PIN05)
    }
    #[doc = "Double Drive Strength"]
    #[inline(always)]
    pub fn double_pin05(self) -> &'a mut W {
        self.variant(PIN05_A::DOUBLE_PIN05)
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
        self.w.bits = (self.w.bits & !(0x01 << 5)) | (((value as u16) & 0x01) << 5);
        self.w
    }
}
#[doc = "Drive Strength Pin 06\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN06_A {
    #[doc = "0: Single Drive Strength"]
    SINGLE_PIN06 = 0,
    #[doc = "1: Double Drive Strength"]
    DOUBLE_PIN06 = 1,
}
impl From<PIN06_A> for bool {
    #[inline(always)]
    fn from(variant: PIN06_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `PIN06`"]
pub type PIN06_R = crate::R<bool, PIN06_A>;
impl PIN06_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PIN06_A {
        match self.bits {
            false => PIN06_A::SINGLE_PIN06,
            true => PIN06_A::DOUBLE_PIN06,
        }
    }
    #[doc = "Checks if the value of the field is `SINGLE_PIN06`"]
    #[inline(always)]
    pub fn is_single_pin06(&self) -> bool {
        *self == PIN06_A::SINGLE_PIN06
    }
    #[doc = "Checks if the value of the field is `DOUBLE_PIN06`"]
    #[inline(always)]
    pub fn is_double_pin06(&self) -> bool {
        *self == PIN06_A::DOUBLE_PIN06
    }
}
#[doc = "Write proxy for field `PIN06`"]
pub struct PIN06_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN06_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN06_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Single Drive Strength"]
    #[inline(always)]
    pub fn single_pin06(self) -> &'a mut W {
        self.variant(PIN06_A::SINGLE_PIN06)
    }
    #[doc = "Double Drive Strength"]
    #[inline(always)]
    pub fn double_pin06(self) -> &'a mut W {
        self.variant(PIN06_A::DOUBLE_PIN06)
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
        self.w.bits = (self.w.bits & !(0x01 << 6)) | (((value as u16) & 0x01) << 6);
        self.w
    }
}
#[doc = "Drive Strength Pin 07\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN07_A {
    #[doc = "0: Single Drive Strength"]
    SINGLE_PIN07 = 0,
    #[doc = "1: Double Drive Strength"]
    DOUBLE_PIN07 = 1,
}
impl From<PIN07_A> for bool {
    #[inline(always)]
    fn from(variant: PIN07_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `PIN07`"]
pub type PIN07_R = crate::R<bool, PIN07_A>;
impl PIN07_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PIN07_A {
        match self.bits {
            false => PIN07_A::SINGLE_PIN07,
            true => PIN07_A::DOUBLE_PIN07,
        }
    }
    #[doc = "Checks if the value of the field is `SINGLE_PIN07`"]
    #[inline(always)]
    pub fn is_single_pin07(&self) -> bool {
        *self == PIN07_A::SINGLE_PIN07
    }
    #[doc = "Checks if the value of the field is `DOUBLE_PIN07`"]
    #[inline(always)]
    pub fn is_double_pin07(&self) -> bool {
        *self == PIN07_A::DOUBLE_PIN07
    }
}
#[doc = "Write proxy for field `PIN07`"]
pub struct PIN07_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN07_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN07_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Single Drive Strength"]
    #[inline(always)]
    pub fn single_pin07(self) -> &'a mut W {
        self.variant(PIN07_A::SINGLE_PIN07)
    }
    #[doc = "Double Drive Strength"]
    #[inline(always)]
    pub fn double_pin07(self) -> &'a mut W {
        self.variant(PIN07_A::DOUBLE_PIN07)
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
        self.w.bits = (self.w.bits & !(0x01 << 7)) | (((value as u16) & 0x01) << 7);
        self.w
    }
}
#[doc = "Drive Strength Pin 08\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN08_A {
    #[doc = "0: Single Drive Strength"]
    SINGLE_PIN08 = 0,
    #[doc = "1: Double Drive Strength"]
    DOUBLE_PIN08 = 1,
}
impl From<PIN08_A> for bool {
    #[inline(always)]
    fn from(variant: PIN08_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `PIN08`"]
pub type PIN08_R = crate::R<bool, PIN08_A>;
impl PIN08_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PIN08_A {
        match self.bits {
            false => PIN08_A::SINGLE_PIN08,
            true => PIN08_A::DOUBLE_PIN08,
        }
    }
    #[doc = "Checks if the value of the field is `SINGLE_PIN08`"]
    #[inline(always)]
    pub fn is_single_pin08(&self) -> bool {
        *self == PIN08_A::SINGLE_PIN08
    }
    #[doc = "Checks if the value of the field is `DOUBLE_PIN08`"]
    #[inline(always)]
    pub fn is_double_pin08(&self) -> bool {
        *self == PIN08_A::DOUBLE_PIN08
    }
}
#[doc = "Write proxy for field `PIN08`"]
pub struct PIN08_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN08_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN08_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Single Drive Strength"]
    #[inline(always)]
    pub fn single_pin08(self) -> &'a mut W {
        self.variant(PIN08_A::SINGLE_PIN08)
    }
    #[doc = "Double Drive Strength"]
    #[inline(always)]
    pub fn double_pin08(self) -> &'a mut W {
        self.variant(PIN08_A::DOUBLE_PIN08)
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
        self.w.bits = (self.w.bits & !(0x01 << 8)) | (((value as u16) & 0x01) << 8);
        self.w
    }
}
#[doc = "Drive Strength Pin 09\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN09_A {
    #[doc = "0: Single Drive Strength"]
    SINGLE_PIN09 = 0,
    #[doc = "1: Double Drive Strength"]
    DOUBLE_PIN09 = 1,
}
impl From<PIN09_A> for bool {
    #[inline(always)]
    fn from(variant: PIN09_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `PIN09`"]
pub type PIN09_R = crate::R<bool, PIN09_A>;
impl PIN09_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PIN09_A {
        match self.bits {
            false => PIN09_A::SINGLE_PIN09,
            true => PIN09_A::DOUBLE_PIN09,
        }
    }
    #[doc = "Checks if the value of the field is `SINGLE_PIN09`"]
    #[inline(always)]
    pub fn is_single_pin09(&self) -> bool {
        *self == PIN09_A::SINGLE_PIN09
    }
    #[doc = "Checks if the value of the field is `DOUBLE_PIN09`"]
    #[inline(always)]
    pub fn is_double_pin09(&self) -> bool {
        *self == PIN09_A::DOUBLE_PIN09
    }
}
#[doc = "Write proxy for field `PIN09`"]
pub struct PIN09_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN09_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN09_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Single Drive Strength"]
    #[inline(always)]
    pub fn single_pin09(self) -> &'a mut W {
        self.variant(PIN09_A::SINGLE_PIN09)
    }
    #[doc = "Double Drive Strength"]
    #[inline(always)]
    pub fn double_pin09(self) -> &'a mut W {
        self.variant(PIN09_A::DOUBLE_PIN09)
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
        self.w.bits = (self.w.bits & !(0x01 << 9)) | (((value as u16) & 0x01) << 9);
        self.w
    }
}
#[doc = "Drive Strength Pin 10\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN10_A {
    #[doc = "0: Single Drive Strength"]
    SINGLE_PIN10 = 0,
    #[doc = "1: Double Drive Strength"]
    DOUBLE_PIN10 = 1,
}
impl From<PIN10_A> for bool {
    #[inline(always)]
    fn from(variant: PIN10_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `PIN10`"]
pub type PIN10_R = crate::R<bool, PIN10_A>;
impl PIN10_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PIN10_A {
        match self.bits {
            false => PIN10_A::SINGLE_PIN10,
            true => PIN10_A::DOUBLE_PIN10,
        }
    }
    #[doc = "Checks if the value of the field is `SINGLE_PIN10`"]
    #[inline(always)]
    pub fn is_single_pin10(&self) -> bool {
        *self == PIN10_A::SINGLE_PIN10
    }
    #[doc = "Checks if the value of the field is `DOUBLE_PIN10`"]
    #[inline(always)]
    pub fn is_double_pin10(&self) -> bool {
        *self == PIN10_A::DOUBLE_PIN10
    }
}
#[doc = "Write proxy for field `PIN10`"]
pub struct PIN10_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN10_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN10_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Single Drive Strength"]
    #[inline(always)]
    pub fn single_pin10(self) -> &'a mut W {
        self.variant(PIN10_A::SINGLE_PIN10)
    }
    #[doc = "Double Drive Strength"]
    #[inline(always)]
    pub fn double_pin10(self) -> &'a mut W {
        self.variant(PIN10_A::DOUBLE_PIN10)
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
        self.w.bits = (self.w.bits & !(0x01 << 10)) | (((value as u16) & 0x01) << 10);
        self.w
    }
}
#[doc = "Drive Strength Pin 11\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN11_A {
    #[doc = "0: Single Drive Strength"]
    SINGLE_PIN11 = 0,
    #[doc = "1: Double Drive Strength"]
    DOUBLE_PIN11 = 1,
}
impl From<PIN11_A> for bool {
    #[inline(always)]
    fn from(variant: PIN11_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `PIN11`"]
pub type PIN11_R = crate::R<bool, PIN11_A>;
impl PIN11_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PIN11_A {
        match self.bits {
            false => PIN11_A::SINGLE_PIN11,
            true => PIN11_A::DOUBLE_PIN11,
        }
    }
    #[doc = "Checks if the value of the field is `SINGLE_PIN11`"]
    #[inline(always)]
    pub fn is_single_pin11(&self) -> bool {
        *self == PIN11_A::SINGLE_PIN11
    }
    #[doc = "Checks if the value of the field is `DOUBLE_PIN11`"]
    #[inline(always)]
    pub fn is_double_pin11(&self) -> bool {
        *self == PIN11_A::DOUBLE_PIN11
    }
}
#[doc = "Write proxy for field `PIN11`"]
pub struct PIN11_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN11_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN11_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Single Drive Strength"]
    #[inline(always)]
    pub fn single_pin11(self) -> &'a mut W {
        self.variant(PIN11_A::SINGLE_PIN11)
    }
    #[doc = "Double Drive Strength"]
    #[inline(always)]
    pub fn double_pin11(self) -> &'a mut W {
        self.variant(PIN11_A::DOUBLE_PIN11)
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
        self.w.bits = (self.w.bits & !(0x01 << 11)) | (((value as u16) & 0x01) << 11);
        self.w
    }
}
#[doc = "Drive Strength Pin 12\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN12_A {
    #[doc = "0: Single Drive Strength"]
    SINGLE_PIN12 = 0,
    #[doc = "1: Double Drive Strength"]
    DOUBLE_PIN12 = 1,
}
impl From<PIN12_A> for bool {
    #[inline(always)]
    fn from(variant: PIN12_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `PIN12`"]
pub type PIN12_R = crate::R<bool, PIN12_A>;
impl PIN12_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PIN12_A {
        match self.bits {
            false => PIN12_A::SINGLE_PIN12,
            true => PIN12_A::DOUBLE_PIN12,
        }
    }
    #[doc = "Checks if the value of the field is `SINGLE_PIN12`"]
    #[inline(always)]
    pub fn is_single_pin12(&self) -> bool {
        *self == PIN12_A::SINGLE_PIN12
    }
    #[doc = "Checks if the value of the field is `DOUBLE_PIN12`"]
    #[inline(always)]
    pub fn is_double_pin12(&self) -> bool {
        *self == PIN12_A::DOUBLE_PIN12
    }
}
#[doc = "Write proxy for field `PIN12`"]
pub struct PIN12_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN12_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN12_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Single Drive Strength"]
    #[inline(always)]
    pub fn single_pin12(self) -> &'a mut W {
        self.variant(PIN12_A::SINGLE_PIN12)
    }
    #[doc = "Double Drive Strength"]
    #[inline(always)]
    pub fn double_pin12(self) -> &'a mut W {
        self.variant(PIN12_A::DOUBLE_PIN12)
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
        self.w.bits = (self.w.bits & !(0x01 << 12)) | (((value as u16) & 0x01) << 12);
        self.w
    }
}
#[doc = "Drive Strength Pin 13\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN13_A {
    #[doc = "0: Single Drive Strength"]
    SINGLE_PIN13 = 0,
    #[doc = "1: Double Drive Strength"]
    DOUBLE_PIN13 = 1,
}
impl From<PIN13_A> for bool {
    #[inline(always)]
    fn from(variant: PIN13_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `PIN13`"]
pub type PIN13_R = crate::R<bool, PIN13_A>;
impl PIN13_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PIN13_A {
        match self.bits {
            false => PIN13_A::SINGLE_PIN13,
            true => PIN13_A::DOUBLE_PIN13,
        }
    }
    #[doc = "Checks if the value of the field is `SINGLE_PIN13`"]
    #[inline(always)]
    pub fn is_single_pin13(&self) -> bool {
        *self == PIN13_A::SINGLE_PIN13
    }
    #[doc = "Checks if the value of the field is `DOUBLE_PIN13`"]
    #[inline(always)]
    pub fn is_double_pin13(&self) -> bool {
        *self == PIN13_A::DOUBLE_PIN13
    }
}
#[doc = "Write proxy for field `PIN13`"]
pub struct PIN13_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN13_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN13_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Single Drive Strength"]
    #[inline(always)]
    pub fn single_pin13(self) -> &'a mut W {
        self.variant(PIN13_A::SINGLE_PIN13)
    }
    #[doc = "Double Drive Strength"]
    #[inline(always)]
    pub fn double_pin13(self) -> &'a mut W {
        self.variant(PIN13_A::DOUBLE_PIN13)
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
        self.w.bits = (self.w.bits & !(0x01 << 13)) | (((value as u16) & 0x01) << 13);
        self.w
    }
}
#[doc = "Drive Strength Pin 14\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN14_A {
    #[doc = "0: Single Drive Strength"]
    SINGLE_PIN14 = 0,
    #[doc = "1: Double Drive Strength"]
    DOUBLE_PIN14 = 1,
}
impl From<PIN14_A> for bool {
    #[inline(always)]
    fn from(variant: PIN14_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `PIN14`"]
pub type PIN14_R = crate::R<bool, PIN14_A>;
impl PIN14_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PIN14_A {
        match self.bits {
            false => PIN14_A::SINGLE_PIN14,
            true => PIN14_A::DOUBLE_PIN14,
        }
    }
    #[doc = "Checks if the value of the field is `SINGLE_PIN14`"]
    #[inline(always)]
    pub fn is_single_pin14(&self) -> bool {
        *self == PIN14_A::SINGLE_PIN14
    }
    #[doc = "Checks if the value of the field is `DOUBLE_PIN14`"]
    #[inline(always)]
    pub fn is_double_pin14(&self) -> bool {
        *self == PIN14_A::DOUBLE_PIN14
    }
}
#[doc = "Write proxy for field `PIN14`"]
pub struct PIN14_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN14_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN14_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Single Drive Strength"]
    #[inline(always)]
    pub fn single_pin14(self) -> &'a mut W {
        self.variant(PIN14_A::SINGLE_PIN14)
    }
    #[doc = "Double Drive Strength"]
    #[inline(always)]
    pub fn double_pin14(self) -> &'a mut W {
        self.variant(PIN14_A::DOUBLE_PIN14)
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
        self.w.bits = (self.w.bits & !(0x01 << 14)) | (((value as u16) & 0x01) << 14);
        self.w
    }
}
#[doc = "Drive Strength Pin 15\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PIN15_A {
    #[doc = "0: Single Drive Strength"]
    SINGLE_PIN15 = 0,
    #[doc = "1: Double Drive Strength"]
    DOUBLE_PIN15 = 1,
}
impl From<PIN15_A> for bool {
    #[inline(always)]
    fn from(variant: PIN15_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `PIN15`"]
pub type PIN15_R = crate::R<bool, PIN15_A>;
impl PIN15_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PIN15_A {
        match self.bits {
            false => PIN15_A::SINGLE_PIN15,
            true => PIN15_A::DOUBLE_PIN15,
        }
    }
    #[doc = "Checks if the value of the field is `SINGLE_PIN15`"]
    #[inline(always)]
    pub fn is_single_pin15(&self) -> bool {
        *self == PIN15_A::SINGLE_PIN15
    }
    #[doc = "Checks if the value of the field is `DOUBLE_PIN15`"]
    #[inline(always)]
    pub fn is_double_pin15(&self) -> bool {
        *self == PIN15_A::DOUBLE_PIN15
    }
}
#[doc = "Write proxy for field `PIN15`"]
pub struct PIN15_W<'a> {
    w: &'a mut W,
}
impl<'a> PIN15_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PIN15_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Single Drive Strength"]
    #[inline(always)]
    pub fn single_pin15(self) -> &'a mut W {
        self.variant(PIN15_A::SINGLE_PIN15)
    }
    #[doc = "Double Drive Strength"]
    #[inline(always)]
    pub fn double_pin15(self) -> &'a mut W {
        self.variant(PIN15_A::DOUBLE_PIN15)
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
        self.w.bits = (self.w.bits & !(0x01 << 15)) | (((value as u16) & 0x01) << 15);
        self.w
    }
}
impl R {
    #[doc = "Bit 0 - Drive Strength Pin 00"]
    #[inline(always)]
    pub fn pin00(&self) -> PIN00_R {
        PIN00_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Drive Strength Pin 01"]
    #[inline(always)]
    pub fn pin01(&self) -> PIN01_R {
        PIN01_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Drive Strength Pin 02"]
    #[inline(always)]
    pub fn pin02(&self) -> PIN02_R {
        PIN02_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Drive Strength Pin 03"]
    #[inline(always)]
    pub fn pin03(&self) -> PIN03_R {
        PIN03_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Drive Strength Pin 04"]
    #[inline(always)]
    pub fn pin04(&self) -> PIN04_R {
        PIN04_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bit 5 - Drive Strength Pin 05"]
    #[inline(always)]
    pub fn pin05(&self) -> PIN05_R {
        PIN05_R::new(((self.bits >> 5) & 0x01) != 0)
    }
    #[doc = "Bit 6 - Drive Strength Pin 06"]
    #[inline(always)]
    pub fn pin06(&self) -> PIN06_R {
        PIN06_R::new(((self.bits >> 6) & 0x01) != 0)
    }
    #[doc = "Bit 7 - Drive Strength Pin 07"]
    #[inline(always)]
    pub fn pin07(&self) -> PIN07_R {
        PIN07_R::new(((self.bits >> 7) & 0x01) != 0)
    }
    #[doc = "Bit 8 - Drive Strength Pin 08"]
    #[inline(always)]
    pub fn pin08(&self) -> PIN08_R {
        PIN08_R::new(((self.bits >> 8) & 0x01) != 0)
    }
    #[doc = "Bit 9 - Drive Strength Pin 09"]
    #[inline(always)]
    pub fn pin09(&self) -> PIN09_R {
        PIN09_R::new(((self.bits >> 9) & 0x01) != 0)
    }
    #[doc = "Bit 10 - Drive Strength Pin 10"]
    #[inline(always)]
    pub fn pin10(&self) -> PIN10_R {
        PIN10_R::new(((self.bits >> 10) & 0x01) != 0)
    }
    #[doc = "Bit 11 - Drive Strength Pin 11"]
    #[inline(always)]
    pub fn pin11(&self) -> PIN11_R {
        PIN11_R::new(((self.bits >> 11) & 0x01) != 0)
    }
    #[doc = "Bit 12 - Drive Strength Pin 12"]
    #[inline(always)]
    pub fn pin12(&self) -> PIN12_R {
        PIN12_R::new(((self.bits >> 12) & 0x01) != 0)
    }
    #[doc = "Bit 13 - Drive Strength Pin 13"]
    #[inline(always)]
    pub fn pin13(&self) -> PIN13_R {
        PIN13_R::new(((self.bits >> 13) & 0x01) != 0)
    }
    #[doc = "Bit 14 - Drive Strength Pin 14"]
    #[inline(always)]
    pub fn pin14(&self) -> PIN14_R {
        PIN14_R::new(((self.bits >> 14) & 0x01) != 0)
    }
    #[doc = "Bit 15 - Drive Strength Pin 15"]
    #[inline(always)]
    pub fn pin15(&self) -> PIN15_R {
        PIN15_R::new(((self.bits >> 15) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Drive Strength Pin 00"]
    #[inline(always)]
    pub fn pin00(&mut self) -> PIN00_W {
        PIN00_W { w: self }
    }
    #[doc = "Bit 1 - Drive Strength Pin 01"]
    #[inline(always)]
    pub fn pin01(&mut self) -> PIN01_W {
        PIN01_W { w: self }
    }
    #[doc = "Bit 2 - Drive Strength Pin 02"]
    #[inline(always)]
    pub fn pin02(&mut self) -> PIN02_W {
        PIN02_W { w: self }
    }
    #[doc = "Bit 3 - Drive Strength Pin 03"]
    #[inline(always)]
    pub fn pin03(&mut self) -> PIN03_W {
        PIN03_W { w: self }
    }
    #[doc = "Bit 4 - Drive Strength Pin 04"]
    #[inline(always)]
    pub fn pin04(&mut self) -> PIN04_W {
        PIN04_W { w: self }
    }
    #[doc = "Bit 5 - Drive Strength Pin 05"]
    #[inline(always)]
    pub fn pin05(&mut self) -> PIN05_W {
        PIN05_W { w: self }
    }
    #[doc = "Bit 6 - Drive Strength Pin 06"]
    #[inline(always)]
    pub fn pin06(&mut self) -> PIN06_W {
        PIN06_W { w: self }
    }
    #[doc = "Bit 7 - Drive Strength Pin 07"]
    #[inline(always)]
    pub fn pin07(&mut self) -> PIN07_W {
        PIN07_W { w: self }
    }
    #[doc = "Bit 8 - Drive Strength Pin 08"]
    #[inline(always)]
    pub fn pin08(&mut self) -> PIN08_W {
        PIN08_W { w: self }
    }
    #[doc = "Bit 9 - Drive Strength Pin 09"]
    #[inline(always)]
    pub fn pin09(&mut self) -> PIN09_W {
        PIN09_W { w: self }
    }
    #[doc = "Bit 10 - Drive Strength Pin 10"]
    #[inline(always)]
    pub fn pin10(&mut self) -> PIN10_W {
        PIN10_W { w: self }
    }
    #[doc = "Bit 11 - Drive Strength Pin 11"]
    #[inline(always)]
    pub fn pin11(&mut self) -> PIN11_W {
        PIN11_W { w: self }
    }
    #[doc = "Bit 12 - Drive Strength Pin 12"]
    #[inline(always)]
    pub fn pin12(&mut self) -> PIN12_W {
        PIN12_W { w: self }
    }
    #[doc = "Bit 13 - Drive Strength Pin 13"]
    #[inline(always)]
    pub fn pin13(&mut self) -> PIN13_W {
        PIN13_W { w: self }
    }
    #[doc = "Bit 14 - Drive Strength Pin 14"]
    #[inline(always)]
    pub fn pin14(&mut self) -> PIN14_W {
        PIN14_W { w: self }
    }
    #[doc = "Bit 15 - Drive Strength Pin 15"]
    #[inline(always)]
    pub fn pin15(&mut self) -> PIN15_W {
        PIN15_W { w: self }
    }
}
