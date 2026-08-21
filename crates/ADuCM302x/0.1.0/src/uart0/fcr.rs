#[doc = "Reader of register FCR"]
pub type R = crate::R<u16, super::FCR>;
#[doc = "Writer for register FCR"]
pub type W = crate::W<u16, super::FCR>;
#[doc = "Register FCR `reset()`'s with value 0"]
impl crate::ResetValue for super::FCR {
    type Type = u16;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `FIFOEN`"]
pub type FIFOEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `FIFOEN`"]
pub struct FIFOEN_W<'a> {
    w: &'a mut W,
}
impl<'a> FIFOEN_W<'a> {
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
#[doc = "Write proxy for field `RFCLR`"]
pub struct RFCLR_W<'a> {
    w: &'a mut W,
}
impl<'a> RFCLR_W<'a> {
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
#[doc = "Write proxy for field `TFCLR`"]
pub struct TFCLR_W<'a> {
    w: &'a mut W,
}
impl<'a> TFCLR_W<'a> {
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
#[doc = "FIFO DMA Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FDMAMD_A {
    #[doc = "0: In DMA mode 0, RX DMA request will be asserted whenever there's data in RBR or RX FIFO and de-assert whenever RBR or RX FIFO is empty; TX DMA request will be asserted whenever THR or TX FIFO is empty and de-assert whenever data written to."]
    MODE0 = 0,
    #[doc = "1: in DMA mode 1, RX DMA request will be asserted whenever RX FIFO trig level or time out reached and de-assert thereafter when RX FIFO is empty; TX DMA request will be asserted whenever TX FIFO is empty and de-assert thereafter when TX FIFO is completely filled up full."]
    MODE1 = 1,
}
impl From<FDMAMD_A> for bool {
    #[inline(always)]
    fn from(variant: FDMAMD_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `FDMAMD`"]
pub type FDMAMD_R = crate::R<bool, FDMAMD_A>;
impl FDMAMD_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> FDMAMD_A {
        match self.bits {
            false => FDMAMD_A::MODE0,
            true => FDMAMD_A::MODE1,
        }
    }
    #[doc = "Checks if the value of the field is `MODE0`"]
    #[inline(always)]
    pub fn is_mode0(&self) -> bool {
        *self == FDMAMD_A::MODE0
    }
    #[doc = "Checks if the value of the field is `MODE1`"]
    #[inline(always)]
    pub fn is_mode1(&self) -> bool {
        *self == FDMAMD_A::MODE1
    }
}
#[doc = "Write proxy for field `FDMAMD`"]
pub struct FDMAMD_W<'a> {
    w: &'a mut W,
}
impl<'a> FDMAMD_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: FDMAMD_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "In DMA mode 0, RX DMA request will be asserted whenever there's data in RBR or RX FIFO and de-assert whenever RBR or RX FIFO is empty; TX DMA request will be asserted whenever THR or TX FIFO is empty and de-assert whenever data written to."]
    #[inline(always)]
    pub fn mode0(self) -> &'a mut W {
        self.variant(FDMAMD_A::MODE0)
    }
    #[doc = "in DMA mode 1, RX DMA request will be asserted whenever RX FIFO trig level or time out reached and de-assert thereafter when RX FIFO is empty; TX DMA request will be asserted whenever TX FIFO is empty and de-assert thereafter when TX FIFO is completely filled up full."]
    #[inline(always)]
    pub fn mode1(self) -> &'a mut W {
        self.variant(FDMAMD_A::MODE1)
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
#[doc = "Reader of field `RFTRIG`"]
pub type RFTRIG_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `RFTRIG`"]
pub struct RFTRIG_W<'a> {
    w: &'a mut W,
}
impl<'a> RFTRIG_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 6)) | (((value as u16) & 0x03) << 6);
        self.w
    }
}
impl R {
    #[doc = "Bit 0 - FIFO Enable as to Work in 16550 Mode"]
    #[inline(always)]
    pub fn fifoen(&self) -> FIFOEN_R {
        FIFOEN_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 3 - FIFO DMA Mode"]
    #[inline(always)]
    pub fn fdmamd(&self) -> FDMAMD_R {
        FDMAMD_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bits 6:7 - Rx FIFO Trigger Level"]
    #[inline(always)]
    pub fn rftrig(&self) -> RFTRIG_R {
        RFTRIG_R::new(((self.bits >> 6) & 0x03) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - FIFO Enable as to Work in 16550 Mode"]
    #[inline(always)]
    pub fn fifoen(&mut self) -> FIFOEN_W {
        FIFOEN_W { w: self }
    }
    #[doc = "Bit 1 - Clear Rx FIFO"]
    #[inline(always)]
    pub fn rfclr(&mut self) -> RFCLR_W {
        RFCLR_W { w: self }
    }
    #[doc = "Bit 2 - Clear Tx FIFO"]
    #[inline(always)]
    pub fn tfclr(&mut self) -> TFCLR_W {
        TFCLR_W { w: self }
    }
    #[doc = "Bit 3 - FIFO DMA Mode"]
    #[inline(always)]
    pub fn fdmamd(&mut self) -> FDMAMD_W {
        FDMAMD_W { w: self }
    }
    #[doc = "Bits 6:7 - Rx FIFO Trigger Level"]
    #[inline(always)]
    pub fn rftrig(&mut self) -> RFTRIG_W {
        RFTRIG_W { w: self }
    }
}
