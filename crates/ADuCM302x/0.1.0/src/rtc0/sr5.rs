#[doc = "Reader of register SR5"]
pub type R = crate::R<u16, super::SR5>;
#[doc = "Reader of field `WPENDSR3`"]
pub type WPENDSR3_R = crate::R<bool, bool>;
#[doc = "Reader of field `WPENDCR2IC`"]
pub type WPENDCR2IC_R = crate::R<bool, bool>;
#[doc = "Reader of field `WPENDCR3SS`"]
pub type WPENDCR3SS_R = crate::R<bool, bool>;
#[doc = "Reader of field `WPENDCR4SS`"]
pub type WPENDCR4SS_R = crate::R<bool, bool>;
#[doc = "Reader of field `WPENDSSMSK`"]
pub type WPENDSSMSK_R = crate::R<bool, bool>;
#[doc = "Reader of field `WPENDSS1ARL`"]
pub type WPENDSS1ARL_R = crate::R<bool, bool>;
#[doc = "Reader of field `WPENDSS1`"]
pub type WPENDSS1_R = crate::R<bool, bool>;
#[doc = "Reader of field `RPENDIC0`"]
pub type RPENDIC0_R = crate::R<bool, bool>;
#[doc = "Reader of field `RPENDIC2`"]
pub type RPENDIC2_R = crate::R<bool, bool>;
#[doc = "Reader of field `RPENDIC3`"]
pub type RPENDIC3_R = crate::R<bool, bool>;
#[doc = "Reader of field `RPENDIC4`"]
pub type RPENDIC4_R = crate::R<bool, bool>;
impl R {
    #[doc = "Bit 0 - Pending Status of Posted Clearances of Interrupt Sources in RTC Status 3 Register"]
    #[inline(always)]
    pub fn wpendsr3(&self) -> WPENDSR3_R {
        WPENDSR3_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Pending Status of Posted Writes to RTC Control 2 for Configuring Input Capture Channels Register"]
    #[inline(always)]
    pub fn wpendcr2ic(&self) -> WPENDCR2IC_R {
        WPENDCR2IC_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Pending Status of Posted Writes to RTC Control 3 for Configuring SensorStrobe Channel Register"]
    #[inline(always)]
    pub fn wpendcr3ss(&self) -> WPENDCR3SS_R {
        WPENDCR3SS_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Pending Status of Posted Writes to RTC Control 4 for Configuring SensorStrobe Channel Register"]
    #[inline(always)]
    pub fn wpendcr4ss(&self) -> WPENDCR4SS_R {
        WPENDCR4SS_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Pending Status of Posted Writes to RTC Masks for SensorStrobe Channel Register"]
    #[inline(always)]
    pub fn wpendssmsk(&self) -> WPENDSSMSK_R {
        WPENDSSMSK_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bit 5 - Pending Status of Posted Writes to RTC Auto-Reload for SensorStrobe Channel 1 Register"]
    #[inline(always)]
    pub fn wpendss1arl(&self) -> WPENDSS1ARL_R {
        WPENDSS1ARL_R::new(((self.bits >> 5) & 0x01) != 0)
    }
    #[doc = "Bit 6 - Pending Status of Posted Writes to SensorStrobe Channel 1"]
    #[inline(always)]
    pub fn wpendss1(&self) -> WPENDSS1_R {
        WPENDSS1_R::new(((self.bits >> 6) & 0x01) != 0)
    }
    #[doc = "Bit 10 - Pending Status of Posted Reads of Input Capture Channel 0"]
    #[inline(always)]
    pub fn rpendic0(&self) -> RPENDIC0_R {
        RPENDIC0_R::new(((self.bits >> 10) & 0x01) != 0)
    }
    #[doc = "Bit 12 - Pending Status of Posted Reads of IC2"]
    #[inline(always)]
    pub fn rpendic2(&self) -> RPENDIC2_R {
        RPENDIC2_R::new(((self.bits >> 12) & 0x01) != 0)
    }
    #[doc = "Bit 13 - Pending Status of Posted Reads of IC3"]
    #[inline(always)]
    pub fn rpendic3(&self) -> RPENDIC3_R {
        RPENDIC3_R::new(((self.bits >> 13) & 0x01) != 0)
    }
    #[doc = "Bit 14 - Pending Status of Posted Reads of IC4"]
    #[inline(always)]
    pub fn rpendic4(&self) -> RPENDIC4_R {
        RPENDIC4_R::new(((self.bits >> 14) & 0x01) != 0)
    }
}
