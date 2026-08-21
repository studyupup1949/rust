#[doc = "Reader of register CFG0"]
pub type R = crate::R<u32, super::CFG0>;
#[doc = "Writer for register CFG0"]
pub type W = crate::W<u32, super::CFG0>;
#[doc = "Register CFG0 `reset()`'s with value 0x0020_0000"]
impl crate::ResetValue for super::CFG0 {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x0020_0000
    }
}
#[doc = "Reader of field `IRQ0MDE`"]
pub type IRQ0MDE_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `IRQ0MDE`"]
pub struct IRQ0MDE_W<'a> {
    w: &'a mut W,
}
impl<'a> IRQ0MDE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x07) | ((value as u32) & 0x07);
        self.w
    }
}
#[doc = "Reader of field `IRQ0EN`"]
pub type IRQ0EN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IRQ0EN`"]
pub struct IRQ0EN_W<'a> {
    w: &'a mut W,
}
impl<'a> IRQ0EN_W<'a> {
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
#[doc = "Reader of field `IRQ1MDE`"]
pub type IRQ1MDE_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `IRQ1MDE`"]
pub struct IRQ1MDE_W<'a> {
    w: &'a mut W,
}
impl<'a> IRQ1MDE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x07 << 4)) | (((value as u32) & 0x07) << 4);
        self.w
    }
}
#[doc = "Reader of field `IRQ1EN`"]
pub type IRQ1EN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IRQ1EN`"]
pub struct IRQ1EN_W<'a> {
    w: &'a mut W,
}
impl<'a> IRQ1EN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 7)) | (((value as u32) & 0x01) << 7);
        self.w
    }
}
#[doc = "Reader of field `IRQ2MDE`"]
pub type IRQ2MDE_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `IRQ2MDE`"]
pub struct IRQ2MDE_W<'a> {
    w: &'a mut W,
}
impl<'a> IRQ2MDE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x07 << 8)) | (((value as u32) & 0x07) << 8);
        self.w
    }
}
#[doc = "Reader of field `IRQ2EN`"]
pub type IRQ2EN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IRQ2EN`"]
pub struct IRQ2EN_W<'a> {
    w: &'a mut W,
}
impl<'a> IRQ2EN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 11)) | (((value as u32) & 0x01) << 11);
        self.w
    }
}
#[doc = "Reader of field `IRQ3MDE`"]
pub type IRQ3MDE_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `IRQ3MDE`"]
pub struct IRQ3MDE_W<'a> {
    w: &'a mut W,
}
impl<'a> IRQ3MDE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x07 << 12)) | (((value as u32) & 0x07) << 12);
        self.w
    }
}
#[doc = "Reader of field `IRQ3EN`"]
pub type IRQ3EN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IRQ3EN`"]
pub struct IRQ3EN_W<'a> {
    w: &'a mut W,
}
impl<'a> IRQ3EN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 15)) | (((value as u32) & 0x01) << 15);
        self.w
    }
}
#[doc = "Reader of field `UART_RX_EN`"]
pub type UART_RX_EN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `UART_RX_EN`"]
pub struct UART_RX_EN_W<'a> {
    w: &'a mut W,
}
impl<'a> UART_RX_EN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 20)) | (((value as u32) & 0x01) << 20);
        self.w
    }
}
#[doc = "Reader of field `UART_RX_MDE`"]
pub type UART_RX_MDE_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `UART_RX_MDE`"]
pub struct UART_RX_MDE_W<'a> {
    w: &'a mut W,
}
impl<'a> UART_RX_MDE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x07 << 21)) | (((value as u32) & 0x07) << 21);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:2 - External Interrupt 0 Mode Registers"]
    #[inline(always)]
    pub fn irq0mde(&self) -> IRQ0MDE_R {
        IRQ0MDE_R::new((self.bits & 0x07) as u8)
    }
    #[doc = "Bit 3 - External Interrupt 0 Enable Bit"]
    #[inline(always)]
    pub fn irq0en(&self) -> IRQ0EN_R {
        IRQ0EN_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bits 4:6 - External Interrupt 1 Mode Registers"]
    #[inline(always)]
    pub fn irq1mde(&self) -> IRQ1MDE_R {
        IRQ1MDE_R::new(((self.bits >> 4) & 0x07) as u8)
    }
    #[doc = "Bit 7 - External Interrupt 1 Enable Bit"]
    #[inline(always)]
    pub fn irq1en(&self) -> IRQ1EN_R {
        IRQ1EN_R::new(((self.bits >> 7) & 0x01) != 0)
    }
    #[doc = "Bits 8:10 - External Interrupt 2 Mode Registers"]
    #[inline(always)]
    pub fn irq2mde(&self) -> IRQ2MDE_R {
        IRQ2MDE_R::new(((self.bits >> 8) & 0x07) as u8)
    }
    #[doc = "Bit 11 - External Interrupt 2 Enable Bit"]
    #[inline(always)]
    pub fn irq2en(&self) -> IRQ2EN_R {
        IRQ2EN_R::new(((self.bits >> 11) & 0x01) != 0)
    }
    #[doc = "Bits 12:14 - External Interrupt 3 Mode Registers"]
    #[inline(always)]
    pub fn irq3mde(&self) -> IRQ3MDE_R {
        IRQ3MDE_R::new(((self.bits >> 12) & 0x07) as u8)
    }
    #[doc = "Bit 15 - External Interrupt 3 Enable Bit"]
    #[inline(always)]
    pub fn irq3en(&self) -> IRQ3EN_R {
        IRQ3EN_R::new(((self.bits >> 15) & 0x01) != 0)
    }
    #[doc = "Bit 20 - External Interrupt Enable Bit"]
    #[inline(always)]
    pub fn uart_rx_en(&self) -> UART_RX_EN_R {
        UART_RX_EN_R::new(((self.bits >> 20) & 0x01) != 0)
    }
    #[doc = "Bits 21:23 - External Interrupt Using UART_RX Wakeup Mode Registers"]
    #[inline(always)]
    pub fn uart_rx_mde(&self) -> UART_RX_MDE_R {
        UART_RX_MDE_R::new(((self.bits >> 21) & 0x07) as u8)
    }
}
impl W {
    #[doc = "Bits 0:2 - External Interrupt 0 Mode Registers"]
    #[inline(always)]
    pub fn irq0mde(&mut self) -> IRQ0MDE_W {
        IRQ0MDE_W { w: self }
    }
    #[doc = "Bit 3 - External Interrupt 0 Enable Bit"]
    #[inline(always)]
    pub fn irq0en(&mut self) -> IRQ0EN_W {
        IRQ0EN_W { w: self }
    }
    #[doc = "Bits 4:6 - External Interrupt 1 Mode Registers"]
    #[inline(always)]
    pub fn irq1mde(&mut self) -> IRQ1MDE_W {
        IRQ1MDE_W { w: self }
    }
    #[doc = "Bit 7 - External Interrupt 1 Enable Bit"]
    #[inline(always)]
    pub fn irq1en(&mut self) -> IRQ1EN_W {
        IRQ1EN_W { w: self }
    }
    #[doc = "Bits 8:10 - External Interrupt 2 Mode Registers"]
    #[inline(always)]
    pub fn irq2mde(&mut self) -> IRQ2MDE_W {
        IRQ2MDE_W { w: self }
    }
    #[doc = "Bit 11 - External Interrupt 2 Enable Bit"]
    #[inline(always)]
    pub fn irq2en(&mut self) -> IRQ2EN_W {
        IRQ2EN_W { w: self }
    }
    #[doc = "Bits 12:14 - External Interrupt 3 Mode Registers"]
    #[inline(always)]
    pub fn irq3mde(&mut self) -> IRQ3MDE_W {
        IRQ3MDE_W { w: self }
    }
    #[doc = "Bit 15 - External Interrupt 3 Enable Bit"]
    #[inline(always)]
    pub fn irq3en(&mut self) -> IRQ3EN_W {
        IRQ3EN_W { w: self }
    }
    #[doc = "Bit 20 - External Interrupt Enable Bit"]
    #[inline(always)]
    pub fn uart_rx_en(&mut self) -> UART_RX_EN_W {
        UART_RX_EN_W { w: self }
    }
    #[doc = "Bits 21:23 - External Interrupt Using UART_RX Wakeup Mode Registers"]
    #[inline(always)]
    pub fn uart_rx_mde(&mut self) -> UART_RX_MDE_W {
        UART_RX_MDE_W { w: self }
    }
}
