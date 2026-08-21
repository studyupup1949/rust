#[doc = "Reader of register CLR"]
pub type R = crate::R<u32, super::CLR>;
#[doc = "Writer for register CLR"]
pub type W = crate::W<u32, super::CLR>;
#[doc = "Register CLR `reset()`'s with value 0"]
impl crate::ResetValue for super::CLR {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `IRQ0`"]
pub type IRQ0_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IRQ0`"]
pub struct IRQ0_W<'a> {
    w: &'a mut W,
}
impl<'a> IRQ0_W<'a> {
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
#[doc = "Reader of field `IRQ1`"]
pub type IRQ1_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IRQ1`"]
pub struct IRQ1_W<'a> {
    w: &'a mut W,
}
impl<'a> IRQ1_W<'a> {
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
#[doc = "Reader of field `IRQ2`"]
pub type IRQ2_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IRQ2`"]
pub struct IRQ2_W<'a> {
    w: &'a mut W,
}
impl<'a> IRQ2_W<'a> {
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
#[doc = "Reader of field `IRQ3`"]
pub type IRQ3_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IRQ3`"]
pub struct IRQ3_W<'a> {
    w: &'a mut W,
}
impl<'a> IRQ3_W<'a> {
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
#[doc = "Reader of field `UART_RX_CLR`"]
pub type UART_RX_CLR_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `UART_RX_CLR`"]
pub struct UART_RX_CLR_W<'a> {
    w: &'a mut W,
}
impl<'a> UART_RX_CLR_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 5)) | (((value as u32) & 0x01) << 5);
        self.w
    }
}
impl R {
    #[doc = "Bit 0 - External Interrupt 0"]
    #[inline(always)]
    pub fn irq0(&self) -> IRQ0_R {
        IRQ0_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - External Interrupt 1"]
    #[inline(always)]
    pub fn irq1(&self) -> IRQ1_R {
        IRQ1_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - External Interrupt 2"]
    #[inline(always)]
    pub fn irq2(&self) -> IRQ2_R {
        IRQ2_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - External Interrupt 3"]
    #[inline(always)]
    pub fn irq3(&self) -> IRQ3_R {
        IRQ3_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 5 - External Interrupt Clear for UART_RX Wakeup Interrupt"]
    #[inline(always)]
    pub fn uart_rx_clr(&self) -> UART_RX_CLR_R {
        UART_RX_CLR_R::new(((self.bits >> 5) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - External Interrupt 0"]
    #[inline(always)]
    pub fn irq0(&mut self) -> IRQ0_W {
        IRQ0_W { w: self }
    }
    #[doc = "Bit 1 - External Interrupt 1"]
    #[inline(always)]
    pub fn irq1(&mut self) -> IRQ1_W {
        IRQ1_W { w: self }
    }
    #[doc = "Bit 2 - External Interrupt 2"]
    #[inline(always)]
    pub fn irq2(&mut self) -> IRQ2_W {
        IRQ2_W { w: self }
    }
    #[doc = "Bit 3 - External Interrupt 3"]
    #[inline(always)]
    pub fn irq3(&mut self) -> IRQ3_W {
        IRQ3_W { w: self }
    }
    #[doc = "Bit 5 - External Interrupt Clear for UART_RX Wakeup Interrupt"]
    #[inline(always)]
    pub fn uart_rx_clr(&mut self) -> UART_RX_CLR_W {
        UART_RX_CLR_W { w: self }
    }
}
