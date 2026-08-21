#[doc = "Reader of register STAT"]
pub type R = crate::R<u16, super::STAT>;
#[doc = "Reader of field `IRQ`"]
pub type IRQ_R = crate::R<bool, bool>;
#[doc = "Reader of field `CLRIRQ`"]
pub type CLRIRQ_R = crate::R<bool, bool>;
#[doc = "Load Register Write Sync in Progress\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LOADING_A {
    #[doc = "0: APB and WDT clock domains LOAD values match."]
    LOAD_MATCH = 0,
    #[doc = "1: APB LOAD value is being synchronized to WDT clock domain."]
    LOAD_SYNCING = 1,
}
impl From<LOADING_A> for bool {
    #[inline(always)]
    fn from(variant: LOADING_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `LOADING`"]
pub type LOADING_R = crate::R<bool, LOADING_A>;
impl LOADING_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> LOADING_A {
        match self.bits {
            false => LOADING_A::LOAD_MATCH,
            true => LOADING_A::LOAD_SYNCING,
        }
    }
    #[doc = "Checks if the value of the field is `LOAD_MATCH`"]
    #[inline(always)]
    pub fn is_load_match(&self) -> bool {
        *self == LOADING_A::LOAD_MATCH
    }
    #[doc = "Checks if the value of the field is `LOAD_SYNCING`"]
    #[inline(always)]
    pub fn is_load_syncing(&self) -> bool {
        *self == LOADING_A::LOAD_SYNCING
    }
}
#[doc = "Control Register Write Sync in Progress\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum COUNTING_A {
    #[doc = "0: APB and WDT clock domain CTRL values match"]
    COUNT_MATCH = 0,
    #[doc = "1: APB CTRL register values are being synchronized to WDT clock domain."]
    COUNT_SYNCING = 1,
}
impl From<COUNTING_A> for bool {
    #[inline(always)]
    fn from(variant: COUNTING_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `COUNTING`"]
pub type COUNTING_R = crate::R<bool, COUNTING_A>;
impl COUNTING_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> COUNTING_A {
        match self.bits {
            false => COUNTING_A::COUNT_MATCH,
            true => COUNTING_A::COUNT_SYNCING,
        }
    }
    #[doc = "Checks if the value of the field is `COUNT_MATCH`"]
    #[inline(always)]
    pub fn is_count_match(&self) -> bool {
        *self == COUNTING_A::COUNT_MATCH
    }
    #[doc = "Checks if the value of the field is `COUNT_SYNCING`"]
    #[inline(always)]
    pub fn is_count_syncing(&self) -> bool {
        *self == COUNTING_A::COUNT_SYNCING
    }
}
#[doc = "Reader of field `LOCKED`"]
pub type LOCKED_R = crate::R<bool, bool>;
impl R {
    #[doc = "Bit 0 - WDT Interrupt"]
    #[inline(always)]
    pub fn irq(&self) -> IRQ_R {
        IRQ_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Clear Interrupt Register Write Sync in Progress"]
    #[inline(always)]
    pub fn clrirq(&self) -> CLRIRQ_R {
        CLRIRQ_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Load Register Write Sync in Progress"]
    #[inline(always)]
    pub fn loading(&self) -> LOADING_R {
        LOADING_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Control Register Write Sync in Progress"]
    #[inline(always)]
    pub fn counting(&self) -> COUNTING_R {
        COUNTING_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Lock Status Bit"]
    #[inline(always)]
    pub fn locked(&self) -> LOCKED_R {
        LOCKED_R::new(((self.bits >> 4) & 0x01) != 0)
    }
}
