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
#[doc = "Reader of field `IRQMODE`"]
pub type IRQMODE_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `IRQMODE`"]
pub struct IRQMODE_W<'a> {
    w: &'a mut W,
}
impl<'a> IRQMODE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x07) | ((value as u16) & 0x07);
        self.w
    }
}
#[doc = "Reader of field `CS`"]
pub type CS_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `CS`"]
pub struct CS_W<'a> {
    w: &'a mut W,
}
impl<'a> CS_W<'a> {
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
#[doc = "Reader of field `TXUNDR`"]
pub type TXUNDR_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `TXUNDR`"]
pub struct TXUNDR_W<'a> {
    w: &'a mut W,
}
impl<'a> TXUNDR_W<'a> {
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
#[doc = "Reader of field `RXOVR`"]
pub type RXOVR_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `RXOVR`"]
pub struct RXOVR_W<'a> {
    w: &'a mut W,
}
impl<'a> RXOVR_W<'a> {
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
#[doc = "Reader of field `RDY`"]
pub type RDY_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `RDY`"]
pub struct RDY_W<'a> {
    w: &'a mut W,
}
impl<'a> RDY_W<'a> {
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
#[doc = "Reader of field `TXDONE`"]
pub type TXDONE_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `TXDONE`"]
pub struct TXDONE_W<'a> {
    w: &'a mut W,
}
impl<'a> TXDONE_W<'a> {
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
#[doc = "Reader of field `XFRDONE`"]
pub type XFRDONE_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `XFRDONE`"]
pub struct XFRDONE_W<'a> {
    w: &'a mut W,
}
impl<'a> XFRDONE_W<'a> {
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
#[doc = "Reader of field `TXEMPTY`"]
pub type TXEMPTY_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `TXEMPTY`"]
pub struct TXEMPTY_W<'a> {
    w: &'a mut W,
}
impl<'a> TXEMPTY_W<'a> {
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
    #[doc = "Bits 0:2 - SPI IRQ Mode Bits"]
    #[inline(always)]
    pub fn irqmode(&self) -> IRQMODE_R {
        IRQMODE_R::new((self.bits & 0x07) as u8)
    }
    #[doc = "Bit 8 - Enable Interrupt on Every CS Edge in Slave CON Mode"]
    #[inline(always)]
    pub fn cs(&self) -> CS_R {
        CS_R::new(((self.bits >> 8) & 0x01) != 0)
    }
    #[doc = "Bit 9 - Tx Underflow Interrupt Enable"]
    #[inline(always)]
    pub fn txundr(&self) -> TXUNDR_R {
        TXUNDR_R::new(((self.bits >> 9) & 0x01) != 0)
    }
    #[doc = "Bit 10 - Rx Overflow Interrupt Enable"]
    #[inline(always)]
    pub fn rxovr(&self) -> RXOVR_R {
        RXOVR_R::new(((self.bits >> 10) & 0x01) != 0)
    }
    #[doc = "Bit 11 - Ready Signal Edge Interrupt Enable"]
    #[inline(always)]
    pub fn rdy(&self) -> RDY_R {
        RDY_R::new(((self.bits >> 11) & 0x01) != 0)
    }
    #[doc = "Bit 12 - SPI Transmit Done Interrupt Enable"]
    #[inline(always)]
    pub fn txdone(&self) -> TXDONE_R {
        TXDONE_R::new(((self.bits >> 12) & 0x01) != 0)
    }
    #[doc = "Bit 13 - SPI Transfer Completion Interrupt Enable"]
    #[inline(always)]
    pub fn xfrdone(&self) -> XFRDONE_R {
        XFRDONE_R::new(((self.bits >> 13) & 0x01) != 0)
    }
    #[doc = "Bit 14 - Tx FIFO Empty Interrupt Enable"]
    #[inline(always)]
    pub fn txempty(&self) -> TXEMPTY_R {
        TXEMPTY_R::new(((self.bits >> 14) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bits 0:2 - SPI IRQ Mode Bits"]
    #[inline(always)]
    pub fn irqmode(&mut self) -> IRQMODE_W {
        IRQMODE_W { w: self }
    }
    #[doc = "Bit 8 - Enable Interrupt on Every CS Edge in Slave CON Mode"]
    #[inline(always)]
    pub fn cs(&mut self) -> CS_W {
        CS_W { w: self }
    }
    #[doc = "Bit 9 - Tx Underflow Interrupt Enable"]
    #[inline(always)]
    pub fn txundr(&mut self) -> TXUNDR_W {
        TXUNDR_W { w: self }
    }
    #[doc = "Bit 10 - Rx Overflow Interrupt Enable"]
    #[inline(always)]
    pub fn rxovr(&mut self) -> RXOVR_W {
        RXOVR_W { w: self }
    }
    #[doc = "Bit 11 - Ready Signal Edge Interrupt Enable"]
    #[inline(always)]
    pub fn rdy(&mut self) -> RDY_W {
        RDY_W { w: self }
    }
    #[doc = "Bit 12 - SPI Transmit Done Interrupt Enable"]
    #[inline(always)]
    pub fn txdone(&mut self) -> TXDONE_W {
        TXDONE_W { w: self }
    }
    #[doc = "Bit 13 - SPI Transfer Completion Interrupt Enable"]
    #[inline(always)]
    pub fn xfrdone(&mut self) -> XFRDONE_W {
        XFRDONE_W { w: self }
    }
    #[doc = "Bit 14 - Tx FIFO Empty Interrupt Enable"]
    #[inline(always)]
    pub fn txempty(&mut self) -> TXEMPTY_W {
        TXEMPTY_W { w: self }
    }
}
