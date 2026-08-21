#[doc = "Reader of register CTL"]
pub type R = crate::R<u32, super::CTL>;
#[doc = "Writer for register CTL"]
pub type W = crate::W<u32, super::CTL>;
#[doc = "Register CTL `reset()`'s with value 0x1000_0000"]
impl crate::ResetValue for super::CTL {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x1000_0000
    }
}
#[doc = "CRC Peripheral Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EN_A {
    #[doc = "0: CRC peripheral is disabled"]
    CRC_DIS = 0,
    #[doc = "1: CRC peripheral is enabled"]
    CRC_EN = 1,
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
            false => EN_A::CRC_DIS,
            true => EN_A::CRC_EN,
        }
    }
    #[doc = "Checks if the value of the field is `CRC_DIS`"]
    #[inline(always)]
    pub fn is_crc_dis(&self) -> bool {
        *self == EN_A::CRC_DIS
    }
    #[doc = "Checks if the value of the field is `CRC_EN`"]
    #[inline(always)]
    pub fn is_crc_en(&self) -> bool {
        *self == EN_A::CRC_EN
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
    #[doc = "CRC peripheral is disabled"]
    #[inline(always)]
    pub fn crc_dis(self) -> &'a mut W {
        self.variant(EN_A::CRC_DIS)
    }
    #[doc = "CRC peripheral is enabled"]
    #[inline(always)]
    pub fn crc_en(self) -> &'a mut W {
        self.variant(EN_A::CRC_EN)
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
        self.w.bits = (self.w.bits & !0x01) | ((value as u32) & 0x01);
        self.w
    }
}
#[doc = "LSB First Calculation Order\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LSBFIRST_A {
    #[doc = "0: MSB First CRC calculation is done"]
    MSB_FIRST = 0,
    #[doc = "1: LSB First CRC calculation is done"]
    LSB_FIRST = 1,
}
impl From<LSBFIRST_A> for bool {
    #[inline(always)]
    fn from(variant: LSBFIRST_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `LSBFIRST`"]
pub type LSBFIRST_R = crate::R<bool, LSBFIRST_A>;
impl LSBFIRST_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> LSBFIRST_A {
        match self.bits {
            false => LSBFIRST_A::MSB_FIRST,
            true => LSBFIRST_A::LSB_FIRST,
        }
    }
    #[doc = "Checks if the value of the field is `MSB_FIRST`"]
    #[inline(always)]
    pub fn is_msb_first(&self) -> bool {
        *self == LSBFIRST_A::MSB_FIRST
    }
    #[doc = "Checks if the value of the field is `LSB_FIRST`"]
    #[inline(always)]
    pub fn is_lsb_first(&self) -> bool {
        *self == LSBFIRST_A::LSB_FIRST
    }
}
#[doc = "Write proxy for field `LSBFIRST`"]
pub struct LSBFIRST_W<'a> {
    w: &'a mut W,
}
impl<'a> LSBFIRST_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: LSBFIRST_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "MSB First CRC calculation is done"]
    #[inline(always)]
    pub fn msb_first(self) -> &'a mut W {
        self.variant(LSBFIRST_A::MSB_FIRST)
    }
    #[doc = "LSB First CRC calculation is done"]
    #[inline(always)]
    pub fn lsb_first(self) -> &'a mut W {
        self.variant(LSBFIRST_A::LSB_FIRST)
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
        self.w.bits = (self.w.bits & !(0x01 << 1)) | (((value as u32) & 0x01) << 1);
        self.w
    }
}
#[doc = "Bit Mirroring\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BITMIRR_A {
    #[doc = "0: Bit Mirroring is disabled"]
    BITMIRR_DIS = 0,
    #[doc = "1: Bit Mirroring is enabled"]
    BITMIRR_EN = 1,
}
impl From<BITMIRR_A> for bool {
    #[inline(always)]
    fn from(variant: BITMIRR_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `BITMIRR`"]
pub type BITMIRR_R = crate::R<bool, BITMIRR_A>;
impl BITMIRR_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> BITMIRR_A {
        match self.bits {
            false => BITMIRR_A::BITMIRR_DIS,
            true => BITMIRR_A::BITMIRR_EN,
        }
    }
    #[doc = "Checks if the value of the field is `BITMIRR_DIS`"]
    #[inline(always)]
    pub fn is_bitmirr_dis(&self) -> bool {
        *self == BITMIRR_A::BITMIRR_DIS
    }
    #[doc = "Checks if the value of the field is `BITMIRR_EN`"]
    #[inline(always)]
    pub fn is_bitmirr_en(&self) -> bool {
        *self == BITMIRR_A::BITMIRR_EN
    }
}
#[doc = "Write proxy for field `BITMIRR`"]
pub struct BITMIRR_W<'a> {
    w: &'a mut W,
}
impl<'a> BITMIRR_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: BITMIRR_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Bit Mirroring is disabled"]
    #[inline(always)]
    pub fn bitmirr_dis(self) -> &'a mut W {
        self.variant(BITMIRR_A::BITMIRR_DIS)
    }
    #[doc = "Bit Mirroring is enabled"]
    #[inline(always)]
    pub fn bitmirr_en(self) -> &'a mut W {
        self.variant(BITMIRR_A::BITMIRR_EN)
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
        self.w.bits = (self.w.bits & !(0x01 << 2)) | (((value as u32) & 0x01) << 2);
        self.w
    }
}
#[doc = "Byte Mirroring\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BYTMIRR_A {
    #[doc = "0: Byte Mirroring is disabled"]
    BYTEMIR_DIS = 0,
    #[doc = "1: Byte Mirroring is enabled"]
    BYTEMIR_EN = 1,
}
impl From<BYTMIRR_A> for bool {
    #[inline(always)]
    fn from(variant: BYTMIRR_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `BYTMIRR`"]
pub type BYTMIRR_R = crate::R<bool, BYTMIRR_A>;
impl BYTMIRR_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> BYTMIRR_A {
        match self.bits {
            false => BYTMIRR_A::BYTEMIR_DIS,
            true => BYTMIRR_A::BYTEMIR_EN,
        }
    }
    #[doc = "Checks if the value of the field is `BYTEMIR_DIS`"]
    #[inline(always)]
    pub fn is_bytemir_dis(&self) -> bool {
        *self == BYTMIRR_A::BYTEMIR_DIS
    }
    #[doc = "Checks if the value of the field is `BYTEMIR_EN`"]
    #[inline(always)]
    pub fn is_bytemir_en(&self) -> bool {
        *self == BYTMIRR_A::BYTEMIR_EN
    }
}
#[doc = "Write proxy for field `BYTMIRR`"]
pub struct BYTMIRR_W<'a> {
    w: &'a mut W,
}
impl<'a> BYTMIRR_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: BYTMIRR_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Byte Mirroring is disabled"]
    #[inline(always)]
    pub fn bytemir_dis(self) -> &'a mut W {
        self.variant(BYTMIRR_A::BYTEMIR_DIS)
    }
    #[doc = "Byte Mirroring is enabled"]
    #[inline(always)]
    pub fn bytemir_en(self) -> &'a mut W {
        self.variant(BYTMIRR_A::BYTEMIR_EN)
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
        self.w.bits = (self.w.bits & !(0x01 << 3)) | (((value as u32) & 0x01) << 3);
        self.w
    }
}
#[doc = "Word16 Swap\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum W16SWP_A {
    #[doc = "0: Word16 Swap disabled"]
    W16SP_DIS = 0,
    #[doc = "1: Word16 Swap enabled"]
    W16SP_EN = 1,
}
impl From<W16SWP_A> for bool {
    #[inline(always)]
    fn from(variant: W16SWP_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `W16SWP`"]
pub type W16SWP_R = crate::R<bool, W16SWP_A>;
impl W16SWP_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> W16SWP_A {
        match self.bits {
            false => W16SWP_A::W16SP_DIS,
            true => W16SWP_A::W16SP_EN,
        }
    }
    #[doc = "Checks if the value of the field is `W16SP_DIS`"]
    #[inline(always)]
    pub fn is_w16sp_dis(&self) -> bool {
        *self == W16SWP_A::W16SP_DIS
    }
    #[doc = "Checks if the value of the field is `W16SP_EN`"]
    #[inline(always)]
    pub fn is_w16sp_en(&self) -> bool {
        *self == W16SWP_A::W16SP_EN
    }
}
#[doc = "Write proxy for field `W16SWP`"]
pub struct W16SWP_W<'a> {
    w: &'a mut W,
}
impl<'a> W16SWP_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: W16SWP_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Word16 Swap disabled"]
    #[inline(always)]
    pub fn w16sp_dis(self) -> &'a mut W {
        self.variant(W16SWP_A::W16SP_DIS)
    }
    #[doc = "Word16 Swap enabled"]
    #[inline(always)]
    pub fn w16sp_en(self) -> &'a mut W {
        self.variant(W16SWP_A::W16SP_EN)
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
        self.w.bits = (self.w.bits & !(0x01 << 4)) | (((value as u32) & 0x01) << 4);
        self.w
    }
}
#[doc = "Reader of field `RevID`"]
pub type REVID_R = crate::R<u8, u8>;
impl R {
    #[doc = "Bit 0 - CRC Peripheral Enable"]
    #[inline(always)]
    pub fn en(&self) -> EN_R {
        EN_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - LSB First Calculation Order"]
    #[inline(always)]
    pub fn lsbfirst(&self) -> LSBFIRST_R {
        LSBFIRST_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Bit Mirroring"]
    #[inline(always)]
    pub fn bitmirr(&self) -> BITMIRR_R {
        BITMIRR_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Byte Mirroring"]
    #[inline(always)]
    pub fn bytmirr(&self) -> BYTMIRR_R {
        BYTMIRR_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Word16 Swap"]
    #[inline(always)]
    pub fn w16swp(&self) -> W16SWP_R {
        W16SWP_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bits 28:31 - Revision ID"]
    #[inline(always)]
    pub fn rev_id(&self) -> REVID_R {
        REVID_R::new(((self.bits >> 28) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - CRC Peripheral Enable"]
    #[inline(always)]
    pub fn en(&mut self) -> EN_W {
        EN_W { w: self }
    }
    #[doc = "Bit 1 - LSB First Calculation Order"]
    #[inline(always)]
    pub fn lsbfirst(&mut self) -> LSBFIRST_W {
        LSBFIRST_W { w: self }
    }
    #[doc = "Bit 2 - Bit Mirroring"]
    #[inline(always)]
    pub fn bitmirr(&mut self) -> BITMIRR_W {
        BITMIRR_W { w: self }
    }
    #[doc = "Bit 3 - Byte Mirroring"]
    #[inline(always)]
    pub fn bytmirr(&mut self) -> BYTMIRR_W {
        BYTMIRR_W { w: self }
    }
    #[doc = "Bit 4 - Word16 Swap"]
    #[inline(always)]
    pub fn w16swp(&mut self) -> W16SWP_W {
        W16SWP_W { w: self }
    }
}
