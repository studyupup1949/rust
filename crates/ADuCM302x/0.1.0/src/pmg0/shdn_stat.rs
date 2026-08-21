#[doc = "Reader of register SHDN_STAT"]
pub type R = crate::R<u32, super::SHDN_STAT>;
#[doc = "Reader of field `EXTINT0`"]
pub type EXTINT0_R = crate::R<bool, bool>;
#[doc = "Reader of field `EXTINT1`"]
pub type EXTINT1_R = crate::R<bool, bool>;
#[doc = "Reader of field `EXTINT2`"]
pub type EXTINT2_R = crate::R<bool, bool>;
#[doc = "Reader of field `RTC`"]
pub type RTC_R = crate::R<bool, bool>;
impl R {
    #[doc = "Bit 0 - Wakeup by Interrupt from External Interrupt 0"]
    #[inline(always)]
    pub fn extint0(&self) -> EXTINT0_R {
        EXTINT0_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Wakeup by Interrupt from External Interrupt 1"]
    #[inline(always)]
    pub fn extint1(&self) -> EXTINT1_R {
        EXTINT1_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Wakeup by Interrupt from External Interrupt 2"]
    #[inline(always)]
    pub fn extint2(&self) -> EXTINT2_R {
        EXTINT2_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Wakeup by Interrupt from RTC"]
    #[inline(always)]
    pub fn rtc(&self) -> RTC_R {
        RTC_R::new(((self.bits >> 3) & 0x01) != 0)
    }
}
