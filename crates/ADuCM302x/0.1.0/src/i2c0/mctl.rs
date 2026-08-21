#[doc = "Reader of register MCTL"]
pub type R = crate::R<u16, super::MCTL>;
#[doc = "Writer for register MCTL"]
pub type W = crate::W<u16, super::MCTL>;
#[doc = "Register MCTL `reset()`'s with value 0"]
impl crate::ResetValue for super::MCTL {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `MASEN`"]
pub type MASEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `MASEN`"]
pub struct MASEN_W<'a> {
    w: &'a mut W,
}
impl<'a> MASEN_W<'a> {
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
#[doc = "Reader of field `COMPLETE`"]
pub type COMPLETE_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `COMPLETE`"]
pub struct COMPLETE_W<'a> {
    w: &'a mut W,
}
impl<'a> COMPLETE_W<'a> {
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
#[doc = "Reader of field `LOOPBACK`"]
pub type LOOPBACK_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `LOOPBACK`"]
pub struct LOOPBACK_W<'a> {
    w: &'a mut W,
}
impl<'a> LOOPBACK_W<'a> {
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
#[doc = "Reader of field `STRETCHSCL`"]
pub type STRETCHSCL_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `STRETCHSCL`"]
pub struct STRETCHSCL_W<'a> {
    w: &'a mut W,
}
impl<'a> STRETCHSCL_W<'a> {
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
#[doc = "Reader of field `IENMRX`"]
pub type IENMRX_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IENMRX`"]
pub struct IENMRX_W<'a> {
    w: &'a mut W,
}
impl<'a> IENMRX_W<'a> {
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
#[doc = "Reader of field `IENMTX`"]
pub type IENMTX_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IENMTX`"]
pub struct IENMTX_W<'a> {
    w: &'a mut W,
}
impl<'a> IENMTX_W<'a> {
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
#[doc = "Reader of field `IENALOST`"]
pub type IENALOST_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IENALOST`"]
pub struct IENALOST_W<'a> {
    w: &'a mut W,
}
impl<'a> IENALOST_W<'a> {
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
#[doc = "Reader of field `IENACK`"]
pub type IENACK_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IENACK`"]
pub struct IENACK_W<'a> {
    w: &'a mut W,
}
impl<'a> IENACK_W<'a> {
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
#[doc = "Reader of field `IENCMP`"]
pub type IENCMP_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IENCMP`"]
pub struct IENCMP_W<'a> {
    w: &'a mut W,
}
impl<'a> IENCMP_W<'a> {
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
#[doc = "Reader of field `MXMITDEC`"]
pub type MXMITDEC_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `MXMITDEC`"]
pub struct MXMITDEC_W<'a> {
    w: &'a mut W,
}
impl<'a> MXMITDEC_W<'a> {
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
#[doc = "Write proxy for field `MRXDMA`"]
pub struct MRXDMA_W<'a> {
    w: &'a mut W,
}
impl<'a> MRXDMA_W<'a> {
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
#[doc = "Write proxy for field `MTXDMA`"]
pub struct MTXDMA_W<'a> {
    w: &'a mut W,
}
impl<'a> MTXDMA_W<'a> {
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
#[doc = "Reader of field `BUSCLR`"]
pub type BUSCLR_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `BUSCLR`"]
pub struct BUSCLR_W<'a> {
    w: &'a mut W,
}
impl<'a> BUSCLR_W<'a> {
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
#[doc = "Reader of field `STOPBUSCLR`"]
pub type STOPBUSCLR_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `STOPBUSCLR`"]
pub struct STOPBUSCLR_W<'a> {
    w: &'a mut W,
}
impl<'a> STOPBUSCLR_W<'a> {
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
impl R {
    #[doc = "Bit 0 - Master Enable"]
    #[inline(always)]
    pub fn masen(&self) -> MASEN_R {
        MASEN_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Start Back-off Disable"]
    #[inline(always)]
    pub fn complete(&self) -> COMPLETE_R {
        COMPLETE_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Internal Loopback Enable"]
    #[inline(always)]
    pub fn loopback(&self) -> LOOPBACK_R {
        LOOPBACK_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Stretch SCL Enable"]
    #[inline(always)]
    pub fn stretchscl(&self) -> STRETCHSCL_R {
        STRETCHSCL_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Receive Request Interrupt Enable"]
    #[inline(always)]
    pub fn ienmrx(&self) -> IENMRX_R {
        IENMRX_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bit 5 - Transmit Request Interrupt Enable"]
    #[inline(always)]
    pub fn ienmtx(&self) -> IENMTX_R {
        IENMTX_R::new(((self.bits >> 5) & 0x01) != 0)
    }
    #[doc = "Bit 6 - Arbitration Lost Interrupt Enable"]
    #[inline(always)]
    pub fn ienalost(&self) -> IENALOST_R {
        IENALOST_R::new(((self.bits >> 6) & 0x01) != 0)
    }
    #[doc = "Bit 7 - ACK Not Received Interrupt Enable"]
    #[inline(always)]
    pub fn ienack(&self) -> IENACK_R {
        IENACK_R::new(((self.bits >> 7) & 0x01) != 0)
    }
    #[doc = "Bit 8 - Transaction Completed (or Stop Detected) Interrupt Enable"]
    #[inline(always)]
    pub fn iencmp(&self) -> IENCMP_R {
        IENCMP_R::new(((self.bits >> 8) & 0x01) != 0)
    }
    #[doc = "Bit 9 - Decrement Master Tx FIFO Status When a Byte Txed"]
    #[inline(always)]
    pub fn mxmitdec(&self) -> MXMITDEC_R {
        MXMITDEC_R::new(((self.bits >> 9) & 0x01) != 0)
    }
    #[doc = "Bit 12 - Bus-Clear Enable"]
    #[inline(always)]
    pub fn busclr(&self) -> BUSCLR_R {
        BUSCLR_R::new(((self.bits >> 12) & 0x01) != 0)
    }
    #[doc = "Bit 13 - Prestop Bus Clear"]
    #[inline(always)]
    pub fn stopbusclr(&self) -> STOPBUSCLR_R {
        STOPBUSCLR_R::new(((self.bits >> 13) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Master Enable"]
    #[inline(always)]
    pub fn masen(&mut self) -> MASEN_W {
        MASEN_W { w: self }
    }
    #[doc = "Bit 1 - Start Back-off Disable"]
    #[inline(always)]
    pub fn complete(&mut self) -> COMPLETE_W {
        COMPLETE_W { w: self }
    }
    #[doc = "Bit 2 - Internal Loopback Enable"]
    #[inline(always)]
    pub fn loopback(&mut self) -> LOOPBACK_W {
        LOOPBACK_W { w: self }
    }
    #[doc = "Bit 3 - Stretch SCL Enable"]
    #[inline(always)]
    pub fn stretchscl(&mut self) -> STRETCHSCL_W {
        STRETCHSCL_W { w: self }
    }
    #[doc = "Bit 4 - Receive Request Interrupt Enable"]
    #[inline(always)]
    pub fn ienmrx(&mut self) -> IENMRX_W {
        IENMRX_W { w: self }
    }
    #[doc = "Bit 5 - Transmit Request Interrupt Enable"]
    #[inline(always)]
    pub fn ienmtx(&mut self) -> IENMTX_W {
        IENMTX_W { w: self }
    }
    #[doc = "Bit 6 - Arbitration Lost Interrupt Enable"]
    #[inline(always)]
    pub fn ienalost(&mut self) -> IENALOST_W {
        IENALOST_W { w: self }
    }
    #[doc = "Bit 7 - ACK Not Received Interrupt Enable"]
    #[inline(always)]
    pub fn ienack(&mut self) -> IENACK_W {
        IENACK_W { w: self }
    }
    #[doc = "Bit 8 - Transaction Completed (or Stop Detected) Interrupt Enable"]
    #[inline(always)]
    pub fn iencmp(&mut self) -> IENCMP_W {
        IENCMP_W { w: self }
    }
    #[doc = "Bit 9 - Decrement Master Tx FIFO Status When a Byte Txed"]
    #[inline(always)]
    pub fn mxmitdec(&mut self) -> MXMITDEC_W {
        MXMITDEC_W { w: self }
    }
    #[doc = "Bit 10 - Enable Master Rx DMA Request"]
    #[inline(always)]
    pub fn mrxdma(&mut self) -> MRXDMA_W {
        MRXDMA_W { w: self }
    }
    #[doc = "Bit 11 - Enable Master Tx DMA Request"]
    #[inline(always)]
    pub fn mtxdma(&mut self) -> MTXDMA_W {
        MTXDMA_W { w: self }
    }
    #[doc = "Bit 12 - Bus-Clear Enable"]
    #[inline(always)]
    pub fn busclr(&mut self) -> BUSCLR_W {
        BUSCLR_W { w: self }
    }
    #[doc = "Bit 13 - Prestop Bus Clear"]
    #[inline(always)]
    pub fn stopbusclr(&mut self) -> STOPBUSCLR_W {
        STOPBUSCLR_W { w: self }
    }
}
