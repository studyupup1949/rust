#[doc = "Reader of register SR4"]
pub type R = crate::R<u16, super::SR4>;
#[doc = "Reader of field `WSYNCSR3`"]
pub type WSYNCSR3_R = crate::R<bool, bool>;
#[doc = "Reader of field `WSYNCCR2IC`"]
pub type WSYNCCR2IC_R = crate::R<bool, bool>;
#[doc = "Reader of field `WSYNCCR3SS`"]
pub type WSYNCCR3SS_R = crate::R<bool, bool>;
#[doc = "Reader of field `WSYNCCR4SS`"]
pub type WSYNCCR4SS_R = crate::R<bool, bool>;
#[doc = "Reader of field `WSYNCSSMSK`"]
pub type WSYNCSSMSK_R = crate::R<bool, bool>;
#[doc = "Reader of field `WSYNCSS1ARL`"]
pub type WSYNCSS1ARL_R = crate::R<bool, bool>;
#[doc = "Reader of field `WSYNCSS1`"]
pub type WSYNCSS1_R = crate::R<bool, bool>;
#[doc = "Reader of field `RSYNCIC0`"]
pub type RSYNCIC0_R = crate::R<bool, bool>;
#[doc = "Reader of field `RSYNCIC2`"]
pub type RSYNCIC2_R = crate::R<bool, bool>;
#[doc = "Reader of field `RSYNCIC3`"]
pub type RSYNCIC3_R = crate::R<bool, bool>;
#[doc = "Reader of field `RSYNCIC4`"]
pub type RSYNCIC4_R = crate::R<bool, bool>;
impl R {
    #[doc = "Bit 0 - Synchronisation Status of Posted Writes to SR3"]
    #[inline(always)]
    pub fn wsyncsr3(&self) -> WSYNCSR3_R {
        WSYNCSR3_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Synchronization Status of Posted Writes to RTC Control 2 for Configuring Input Capture Channels Register"]
    #[inline(always)]
    pub fn wsynccr2ic(&self) -> WSYNCCR2IC_R {
        WSYNCCR2IC_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Synchronization Status of Posted Writes to RTC Control 3 for Configuring SensorStrobe Channel Register"]
    #[inline(always)]
    pub fn wsynccr3ss(&self) -> WSYNCCR3SS_R {
        WSYNCCR3SS_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Synchronization Status of Posted Writes to RTC Control 4 for Configuring SensorStrobe Channel Register"]
    #[inline(always)]
    pub fn wsynccr4ss(&self) -> WSYNCCR4SS_R {
        WSYNCCR4SS_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Synchronization Status of Posted Writes to Masks for SensorStrobe Channel Register"]
    #[inline(always)]
    pub fn wsyncssmsk(&self) -> WSYNCSSMSK_R {
        WSYNCSSMSK_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bit 5 - Synchronization Status of Posted Writes to RTC Auto-Reload for SensorStrobe Channel 1 Register"]
    #[inline(always)]
    pub fn wsyncss1arl(&self) -> WSYNCSS1ARL_R {
        WSYNCSS1ARL_R::new(((self.bits >> 5) & 0x01) != 0)
    }
    #[doc = "Bit 6 - Synchronization Status of Posted Writes to SensorStrobe Channel 1"]
    #[inline(always)]
    pub fn wsyncss1(&self) -> WSYNCSS1_R {
        WSYNCSS1_R::new(((self.bits >> 6) & 0x01) != 0)
    }
    #[doc = "Bit 10 - Synchronization Status of Posted Reads of RTC Input Channel 0"]
    #[inline(always)]
    pub fn rsyncic0(&self) -> RSYNCIC0_R {
        RSYNCIC0_R::new(((self.bits >> 10) & 0x01) != 0)
    }
    #[doc = "Bit 12 - Synchronization Status of Posted Reads of RTC Input Channel 2"]
    #[inline(always)]
    pub fn rsyncic2(&self) -> RSYNCIC2_R {
        RSYNCIC2_R::new(((self.bits >> 12) & 0x01) != 0)
    }
    #[doc = "Bit 13 - Synchronization Status of Posted Reads of RTC Input Channel 3"]
    #[inline(always)]
    pub fn rsyncic3(&self) -> RSYNCIC3_R {
        RSYNCIC3_R::new(((self.bits >> 13) & 0x01) != 0)
    }
    #[doc = "Bit 14 - Synchronization Status of Posted Reads of RTC Input Channel 4"]
    #[inline(always)]
    pub fn rsyncic4(&self) -> RSYNCIC4_R {
        RSYNCIC4_R::new(((self.bits >> 14) & 0x01) != 0)
    }
}
