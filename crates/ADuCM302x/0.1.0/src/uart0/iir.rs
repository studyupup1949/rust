#[doc = "Reader of register IIR"]
pub type R = crate::R<u16, super::IIR>;
#[doc = "Reader of field `NIRQ`"]
pub type NIRQ_R = crate::R<bool, bool>;
#[doc = "Interrupt Status\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum STAT_A {
    #[doc = "0: Modem status interrupt (Read MSR register to clear)"]
    STAT_EDSSI = 0,
    #[doc = "1: Transmit buffer empty interrupt (Write to Tx register or read IIR register to clear)"]
    STAT_ETBEI = 1,
    #[doc = "2: Receive buffer full interrupt (Read Rx register to clear)"]
    STAT_ERBFI = 2,
    #[doc = "3: Receive line status interrupt (Read LSR register to clear)"]
    STAT_RLSI = 3,
    #[doc = "6: Receive FIFO time-out interrupt (Read Rx register to clear)"]
    STAT_RFTOI = 6,
}
impl From<STAT_A> for u8 {
    #[inline(always)]
    fn from(variant: STAT_A) -> Self {
        variant as _
    }
}
#[doc = "Reader of field `STAT`"]
pub type STAT_R = crate::R<u8, STAT_A>;
impl STAT_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> crate::Variant<u8, STAT_A> {
        use crate::Variant::*;
        match self.bits {
            0 => Val(STAT_A::STAT_EDSSI),
            1 => Val(STAT_A::STAT_ETBEI),
            2 => Val(STAT_A::STAT_ERBFI),
            3 => Val(STAT_A::STAT_RLSI),
            6 => Val(STAT_A::STAT_RFTOI),
            i => Res(i),
        }
    }
    #[doc = "Checks if the value of the field is `STAT_EDSSI`"]
    #[inline(always)]
    pub fn is_stat_edssi(&self) -> bool {
        *self == STAT_A::STAT_EDSSI
    }
    #[doc = "Checks if the value of the field is `STAT_ETBEI`"]
    #[inline(always)]
    pub fn is_stat_etbei(&self) -> bool {
        *self == STAT_A::STAT_ETBEI
    }
    #[doc = "Checks if the value of the field is `STAT_ERBFI`"]
    #[inline(always)]
    pub fn is_stat_erbfi(&self) -> bool {
        *self == STAT_A::STAT_ERBFI
    }
    #[doc = "Checks if the value of the field is `STAT_RLSI`"]
    #[inline(always)]
    pub fn is_stat_rlsi(&self) -> bool {
        *self == STAT_A::STAT_RLSI
    }
    #[doc = "Checks if the value of the field is `STAT_RFTOI`"]
    #[inline(always)]
    pub fn is_stat_rftoi(&self) -> bool {
        *self == STAT_A::STAT_RFTOI
    }
}
#[doc = "Reader of field `FEND`"]
pub type FEND_R = crate::R<u8, u8>;
impl R {
    #[doc = "Bit 0 - Interrupt Flag"]
    #[inline(always)]
    pub fn nirq(&self) -> NIRQ_R {
        NIRQ_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bits 1:3 - Interrupt Status"]
    #[inline(always)]
    pub fn stat(&self) -> STAT_R {
        STAT_R::new(((self.bits >> 1) & 0x07) as u8)
    }
    #[doc = "Bits 6:7 - FIFO Enabled"]
    #[inline(always)]
    pub fn fend(&self) -> FEND_R {
        FEND_R::new(((self.bits >> 6) & 0x03) as u8)
    }
}
