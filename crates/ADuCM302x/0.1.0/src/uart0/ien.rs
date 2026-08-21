#[doc = "Reader of register IEN"]
pub type R = crate::R<u16, super::IEN>;
#[doc = "Writer for register IEN"]
pub type W = crate::W<u16, super::IEN>;
#[doc = "Register IEN `reset()`'s with value 0"]
impl crate::ResetValue for super::IEN {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `ERBFI`"]
pub type ERBFI_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ERBFI`"]
pub struct ERBFI_W<'a> {
    w: &'a mut W,
}
impl<'a> ERBFI_W<'a> {
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
#[doc = "Reader of field `ETBEI`"]
pub type ETBEI_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ETBEI`"]
pub struct ETBEI_W<'a> {
    w: &'a mut W,
}
impl<'a> ETBEI_W<'a> {
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
#[doc = "Reader of field `ELSI`"]
pub type ELSI_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ELSI`"]
pub struct ELSI_W<'a> {
    w: &'a mut W,
}
impl<'a> ELSI_W<'a> {
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
#[doc = "Reader of field `EDSSI`"]
pub type EDSSI_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `EDSSI`"]
pub struct EDSSI_W<'a> {
    w: &'a mut W,
}
impl<'a> EDSSI_W<'a> {
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
#[doc = "Reader of field `EDMAT`"]
pub type EDMAT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `EDMAT`"]
pub struct EDMAT_W<'a> {
    w: &'a mut W,
}
impl<'a> EDMAT_W<'a> {
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
#[doc = "Reader of field `EDMAR`"]
pub type EDMAR_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `EDMAR`"]
pub struct EDMAR_W<'a> {
    w: &'a mut W,
}
impl<'a> EDMAR_W<'a> {
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
impl R {
    #[doc = "Bit 0 - Receive Buffer Full Interrupt"]
    #[inline(always)]
    pub fn erbfi(&self) -> ERBFI_R {
        ERBFI_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Transmit Buffer Empty Interrupt"]
    #[inline(always)]
    pub fn etbei(&self) -> ETBEI_R {
        ETBEI_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Rx Status Interrupt"]
    #[inline(always)]
    pub fn elsi(&self) -> ELSI_R {
        ELSI_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Modem Status Interrupt"]
    #[inline(always)]
    pub fn edssi(&self) -> EDSSI_R {
        EDSSI_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - DMA Requests in Transmit Mode"]
    #[inline(always)]
    pub fn edmat(&self) -> EDMAT_R {
        EDMAT_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bit 5 - DMA Requests in Receive Mode"]
    #[inline(always)]
    pub fn edmar(&self) -> EDMAR_R {
        EDMAR_R::new(((self.bits >> 5) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Receive Buffer Full Interrupt"]
    #[inline(always)]
    pub fn erbfi(&mut self) -> ERBFI_W {
        ERBFI_W { w: self }
    }
    #[doc = "Bit 1 - Transmit Buffer Empty Interrupt"]
    #[inline(always)]
    pub fn etbei(&mut self) -> ETBEI_W {
        ETBEI_W { w: self }
    }
    #[doc = "Bit 2 - Rx Status Interrupt"]
    #[inline(always)]
    pub fn elsi(&mut self) -> ELSI_W {
        ELSI_W { w: self }
    }
    #[doc = "Bit 3 - Modem Status Interrupt"]
    #[inline(always)]
    pub fn edssi(&mut self) -> EDSSI_W {
        EDSSI_W { w: self }
    }
    #[doc = "Bit 4 - DMA Requests in Transmit Mode"]
    #[inline(always)]
    pub fn edmat(&mut self) -> EDMAT_W {
        EDMAT_W { w: self }
    }
    #[doc = "Bit 5 - DMA Requests in Receive Mode"]
    #[inline(always)]
    pub fn edmar(&mut self) -> EDMAR_W {
        EDMAR_W { w: self }
    }
}
