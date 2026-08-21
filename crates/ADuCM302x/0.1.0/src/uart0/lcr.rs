#[doc = "Reader of register LCR"]
pub type R = crate::R<u16, super::LCR>;
#[doc = "Writer for register LCR"]
pub type W = crate::W<u16, super::LCR>;
#[doc = "Register LCR `reset()`'s with value 0"]
impl crate::ResetValue for super::LCR {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `WLS`"]
pub type WLS_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `WLS`"]
pub struct WLS_W<'a> {
    w: &'a mut W,
}
impl<'a> WLS_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x03) | ((value as u16) & 0x03);
        self.w
    }
}
#[doc = "Reader of field `STOP`"]
pub type STOP_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `STOP`"]
pub struct STOP_W<'a> {
    w: &'a mut W,
}
impl<'a> STOP_W<'a> {
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
#[doc = "Reader of field `PEN`"]
pub type PEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `PEN`"]
pub struct PEN_W<'a> {
    w: &'a mut W,
}
impl<'a> PEN_W<'a> {
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
#[doc = "Reader of field `EPS`"]
pub type EPS_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `EPS`"]
pub struct EPS_W<'a> {
    w: &'a mut W,
}
impl<'a> EPS_W<'a> {
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
#[doc = "Stick Parity\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SP_A {
    #[doc = "0: Parity will not be forced based on Parity Select and Parity Enable bits."]
    PAR_NOTFORCED = 0,
    #[doc = "1: Parity forced based on Parity Select and Parity Enable bits."]
    PAR_FORCED = 1,
}
impl From<SP_A> for bool {
    #[inline(always)]
    fn from(variant: SP_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `SP`"]
pub type SP_R = crate::R<bool, SP_A>;
impl SP_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> SP_A {
        match self.bits {
            false => SP_A::PAR_NOTFORCED,
            true => SP_A::PAR_FORCED,
        }
    }
    #[doc = "Checks if the value of the field is `PAR_NOTFORCED`"]
    #[inline(always)]
    pub fn is_par_notforced(&self) -> bool {
        *self == SP_A::PAR_NOTFORCED
    }
    #[doc = "Checks if the value of the field is `PAR_FORCED`"]
    #[inline(always)]
    pub fn is_par_forced(&self) -> bool {
        *self == SP_A::PAR_FORCED
    }
}
#[doc = "Write proxy for field `SP`"]
pub struct SP_W<'a> {
    w: &'a mut W,
}
impl<'a> SP_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: SP_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Parity will not be forced based on Parity Select and Parity Enable bits."]
    #[inline(always)]
    pub fn par_notforced(self) -> &'a mut W {
        self.variant(SP_A::PAR_NOTFORCED)
    }
    #[doc = "Parity forced based on Parity Select and Parity Enable bits."]
    #[inline(always)]
    pub fn par_forced(self) -> &'a mut W {
        self.variant(SP_A::PAR_FORCED)
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
#[doc = "Reader of field `BRK`"]
pub type BRK_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `BRK`"]
pub struct BRK_W<'a> {
    w: &'a mut W,
}
impl<'a> BRK_W<'a> {
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
impl R {
    #[doc = "Bits 0:1 - Word Length Select"]
    #[inline(always)]
    pub fn wls(&self) -> WLS_R {
        WLS_R::new((self.bits & 0x03) as u8)
    }
    #[doc = "Bit 2 - Stop Bit"]
    #[inline(always)]
    pub fn stop(&self) -> STOP_R {
        STOP_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Parity Enable"]
    #[inline(always)]
    pub fn pen(&self) -> PEN_R {
        PEN_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Parity Select"]
    #[inline(always)]
    pub fn eps(&self) -> EPS_R {
        EPS_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bit 5 - Stick Parity"]
    #[inline(always)]
    pub fn sp(&self) -> SP_R {
        SP_R::new(((self.bits >> 5) & 0x01) != 0)
    }
    #[doc = "Bit 6 - Set Break"]
    #[inline(always)]
    pub fn brk(&self) -> BRK_R {
        BRK_R::new(((self.bits >> 6) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bits 0:1 - Word Length Select"]
    #[inline(always)]
    pub fn wls(&mut self) -> WLS_W {
        WLS_W { w: self }
    }
    #[doc = "Bit 2 - Stop Bit"]
    #[inline(always)]
    pub fn stop(&mut self) -> STOP_W {
        STOP_W { w: self }
    }
    #[doc = "Bit 3 - Parity Enable"]
    #[inline(always)]
    pub fn pen(&mut self) -> PEN_W {
        PEN_W { w: self }
    }
    #[doc = "Bit 4 - Parity Select"]
    #[inline(always)]
    pub fn eps(&mut self) -> EPS_W {
        EPS_W { w: self }
    }
    #[doc = "Bit 5 - Stick Parity"]
    #[inline(always)]
    pub fn sp(&mut self) -> SP_W {
        SP_W { w: self }
    }
    #[doc = "Bit 6 - Set Break"]
    #[inline(always)]
    pub fn brk(&mut self) -> BRK_W {
        BRK_W { w: self }
    }
}
