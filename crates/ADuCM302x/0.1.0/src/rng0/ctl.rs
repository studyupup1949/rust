#[doc = "Reader of register CTL"]
pub type R = crate::R<u16, super::CTL>;
#[doc = "Writer for register CTL"]
pub type W = crate::W<u16, super::CTL>;
#[doc = "Register CTL `reset()`'s with value 0"]
impl crate::ResetValue for super::CTL {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "RNG Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EN_A {
    #[doc = "0: Disable the RNG"]
    DISABLE = 0,
    #[doc = "1: Enable the RNG"]
    ENABLE = 1,
}
impl From<EN_A> for bool {
    #[inline(always)]
    fn from(variant: EN_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `EN`"]
pub type EN_R = crate::R<bool, EN_A>;
impl EN_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> EN_A {
        match self.bits {
            false => EN_A::DISABLE,
            true => EN_A::ENABLE,
        }
    }
    #[doc = "Checks if the value of the field is `DISABLE`"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == EN_A::DISABLE
    }
    #[doc = "Checks if the value of the field is `ENABLE`"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == EN_A::ENABLE
    }
}
#[doc = "Write proxy for field `EN`"]
pub struct EN_W<'a> {
    w: &'a mut W,
}
impl<'a> EN_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: EN_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Disable the RNG"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut W {
        self.variant(EN_A::DISABLE)
    }
    #[doc = "Enable the RNG"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut W {
        self.variant(EN_A::ENABLE)
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
#[doc = "Generate a Single Number\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SINGLE_A {
    #[doc = "0: Buffer Word"]
    WORD = 0,
    #[doc = "1: Single Byte"]
    SINGLE = 1,
}
impl From<SINGLE_A> for bool {
    #[inline(always)]
    fn from(variant: SINGLE_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `SINGLE`"]
pub type SINGLE_R = crate::R<bool, SINGLE_A>;
impl SINGLE_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> SINGLE_A {
        match self.bits {
            false => SINGLE_A::WORD,
            true => SINGLE_A::SINGLE,
        }
    }
    #[doc = "Checks if the value of the field is `WORD`"]
    #[inline(always)]
    pub fn is_word(&self) -> bool {
        *self == SINGLE_A::WORD
    }
    #[doc = "Checks if the value of the field is `SINGLE`"]
    #[inline(always)]
    pub fn is_single(&self) -> bool {
        *self == SINGLE_A::SINGLE
    }
}
#[doc = "Write proxy for field `SINGLE`"]
pub struct SINGLE_W<'a> {
    w: &'a mut W,
}
impl<'a> SINGLE_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: SINGLE_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Buffer Word"]
    #[inline(always)]
    pub fn word(self) -> &'a mut W {
        self.variant(SINGLE_A::WORD)
    }
    #[doc = "Single Byte"]
    #[inline(always)]
    pub fn single(self) -> &'a mut W {
        self.variant(SINGLE_A::SINGLE)
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
impl R {
    #[doc = "Bit 0 - RNG Enable"]
    #[inline(always)]
    pub fn en(&self) -> EN_R {
        EN_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 3 - Generate a Single Number"]
    #[inline(always)]
    pub fn single(&self) -> SINGLE_R {
        SINGLE_R::new(((self.bits >> 3) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - RNG Enable"]
    #[inline(always)]
    pub fn en(&mut self) -> EN_W {
        EN_W { w: self }
    }
    #[doc = "Bit 3 - Generate a Single Number"]
    #[inline(always)]
    pub fn single(&mut self) -> SINGLE_W {
        SINGLE_W { w: self }
    }
}
