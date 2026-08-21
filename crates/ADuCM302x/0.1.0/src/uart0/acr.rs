#[doc = "Reader of register ACR"]
pub type R = crate::R<u16, super::ACR>;
#[doc = "Writer for register ACR"]
pub type W = crate::W<u16, super::ACR>;
#[doc = "Register ACR `reset()`'s with value 0"]
impl crate::ResetValue for super::ACR {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Auto Baud Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ABE_A {
    #[doc = "0: Disable auto baudrate"]
    DIS_AUTOBAUD = 0,
    #[doc = "1: Enable auto baudrate"]
    EN_AUTOBAUD = 1,
}
impl From<ABE_A> for bool {
    #[inline(always)]
    fn from(variant: ABE_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `ABE`"]
pub type ABE_R = crate::R<bool, ABE_A>;
impl ABE_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> ABE_A {
        match self.bits {
            false => ABE_A::DIS_AUTOBAUD,
            true => ABE_A::EN_AUTOBAUD,
        }
    }
    #[doc = "Checks if the value of the field is `DIS_AUTOBAUD`"]
    #[inline(always)]
    pub fn is_dis_autobaud(&self) -> bool {
        *self == ABE_A::DIS_AUTOBAUD
    }
    #[doc = "Checks if the value of the field is `EN_AUTOBAUD`"]
    #[inline(always)]
    pub fn is_en_autobaud(&self) -> bool {
        *self == ABE_A::EN_AUTOBAUD
    }
}
#[doc = "Write proxy for field `ABE`"]
pub struct ABE_W<'a> {
    w: &'a mut W,
}
impl<'a> ABE_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: ABE_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Disable auto baudrate"]
    #[inline(always)]
    pub fn dis_autobaud(self) -> &'a mut W {
        self.variant(ABE_A::DIS_AUTOBAUD)
    }
    #[doc = "Enable auto baudrate"]
    #[inline(always)]
    pub fn en_autobaud(self) -> &'a mut W {
        self.variant(ABE_A::EN_AUTOBAUD)
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
#[doc = "Enable Done Interrupt\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DNIEN_A {
    #[doc = "0: Disable done interrupt"]
    DIS_DONEINT = 0,
    #[doc = "1: Enable done interrupt"]
    EN_DONEINT = 1,
}
impl From<DNIEN_A> for bool {
    #[inline(always)]
    fn from(variant: DNIEN_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `DNIEN`"]
pub type DNIEN_R = crate::R<bool, DNIEN_A>;
impl DNIEN_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> DNIEN_A {
        match self.bits {
            false => DNIEN_A::DIS_DONEINT,
            true => DNIEN_A::EN_DONEINT,
        }
    }
    #[doc = "Checks if the value of the field is `DIS_DONEINT`"]
    #[inline(always)]
    pub fn is_dis_doneint(&self) -> bool {
        *self == DNIEN_A::DIS_DONEINT
    }
    #[doc = "Checks if the value of the field is `EN_DONEINT`"]
    #[inline(always)]
    pub fn is_en_doneint(&self) -> bool {
        *self == DNIEN_A::EN_DONEINT
    }
}
#[doc = "Write proxy for field `DNIEN`"]
pub struct DNIEN_W<'a> {
    w: &'a mut W,
}
impl<'a> DNIEN_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: DNIEN_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Disable done interrupt"]
    #[inline(always)]
    pub fn dis_doneint(self) -> &'a mut W {
        self.variant(DNIEN_A::DIS_DONEINT)
    }
    #[doc = "Enable done interrupt"]
    #[inline(always)]
    pub fn en_doneint(self) -> &'a mut W {
        self.variant(DNIEN_A::EN_DONEINT)
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
#[doc = "Enable Time-out Interrupt\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TOIEN_A {
    #[doc = "0: Disable timeout interrupt"]
    DIS_TIMEOUTINT = 0,
    #[doc = "1: Enable timeout interrupt"]
    EN_TIMEOUTINT = 1,
}
impl From<TOIEN_A> for bool {
    #[inline(always)]
    fn from(variant: TOIEN_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `TOIEN`"]
pub type TOIEN_R = crate::R<bool, TOIEN_A>;
impl TOIEN_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> TOIEN_A {
        match self.bits {
            false => TOIEN_A::DIS_TIMEOUTINT,
            true => TOIEN_A::EN_TIMEOUTINT,
        }
    }
    #[doc = "Checks if the value of the field is `DIS_TIMEOUTINT`"]
    #[inline(always)]
    pub fn is_dis_timeoutint(&self) -> bool {
        *self == TOIEN_A::DIS_TIMEOUTINT
    }
    #[doc = "Checks if the value of the field is `EN_TIMEOUTINT`"]
    #[inline(always)]
    pub fn is_en_timeoutint(&self) -> bool {
        *self == TOIEN_A::EN_TIMEOUTINT
    }
}
#[doc = "Write proxy for field `TOIEN`"]
pub struct TOIEN_W<'a> {
    w: &'a mut W,
}
impl<'a> TOIEN_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: TOIEN_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Disable timeout interrupt"]
    #[inline(always)]
    pub fn dis_timeoutint(self) -> &'a mut W {
        self.variant(TOIEN_A::DIS_TIMEOUTINT)
    }
    #[doc = "Enable timeout interrupt"]
    #[inline(always)]
    pub fn en_timeoutint(self) -> &'a mut W {
        self.variant(TOIEN_A::EN_TIMEOUTINT)
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
#[doc = "Starting Edge Count\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum SEC_A {
    #[doc = "0: First edge"]
    SEC_EDGE1 = 0,
    #[doc = "1: Second edge"]
    SEC_EDGE2 = 1,
    #[doc = "2: Third edge"]
    SEC_EDGE3 = 2,
    #[doc = "3: Fourth edge"]
    SEC_EDGE4 = 3,
    #[doc = "4: Fifth edge"]
    SEC_EDGE5 = 4,
    #[doc = "5: Sixth edge"]
    SEC_EDGE6 = 5,
    #[doc = "6: Seventh edge"]
    SEC_EDGE7 = 6,
    #[doc = "7: Eighth edge"]
    SEC_EDGE8 = 7,
}
impl From<SEC_A> for u8 {
    #[inline(always)]
    fn from(variant: SEC_A) -> Self {
        variant as _
    }
}
#[doc = "Reader of field `SEC`"]
pub type SEC_R = crate::R<u8, SEC_A>;
impl SEC_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> SEC_A {
        match self.bits {
            0 => SEC_A::SEC_EDGE1,
            1 => SEC_A::SEC_EDGE2,
            2 => SEC_A::SEC_EDGE3,
            3 => SEC_A::SEC_EDGE4,
            4 => SEC_A::SEC_EDGE5,
            5 => SEC_A::SEC_EDGE6,
            6 => SEC_A::SEC_EDGE7,
            7 => SEC_A::SEC_EDGE8,
            _ => unreachable!(),
        }
    }
    #[doc = "Checks if the value of the field is `SEC_EDGE1`"]
    #[inline(always)]
    pub fn is_sec_edge1(&self) -> bool {
        *self == SEC_A::SEC_EDGE1
    }
    #[doc = "Checks if the value of the field is `SEC_EDGE2`"]
    #[inline(always)]
    pub fn is_sec_edge2(&self) -> bool {
        *self == SEC_A::SEC_EDGE2
    }
    #[doc = "Checks if the value of the field is `SEC_EDGE3`"]
    #[inline(always)]
    pub fn is_sec_edge3(&self) -> bool {
        *self == SEC_A::SEC_EDGE3
    }
    #[doc = "Checks if the value of the field is `SEC_EDGE4`"]
    #[inline(always)]
    pub fn is_sec_edge4(&self) -> bool {
        *self == SEC_A::SEC_EDGE4
    }
    #[doc = "Checks if the value of the field is `SEC_EDGE5`"]
    #[inline(always)]
    pub fn is_sec_edge5(&self) -> bool {
        *self == SEC_A::SEC_EDGE5
    }
    #[doc = "Checks if the value of the field is `SEC_EDGE6`"]
    #[inline(always)]
    pub fn is_sec_edge6(&self) -> bool {
        *self == SEC_A::SEC_EDGE6
    }
    #[doc = "Checks if the value of the field is `SEC_EDGE7`"]
    #[inline(always)]
    pub fn is_sec_edge7(&self) -> bool {
        *self == SEC_A::SEC_EDGE7
    }
    #[doc = "Checks if the value of the field is `SEC_EDGE8`"]
    #[inline(always)]
    pub fn is_sec_edge8(&self) -> bool {
        *self == SEC_A::SEC_EDGE8
    }
}
#[doc = "Write proxy for field `SEC`"]
pub struct SEC_W<'a> {
    w: &'a mut W,
}
impl<'a> SEC_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: SEC_A) -> &'a mut W {
        {
            self.bits(variant.into())
        }
    }
    #[doc = "First edge"]
    #[inline(always)]
    pub fn sec_edge1(self) -> &'a mut W {
        self.variant(SEC_A::SEC_EDGE1)
    }
    #[doc = "Second edge"]
    #[inline(always)]
    pub fn sec_edge2(self) -> &'a mut W {
        self.variant(SEC_A::SEC_EDGE2)
    }
    #[doc = "Third edge"]
    #[inline(always)]
    pub fn sec_edge3(self) -> &'a mut W {
        self.variant(SEC_A::SEC_EDGE3)
    }
    #[doc = "Fourth edge"]
    #[inline(always)]
    pub fn sec_edge4(self) -> &'a mut W {
        self.variant(SEC_A::SEC_EDGE4)
    }
    #[doc = "Fifth edge"]
    #[inline(always)]
    pub fn sec_edge5(self) -> &'a mut W {
        self.variant(SEC_A::SEC_EDGE5)
    }
    #[doc = "Sixth edge"]
    #[inline(always)]
    pub fn sec_edge6(self) -> &'a mut W {
        self.variant(SEC_A::SEC_EDGE6)
    }
    #[doc = "Seventh edge"]
    #[inline(always)]
    pub fn sec_edge7(self) -> &'a mut W {
        self.variant(SEC_A::SEC_EDGE7)
    }
    #[doc = "Eighth edge"]
    #[inline(always)]
    pub fn sec_edge8(self) -> &'a mut W {
        self.variant(SEC_A::SEC_EDGE8)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x07 << 4)) | (((value as u16) & 0x07) << 4);
        self.w
    }
}
#[doc = "Ending Edge Count\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum EEC_A {
    #[doc = "0: First edge"]
    EEC_EDGE1 = 0,
    #[doc = "1: Second edge"]
    EEC_EDGE2 = 1,
    #[doc = "2: Third edge"]
    EEC_EDGE3 = 2,
    #[doc = "3: Fourth edge"]
    EEC_EDGE4 = 3,
    #[doc = "4: Fifth edge"]
    EEC_EDGE5 = 4,
    #[doc = "5: Sixth edge"]
    EEC_EDGE6 = 5,
    #[doc = "6: Seventh edge"]
    EEC_EDGE7 = 6,
    #[doc = "7: Eighth edge"]
    EEC_EDGE8 = 7,
    #[doc = "8: Ninth edge"]
    EEC_EDGE9 = 8,
}
impl From<EEC_A> for u8 {
    #[inline(always)]
    fn from(variant: EEC_A) -> Self {
        variant as _
    }
}
#[doc = "Reader of field `EEC`"]
pub type EEC_R = crate::R<u8, EEC_A>;
impl EEC_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> crate::Variant<u8, EEC_A> {
        use crate::Variant::*;
        match self.bits {
            0 => Val(EEC_A::EEC_EDGE1),
            1 => Val(EEC_A::EEC_EDGE2),
            2 => Val(EEC_A::EEC_EDGE3),
            3 => Val(EEC_A::EEC_EDGE4),
            4 => Val(EEC_A::EEC_EDGE5),
            5 => Val(EEC_A::EEC_EDGE6),
            6 => Val(EEC_A::EEC_EDGE7),
            7 => Val(EEC_A::EEC_EDGE8),
            8 => Val(EEC_A::EEC_EDGE9),
            i => Res(i),
        }
    }
    #[doc = "Checks if the value of the field is `EEC_EDGE1`"]
    #[inline(always)]
    pub fn is_eec_edge1(&self) -> bool {
        *self == EEC_A::EEC_EDGE1
    }
    #[doc = "Checks if the value of the field is `EEC_EDGE2`"]
    #[inline(always)]
    pub fn is_eec_edge2(&self) -> bool {
        *self == EEC_A::EEC_EDGE2
    }
    #[doc = "Checks if the value of the field is `EEC_EDGE3`"]
    #[inline(always)]
    pub fn is_eec_edge3(&self) -> bool {
        *self == EEC_A::EEC_EDGE3
    }
    #[doc = "Checks if the value of the field is `EEC_EDGE4`"]
    #[inline(always)]
    pub fn is_eec_edge4(&self) -> bool {
        *self == EEC_A::EEC_EDGE4
    }
    #[doc = "Checks if the value of the field is `EEC_EDGE5`"]
    #[inline(always)]
    pub fn is_eec_edge5(&self) -> bool {
        *self == EEC_A::EEC_EDGE5
    }
    #[doc = "Checks if the value of the field is `EEC_EDGE6`"]
    #[inline(always)]
    pub fn is_eec_edge6(&self) -> bool {
        *self == EEC_A::EEC_EDGE6
    }
    #[doc = "Checks if the value of the field is `EEC_EDGE7`"]
    #[inline(always)]
    pub fn is_eec_edge7(&self) -> bool {
        *self == EEC_A::EEC_EDGE7
    }
    #[doc = "Checks if the value of the field is `EEC_EDGE8`"]
    #[inline(always)]
    pub fn is_eec_edge8(&self) -> bool {
        *self == EEC_A::EEC_EDGE8
    }
    #[doc = "Checks if the value of the field is `EEC_EDGE9`"]
    #[inline(always)]
    pub fn is_eec_edge9(&self) -> bool {
        *self == EEC_A::EEC_EDGE9
    }
}
#[doc = "Write proxy for field `EEC`"]
pub struct EEC_W<'a> {
    w: &'a mut W,
}
impl<'a> EEC_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: EEC_A) -> &'a mut W {
        unsafe { self.bits(variant.into()) }
    }
    #[doc = "First edge"]
    #[inline(always)]
    pub fn eec_edge1(self) -> &'a mut W {
        self.variant(EEC_A::EEC_EDGE1)
    }
    #[doc = "Second edge"]
    #[inline(always)]
    pub fn eec_edge2(self) -> &'a mut W {
        self.variant(EEC_A::EEC_EDGE2)
    }
    #[doc = "Third edge"]
    #[inline(always)]
    pub fn eec_edge3(self) -> &'a mut W {
        self.variant(EEC_A::EEC_EDGE3)
    }
    #[doc = "Fourth edge"]
    #[inline(always)]
    pub fn eec_edge4(self) -> &'a mut W {
        self.variant(EEC_A::EEC_EDGE4)
    }
    #[doc = "Fifth edge"]
    #[inline(always)]
    pub fn eec_edge5(self) -> &'a mut W {
        self.variant(EEC_A::EEC_EDGE5)
    }
    #[doc = "Sixth edge"]
    #[inline(always)]
    pub fn eec_edge6(self) -> &'a mut W {
        self.variant(EEC_A::EEC_EDGE6)
    }
    #[doc = "Seventh edge"]
    #[inline(always)]
    pub fn eec_edge7(self) -> &'a mut W {
        self.variant(EEC_A::EEC_EDGE7)
    }
    #[doc = "Eighth edge"]
    #[inline(always)]
    pub fn eec_edge8(self) -> &'a mut W {
        self.variant(EEC_A::EEC_EDGE8)
    }
    #[doc = "Ninth edge"]
    #[inline(always)]
    pub fn eec_edge9(self) -> &'a mut W {
        self.variant(EEC_A::EEC_EDGE9)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x0f << 8)) | (((value as u16) & 0x0f) << 8);
        self.w
    }
}
impl R {
    #[doc = "Bit 0 - Auto Baud Enable"]
    #[inline(always)]
    pub fn abe(&self) -> ABE_R {
        ABE_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Enable Done Interrupt"]
    #[inline(always)]
    pub fn dnien(&self) -> DNIEN_R {
        DNIEN_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Enable Time-out Interrupt"]
    #[inline(always)]
    pub fn toien(&self) -> TOIEN_R {
        TOIEN_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bits 4:6 - Starting Edge Count"]
    #[inline(always)]
    pub fn sec(&self) -> SEC_R {
        SEC_R::new(((self.bits >> 4) & 0x07) as u8)
    }
    #[doc = "Bits 8:11 - Ending Edge Count"]
    #[inline(always)]
    pub fn eec(&self) -> EEC_R {
        EEC_R::new(((self.bits >> 8) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - Auto Baud Enable"]
    #[inline(always)]
    pub fn abe(&mut self) -> ABE_W {
        ABE_W { w: self }
    }
    #[doc = "Bit 1 - Enable Done Interrupt"]
    #[inline(always)]
    pub fn dnien(&mut self) -> DNIEN_W {
        DNIEN_W { w: self }
    }
    #[doc = "Bit 2 - Enable Time-out Interrupt"]
    #[inline(always)]
    pub fn toien(&mut self) -> TOIEN_W {
        TOIEN_W { w: self }
    }
    #[doc = "Bits 4:6 - Starting Edge Count"]
    #[inline(always)]
    pub fn sec(&mut self) -> SEC_W {
        SEC_W { w: self }
    }
    #[doc = "Bits 8:11 - Ending Edge Count"]
    #[inline(always)]
    pub fn eec(&mut self) -> EEC_W {
        EEC_W { w: self }
    }
}
