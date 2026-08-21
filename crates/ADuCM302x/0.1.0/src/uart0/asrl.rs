#[doc = "Reader of register ASRL"]
pub type R = crate::R<u16, super::ASRL>;
#[doc = "Reader of field `DONE`"]
pub type DONE_R = crate::R<bool, bool>;
#[doc = "Reader of field `BRKTO`"]
pub type BRKTO_R = crate::R<bool, bool>;
#[doc = "Reader of field `NSETO`"]
pub type NSETO_R = crate::R<bool, bool>;
#[doc = "Reader of field `NEETO`"]
pub type NEETO_R = crate::R<bool, bool>;
#[doc = "Reader of field `CNT`"]
pub type CNT_R = crate::R<u16, u16>;
impl R {
    #[doc = "Bit 0 - Auto Baud Done Successfully"]
    #[inline(always)]
    pub fn done(&self) -> DONE_R {
        DONE_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Timed Out Due to Long Time Break Condition"]
    #[inline(always)]
    pub fn brkto(&self) -> BRKTO_R {
        BRKTO_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Timed Out Due to No Valid Start Edge Found"]
    #[inline(always)]
    pub fn nseto(&self) -> NSETO_R {
        NSETO_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Timed Out Due to No Valid Ending Edge Found"]
    #[inline(always)]
    pub fn neeto(&self) -> NEETO_R {
        NEETO_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bits 4:15 - CNT\\[11:0\\]
Auto Baud Counter Value"]
    #[inline(always)]
    pub fn cnt(&self) -> CNT_R {
        CNT_R::new(((self.bits >> 4) & 0x0fff) as u16)
    }
}
