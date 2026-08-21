#[doc = "Reader of register MSR"]
pub type R = crate::R<u16, super::MSR>;
#[doc = "Reader of field `DCTS`"]
pub type DCTS_R = crate::R<bool, bool>;
#[doc = "Reader of field `DDSR`"]
pub type DDSR_R = crate::R<bool, bool>;
#[doc = "Reader of field `TERI`"]
pub type TERI_R = crate::R<bool, bool>;
#[doc = "Reader of field `DDCD`"]
pub type DDCD_R = crate::R<bool, bool>;
#[doc = "Reader of field `CTS`"]
pub type CTS_R = crate::R<bool, bool>;
#[doc = "Reader of field `DSR`"]
pub type DSR_R = crate::R<bool, bool>;
#[doc = "Reader of field `RI`"]
pub type RI_R = crate::R<bool, bool>;
#[doc = "Reader of field `DCD`"]
pub type DCD_R = crate::R<bool, bool>;
impl R {
    #[doc = "Bit 0 - Delta CTS"]
    #[inline(always)]
    pub fn dcts(&self) -> DCTS_R {
        DCTS_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Delta DSR"]
    #[inline(always)]
    pub fn ddsr(&self) -> DDSR_R {
        DDSR_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Trailing Edge RI"]
    #[inline(always)]
    pub fn teri(&self) -> TERI_R {
        TERI_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Delta DCD"]
    #[inline(always)]
    pub fn ddcd(&self) -> DDCD_R {
        DDCD_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - Clear to Send"]
    #[inline(always)]
    pub fn cts(&self) -> CTS_R {
        CTS_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bit 5 - Data Set Ready"]
    #[inline(always)]
    pub fn dsr(&self) -> DSR_R {
        DSR_R::new(((self.bits >> 5) & 0x01) != 0)
    }
    #[doc = "Bit 6 - Ring Indicator"]
    #[inline(always)]
    pub fn ri(&self) -> RI_R {
        RI_R::new(((self.bits >> 6) & 0x01) != 0)
    }
    #[doc = "Bit 7 - Data Carrier Detect"]
    #[inline(always)]
    pub fn dcd(&self) -> DCD_R {
        DCD_R::new(((self.bits >> 7) & 0x01) != 0)
    }
}
