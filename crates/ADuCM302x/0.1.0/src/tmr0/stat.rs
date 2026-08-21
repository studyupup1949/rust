#[doc = "Reader of register STAT"]
pub type R = crate::R<u16, super::STAT>;
#[doc = "Reader of field `TIMEOUT`"]
pub type TIMEOUT_R = crate::R<bool, bool>;
#[doc = "Reader of field `CAPTURE`"]
pub type CAPTURE_R = crate::R<bool, bool>;
#[doc = "Reader of field `BUSY`"]
pub type BUSY_R = crate::R<bool, bool>;
#[doc = "Reader of field `PDOK`"]
pub type PDOK_R = crate::R<bool, bool>;
#[doc = "Reader of field `CNTRST`"]
pub type CNTRST_R = crate::R<bool, bool>;
impl R {
    #[doc = "Bit 0 - Timeout Event Occurred"]
    #[inline(always)]
    pub fn timeout(&self) -> TIMEOUT_R {
        TIMEOUT_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Capture Event Pending"]
    #[inline(always)]
    pub fn capture(&self) -> CAPTURE_R {
        CAPTURE_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 6 - Timer Busy"]
    #[inline(always)]
    pub fn busy(&self) -> BUSY_R {
        BUSY_R::new(((self.bits >> 6) & 0x01) != 0)
    }
    #[doc = "Bit 7 - Clear Interrupt Register Synchronization"]
    #[inline(always)]
    pub fn pdok(&self) -> PDOK_R {
        PDOK_R::new(((self.bits >> 7) & 0x01) != 0)
    }
    #[doc = "Bit 8 - Counter Reset Occurring"]
    #[inline(always)]
    pub fn cntrst(&self) -> CNTRST_R {
        CNTRST_R::new(((self.bits >> 8) & 0x01) != 0)
    }
}
