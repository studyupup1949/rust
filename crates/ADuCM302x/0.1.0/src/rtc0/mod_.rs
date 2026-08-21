#[doc = "Reader of register MOD"]
pub type R = crate::R<u16, super::MOD>;
#[doc = "Reader of field `CNTMOD60`"]
pub type CNTMOD60_R = crate::R<u8, u8>;
#[doc = "Reader of field `INCR`"]
pub type INCR_R = crate::R<u8, u8>;
#[doc = "Reader of field `TRMBDY`"]
pub type TRMBDY_R = crate::R<bool, bool>;
#[doc = "Reader of field `CNT0_4TOZERO`"]
pub type CNT0_4TOZERO_R = crate::R<u8, u8>;
impl R {
    #[doc = "Bits 0:5 - Modulo-60 Value of the RTC Count: CNT1 and CNT0"]
    #[inline(always)]
    pub fn cntmod60(&self) -> CNTMOD60_R {
        CNTMOD60_R::new((self.bits & 0x3f) as u8)
    }
    #[doc = "Bits 6:9 - Most Recent Increment Value Added to the RTC Count in CNT1 and CNT0"]
    #[inline(always)]
    pub fn incr(&self) -> INCR_R {
        INCR_R::new(((self.bits >> 6) & 0x0f) as u8)
    }
    #[doc = "Bit 10 - Trim Boundary Indicator"]
    #[inline(always)]
    pub fn trmbdy(&self) -> TRMBDY_R {
        TRMBDY_R::new(((self.bits >> 10) & 0x01) != 0)
    }
    #[doc = "Bits 11:15 - Mirror of CNT0\\[4:0\\]"]
    #[inline(always)]
    pub fn cnt0_4tozero(&self) -> CNT0_4TOZERO_R {
        CNT0_4TOZERO_R::new(((self.bits >> 11) & 0x1f) as u8)
    }
}
