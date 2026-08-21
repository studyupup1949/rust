#[doc = "Reader of register IEN_B"]
pub type R = crate::R<u32, super::IEN_B>;
#[doc = "Writer for register IEN_B"]
pub type W = crate::W<u32, super::IEN_B>;
#[doc = "Register IEN_B `reset()`'s with value 0"]
impl crate::ResetValue for super::IEN_B {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Transmit Finish Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TF_A {
    #[doc = "0: Transfer Finish Interrupt is disabled"]
    CTL_TXFIN_DIS = 0,
    #[doc = "1: Transfer Finish Interrupt is Enabled"]
    CTL_TXFIN_EN = 1,
}
impl From<TF_A> for bool {
    #[inline(always)]
    fn from(variant: TF_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `TF`"]
pub type TF_R = crate::R<bool, TF_A>;
impl TF_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> TF_A {
        match self.bits {
            false => TF_A::CTL_TXFIN_DIS,
            true => TF_A::CTL_TXFIN_EN,
        }
    }
    #[doc = "Checks if the value of the field is `CTL_TXFIN_DIS`"]
    #[inline(always)]
    pub fn is_ctl_txfin_dis(&self) -> bool {
        *self == TF_A::CTL_TXFIN_DIS
    }
    #[doc = "Checks if the value of the field is `CTL_TXFIN_EN`"]
    #[inline(always)]
    pub fn is_ctl_txfin_en(&self) -> bool {
        *self == TF_A::CTL_TXFIN_EN
    }
}
#[doc = "Write proxy for field `TF`"]
pub struct TF_W<'a> {
    w: &'a mut W,
}
impl<'a> TF_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: TF_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Transfer Finish Interrupt is disabled"]
    #[inline(always)]
    pub fn ctl_txfin_dis(self) -> &'a mut W {
        self.variant(TF_A::CTL_TXFIN_DIS)
    }
    #[doc = "Transfer Finish Interrupt is Enabled"]
    #[inline(always)]
    pub fn ctl_txfin_en(self) -> &'a mut W {
        self.variant(TF_A::CTL_TXFIN_EN)
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
#[doc = "Reader of field `DERRMSK`"]
pub type DERRMSK_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `DERRMSK`"]
pub struct DERRMSK_W<'a> {
    w: &'a mut W,
}
impl<'a> DERRMSK_W<'a> {
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
#[doc = "Reader of field `FSERRMSK`"]
pub type FSERRMSK_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `FSERRMSK`"]
pub struct FSERRMSK_W<'a> {
    w: &'a mut W,
}
impl<'a> FSERRMSK_W<'a> {
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
#[doc = "Reader of field `DATA`"]
pub type DATA_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `DATA`"]
pub struct DATA_W<'a> {
    w: &'a mut W,
}
impl<'a> DATA_W<'a> {
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
#[doc = "Reader of field `SYSDATERR`"]
pub type SYSDATERR_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SYSDATERR`"]
pub struct SYSDATERR_W<'a> {
    w: &'a mut W,
}
impl<'a> SYSDATERR_W<'a> {
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
impl R {
    #[doc = "Bit 0 - Transmit Finish Interrupt Enable"]
    #[inline(always)]
    pub fn tf(&self) -> TF_R {
        TF_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Data Error (Interrupt) Mask"]
    #[inline(always)]
    pub fn derrmsk(&self) -> DERRMSK_R {
        DERRMSK_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Frame Sync Error (Interrupt) Mask"]
    #[inline(always)]
    pub fn fserrmsk(&self) -> FSERRMSK_R {
        FSERRMSK_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Data Request Interrupt to the Core"]
    #[inline(always)]
    pub fn data(&self) -> DATA_R {
        DATA_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Data Error for System Writes or Reads"]
    #[inline(always)]
    pub fn sysdaterr(&self) -> SYSDATERR_R {
        SYSDATERR_R::new(((self.bits >> 4) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Transmit Finish Interrupt Enable"]
    #[inline(always)]
    pub fn tf(&mut self) -> TF_W {
        TF_W { w: self }
    }
    #[doc = "Bit 1 - Data Error (Interrupt) Mask"]
    #[inline(always)]
    pub fn derrmsk(&mut self) -> DERRMSK_W {
        DERRMSK_W { w: self }
    }
    #[doc = "Bit 2 - Frame Sync Error (Interrupt) Mask"]
    #[inline(always)]
    pub fn fserrmsk(&mut self) -> FSERRMSK_W {
        FSERRMSK_W { w: self }
    }
    #[doc = "Bit 3 - Data Request Interrupt to the Core"]
    #[inline(always)]
    pub fn data(&mut self) -> DATA_W {
        DATA_W { w: self }
    }
    #[doc = "Bit 4 - Data Error for System Writes or Reads"]
    #[inline(always)]
    pub fn sysdaterr(&mut self) -> SYSDATERR_W {
        SYSDATERR_W { w: self }
    }
}
