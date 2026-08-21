#[doc = "Reader of register SCTL"]
pub type R = crate::R<u16, super::SCTL>;
#[doc = "Writer for register SCTL"]
pub type W = crate::W<u16, super::SCTL>;
#[doc = "Register SCTL `reset()`'s with value 0"]
impl crate::ResetValue for super::SCTL {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `SLVEN`"]
pub type SLVEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SLVEN`"]
pub struct SLVEN_W<'a> {
    w: &'a mut W,
}
impl<'a> SLVEN_W<'a> {
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
#[doc = "Reader of field `ADR10EN`"]
pub type ADR10EN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ADR10EN`"]
pub struct ADR10EN_W<'a> {
    w: &'a mut W,
}
impl<'a> ADR10EN_W<'a> {
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
#[doc = "Reader of field `GCEN`"]
pub type GCEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `GCEN`"]
pub struct GCEN_W<'a> {
    w: &'a mut W,
}
impl<'a> GCEN_W<'a> {
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
#[doc = "Reader of field `HGCEN`"]
pub type HGCEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `HGCEN`"]
pub struct HGCEN_W<'a> {
    w: &'a mut W,
}
impl<'a> HGCEN_W<'a> {
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
#[doc = "Write proxy for field `GCSBCLR`"]
pub struct GCSBCLR_W<'a> {
    w: &'a mut W,
}
impl<'a> GCSBCLR_W<'a> {
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
#[doc = "Reader of field `EARLYTXR`"]
pub type EARLYTXR_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `EARLYTXR`"]
pub struct EARLYTXR_W<'a> {
    w: &'a mut W,
}
impl<'a> EARLYTXR_W<'a> {
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
#[doc = "Reader of field `NACK`"]
pub type NACK_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `NACK`"]
pub struct NACK_W<'a> {
    w: &'a mut W,
}
impl<'a> NACK_W<'a> {
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
#[doc = "Reader of field `IENSTOP`"]
pub type IENSTOP_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IENSTOP`"]
pub struct IENSTOP_W<'a> {
    w: &'a mut W,
}
impl<'a> IENSTOP_W<'a> {
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
#[doc = "Reader of field `IENSRX`"]
pub type IENSRX_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IENSRX`"]
pub struct IENSRX_W<'a> {
    w: &'a mut W,
}
impl<'a> IENSRX_W<'a> {
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
#[doc = "Reader of field `IENSTX`"]
pub type IENSTX_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IENSTX`"]
pub struct IENSTX_W<'a> {
    w: &'a mut W,
}
impl<'a> IENSTX_W<'a> {
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
#[doc = "Reader of field `STXDEC`"]
pub type STXDEC_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `STXDEC`"]
pub struct STXDEC_W<'a> {
    w: &'a mut W,
}
impl<'a> STXDEC_W<'a> {
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
#[doc = "Reader of field `IENREPST`"]
pub type IENREPST_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IENREPST`"]
pub struct IENREPST_W<'a> {
    w: &'a mut W,
}
impl<'a> IENREPST_W<'a> {
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
#[doc = "Reader of field `SRXDMA`"]
pub type SRXDMA_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SRXDMA`"]
pub struct SRXDMA_W<'a> {
    w: &'a mut W,
}
impl<'a> SRXDMA_W<'a> {
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
#[doc = "Reader of field `STXDMA`"]
pub type STXDMA_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `STXDMA`"]
pub struct STXDMA_W<'a> {
    w: &'a mut W,
}
impl<'a> STXDMA_W<'a> {
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
impl R {
    #[doc = "Bit 0 - Slave Enable"]
    #[inline(always)]
    pub fn slven(&self) -> SLVEN_R {
        SLVEN_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Enabled 10-bit Addressing"]
    #[inline(always)]
    pub fn adr10en(&self) -> ADR10EN_R {
        ADR10EN_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - General Call Enable"]
    #[inline(always)]
    pub fn gcen(&self) -> GCEN_R {
        GCEN_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Hardware General Call Enable"]
    #[inline(always)]
    pub fn hgcen(&self) -> HGCEN_R {
        HGCEN_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 5 - Early Transmit Request Mode"]
    #[inline(always)]
    pub fn earlytxr(&self) -> EARLYTXR_R {
        EARLYTXR_R::new(((self.bits >> 5) & 0x01) != 0)
    }
    #[doc = "Bit 7 - NACK Next Communication"]
    #[inline(always)]
    pub fn nack(&self) -> NACK_R {
        NACK_R::new(((self.bits >> 7) & 0x01) != 0)
    }
    #[doc = "Bit 8 - Stop Condition Detected Interrupt Enable"]
    #[inline(always)]
    pub fn ienstop(&self) -> IENSTOP_R {
        IENSTOP_R::new(((self.bits >> 8) & 0x01) != 0)
    }
    #[doc = "Bit 9 - Slave Receive Request Interrupt Enable"]
    #[inline(always)]
    pub fn iensrx(&self) -> IENSRX_R {
        IENSRX_R::new(((self.bits >> 9) & 0x01) != 0)
    }
    #[doc = "Bit 10 - Slave Transmit Request Interrupt Enable"]
    #[inline(always)]
    pub fn ienstx(&self) -> IENSTX_R {
        IENSTX_R::new(((self.bits >> 10) & 0x01) != 0)
    }
    #[doc = "Bit 11 - Decrement Slave Tx FIFO Status When a Byte is Txed"]
    #[inline(always)]
    pub fn stxdec(&self) -> STXDEC_R {
        STXDEC_R::new(((self.bits >> 11) & 0x01) != 0)
    }
    #[doc = "Bit 12 - Repeated Start Interrupt Enable"]
    #[inline(always)]
    pub fn ienrepst(&self) -> IENREPST_R {
        IENREPST_R::new(((self.bits >> 12) & 0x01) != 0)
    }
    #[doc = "Bit 13 - Enable Slave Rx DMA Request"]
    #[inline(always)]
    pub fn srxdma(&self) -> SRXDMA_R {
        SRXDMA_R::new(((self.bits >> 13) & 0x01) != 0)
    }
    #[doc = "Bit 14 - Enable Slave Tx DMA Request"]
    #[inline(always)]
    pub fn stxdma(&self) -> STXDMA_R {
        STXDMA_R::new(((self.bits >> 14) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Slave Enable"]
    #[inline(always)]
    pub fn slven(&mut self) -> SLVEN_W {
        SLVEN_W { w: self }
    }
    #[doc = "Bit 1 - Enabled 10-bit Addressing"]
    #[inline(always)]
    pub fn adr10en(&mut self) -> ADR10EN_W {
        ADR10EN_W { w: self }
    }
    #[doc = "Bit 2 - General Call Enable"]
    #[inline(always)]
    pub fn gcen(&mut self) -> GCEN_W {
        GCEN_W { w: self }
    }
    #[doc = "Bit 3 - Hardware General Call Enable"]
    #[inline(always)]
    pub fn hgcen(&mut self) -> HGCEN_W {
        HGCEN_W { w: self }
    }
    #[doc = "Bit 4 - General Call Status Bit Clear"]
    #[inline(always)]
    pub fn gcsbclr(&mut self) -> GCSBCLR_W {
        GCSBCLR_W { w: self }
    }
    #[doc = "Bit 5 - Early Transmit Request Mode"]
    #[inline(always)]
    pub fn earlytxr(&mut self) -> EARLYTXR_W {
        EARLYTXR_W { w: self }
    }
    #[doc = "Bit 7 - NACK Next Communication"]
    #[inline(always)]
    pub fn nack(&mut self) -> NACK_W {
        NACK_W { w: self }
    }
    #[doc = "Bit 8 - Stop Condition Detected Interrupt Enable"]
    #[inline(always)]
    pub fn ienstop(&mut self) -> IENSTOP_W {
        IENSTOP_W { w: self }
    }
    #[doc = "Bit 9 - Slave Receive Request Interrupt Enable"]
    #[inline(always)]
    pub fn iensrx(&mut self) -> IENSRX_W {
        IENSRX_W { w: self }
    }
    #[doc = "Bit 10 - Slave Transmit Request Interrupt Enable"]
    #[inline(always)]
    pub fn ienstx(&mut self) -> IENSTX_W {
        IENSTX_W { w: self }
    }
    #[doc = "Bit 11 - Decrement Slave Tx FIFO Status When a Byte is Txed"]
    #[inline(always)]
    pub fn stxdec(&mut self) -> STXDEC_W {
        STXDEC_W { w: self }
    }
    #[doc = "Bit 12 - Repeated Start Interrupt Enable"]
    #[inline(always)]
    pub fn ienrepst(&mut self) -> IENREPST_W {
        IENREPST_W { w: self }
    }
    #[doc = "Bit 13 - Enable Slave Rx DMA Request"]
    #[inline(always)]
    pub fn srxdma(&mut self) -> SRXDMA_W {
        SRXDMA_W { w: self }
    }
    #[doc = "Bit 14 - Enable Slave Tx DMA Request"]
    #[inline(always)]
    pub fn stxdma(&mut self) -> STXDMA_W {
        STXDMA_W { w: self }
    }
}
