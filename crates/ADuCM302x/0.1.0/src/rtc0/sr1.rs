#[doc = "Reader of register SR1"]
pub type R = crate::R<u16, super::SR1>;
#[doc = "Reader of field `WPNDCR0`"]
pub type WPNDCR0_R = crate::R<bool, bool>;
#[doc = "Reader of field `WPNDSR0`"]
pub type WPNDSR0_R = crate::R<bool, bool>;
#[doc = "Reader of field `WPNDCNT0`"]
pub type WPNDCNT0_R = crate::R<bool, bool>;
#[doc = "Reader of field `WPNDCNT1`"]
pub type WPNDCNT1_R = crate::R<bool, bool>;
#[doc = "Reader of field `WPNDALM0`"]
pub type WPNDALM0_R = crate::R<bool, bool>;
#[doc = "Reader of field `WPNDALM1`"]
pub type WPNDALM1_R = crate::R<bool, bool>;
#[doc = "Reader of field `WPNDTRM`"]
pub type WPNDTRM_R = crate::R<bool, bool>;
impl R {
    #[doc = "Bit 7 - Pending Status of Posted Writes to CR0"]
    #[inline(always)]
    pub fn wpndcr0(&self) -> WPNDCR0_R {
        WPNDCR0_R::new(((self.bits >> 7) & 0x01) != 0)
    }
    #[doc = "Bit 8 - Pending Status of Posted Clearances of Interrupt Sources in SR0"]
    #[inline(always)]
    pub fn wpndsr0(&self) -> WPNDSR0_R {
        WPNDSR0_R::new(((self.bits >> 8) & 0x01) != 0)
    }
    #[doc = "Bit 9 - Pending Status of Posted Writes to CNT0"]
    #[inline(always)]
    pub fn wpndcnt0(&self) -> WPNDCNT0_R {
        WPNDCNT0_R::new(((self.bits >> 9) & 0x01) != 0)
    }
    #[doc = "Bit 10 - Pending Status of Posted Writes to CNT1"]
    #[inline(always)]
    pub fn wpndcnt1(&self) -> WPNDCNT1_R {
        WPNDCNT1_R::new(((self.bits >> 10) & 0x01) != 0)
    }
    #[doc = "Bit 11 - Pending Status of Posted Writes to ALM0"]
    #[inline(always)]
    pub fn wpndalm0(&self) -> WPNDALM0_R {
        WPNDALM0_R::new(((self.bits >> 11) & 0x01) != 0)
    }
    #[doc = "Bit 12 - Pending Status of Posted Writes to ALM1"]
    #[inline(always)]
    pub fn wpndalm1(&self) -> WPNDALM1_R {
        WPNDALM1_R::new(((self.bits >> 12) & 0x01) != 0)
    }
    #[doc = "Bit 13 - Pending Status of Posted Writes to TRM"]
    #[inline(always)]
    pub fn wpndtrm(&self) -> WPNDTRM_R {
        WPNDTRM_R::new(((self.bits >> 13) & 0x01) != 0)
    }
}
