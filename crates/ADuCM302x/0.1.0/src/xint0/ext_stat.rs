#[doc = "Reader of register EXT_STAT"]
pub type R = crate::R<u32, super::EXT_STAT>;
#[doc = "Reader of field `STAT_EXTINT0`"]
pub type STAT_EXTINT0_R = crate::R<bool, bool>;
#[doc = "Reader of field `STAT_EXTINT1`"]
pub type STAT_EXTINT1_R = crate::R<bool, bool>;
#[doc = "Reader of field `STAT_EXTINT2`"]
pub type STAT_EXTINT2_R = crate::R<bool, bool>;
#[doc = "Reader of field `STAT_EXTINT3`"]
pub type STAT_EXTINT3_R = crate::R<bool, bool>;
#[doc = "Reader of field `STAT_UART_RXWKUP`"]
pub type STAT_UART_RXWKUP_R = crate::R<bool, bool>;
impl R {
    #[doc = "Bit 0 - Interrupt Status Bit for External Interrupt 0"]
    #[inline(always)]
    pub fn stat_extint0(&self) -> STAT_EXTINT0_R {
        STAT_EXTINT0_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Interrupt Status Bit for External Interrupt 1"]
    #[inline(always)]
    pub fn stat_extint1(&self) -> STAT_EXTINT1_R {
        STAT_EXTINT1_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Interrupt Status Bit for External Interrupt 2"]
    #[inline(always)]
    pub fn stat_extint2(&self) -> STAT_EXTINT2_R {
        STAT_EXTINT2_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Interrupt Status Bit for External Interrupt 3"]
    #[inline(always)]
    pub fn stat_extint3(&self) -> STAT_EXTINT3_R {
        STAT_EXTINT3_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 5 - Interrupt Status Bit for UART RX Wakeup Interrupt"]
    #[inline(always)]
    pub fn stat_uart_rxwkup(&self) -> STAT_UART_RXWKUP_R {
        STAT_UART_RXWKUP_R::new(((self.bits >> 5) & 0x01) != 0)
    }
}
