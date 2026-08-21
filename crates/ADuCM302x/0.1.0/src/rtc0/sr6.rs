#[doc = "Reader of register SR6"]
pub type R = crate::R<u16, super::SR6>;
#[doc = "Reader of field `IC0UNR`"]
pub type IC0UNR_R = crate::R<bool, bool>;
#[doc = "Reader of field `IC2UNR`"]
pub type IC2UNR_R = crate::R<bool, bool>;
#[doc = "Reader of field `IC3UNR`"]
pub type IC3UNR_R = crate::R<bool, bool>;
#[doc = "Reader of field `IC4UNR`"]
pub type IC4UNR_R = crate::R<bool, bool>;
#[doc = "Reader of field `IC0SNAP`"]
pub type IC0SNAP_R = crate::R<bool, bool>;
#[doc = "Reader of field `FRZCNTPTR`"]
pub type FRZCNTPTR_R = crate::R<u8, u8>;
impl R {
    #[doc = "Bit 0 - Sticky Unread Status of the Input Capture Channel 0"]
    #[inline(always)]
    pub fn ic0unr(&self) -> IC0UNR_R {
        IC0UNR_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 2 - Sticky Unread Status of the Input Capture Channel 2"]
    #[inline(always)]
    pub fn ic2unr(&self) -> IC2UNR_R {
        IC2UNR_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Sticky Unread Status of the Input Capture Channel 3"]
    #[inline(always)]
    pub fn ic3unr(&self) -> IC3UNR_R {
        IC3UNR_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Sticky Unread Status of the Input Capture Channel 4"]
    #[inline(always)]
    pub fn ic4unr(&self) -> IC4UNR_R {
        IC4UNR_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bit 8 - Confirmation That RTC Snapshot 0, 1, 2 Registers Reflect the Value of Input-Capture Channel RTC Input Capture Channel 0"]
    #[inline(always)]
    pub fn ic0snap(&self) -> IC0SNAP_R {
        IC0SNAP_R::new(((self.bits >> 8) & 0x01) != 0)
    }
    #[doc = "Bits 9:10 - Pointer for the Triple-Read Sequence of FRZCNT"]
    #[inline(always)]
    pub fn frzcntptr(&self) -> FRZCNTPTR_R {
        FRZCNTPTR_R::new(((self.bits >> 9) & 0x03) as u8)
    }
}
