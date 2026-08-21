#[doc = "Reader of register STAT_B"]
pub type R = crate::R<u32, super::STAT_B>;
#[doc = "Reader of field `TFI`"]
pub type TFI_R = crate::R<bool, bool>;
#[doc = "Reader of field `DERR`"]
pub type DERR_R = crate::R<bool, bool>;
#[doc = "Reader of field `FSERR`"]
pub type FSERR_R = crate::R<bool, bool>;
#[doc = "Reader of field `DATA`"]
pub type DATA_R = crate::R<bool, bool>;
#[doc = "Reader of field `SYSDATERR`"]
pub type SYSDATERR_R = crate::R<bool, bool>;
#[doc = "Data Transfer Buffer Status\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum DXS_A {
    #[doc = "0: Empty"]
    CTL_EMPTY = 0,
    #[doc = "1: Reserved"]
    CTL_RSV = 1,
    #[doc = "2: Partially full"]
    CTL_PART_FULL = 2,
    #[doc = "3: Full"]
    CTL_FULL = 3,
}
impl From<DXS_A> for u8 {
    #[inline(always)]
    fn from(variant: DXS_A) -> Self {
        variant as _
    }
}
#[doc = "Reader of field `DXS`"]
pub type DXS_R = crate::R<u8, DXS_A>;
impl DXS_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> DXS_A {
        match self.bits {
            0 => DXS_A::CTL_EMPTY,
            1 => DXS_A::CTL_RSV,
            2 => DXS_A::CTL_PART_FULL,
            3 => DXS_A::CTL_FULL,
            _ => unreachable!(),
        }
    }
    #[doc = "Checks if the value of the field is `CTL_EMPTY`"]
    #[inline(always)]
    pub fn is_ctl_empty(&self) -> bool {
        *self == DXS_A::CTL_EMPTY
    }
    #[doc = "Checks if the value of the field is `CTL_RSV`"]
    #[inline(always)]
    pub fn is_ctl_rsv(&self) -> bool {
        *self == DXS_A::CTL_RSV
    }
    #[doc = "Checks if the value of the field is `CTL_PART_FULL`"]
    #[inline(always)]
    pub fn is_ctl_part_full(&self) -> bool {
        *self == DXS_A::CTL_PART_FULL
    }
    #[doc = "Checks if the value of the field is `CTL_FULL`"]
    #[inline(always)]
    pub fn is_ctl_full(&self) -> bool {
        *self == DXS_A::CTL_FULL
    }
}
impl R {
    #[doc = "Bit 0 - Transmit Finish Interrupt Status"]
    #[inline(always)]
    pub fn tfi(&self) -> TFI_R {
        TFI_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Data Error Status"]
    #[inline(always)]
    pub fn derr(&self) -> DERR_R {
        DERR_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Frame Sync Error Status"]
    #[inline(always)]
    pub fn fserr(&self) -> FSERR_R {
        FSERR_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Data Buffer Status"]
    #[inline(always)]
    pub fn data(&self) -> DATA_R {
        DATA_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 4 - System Data Error Status"]
    #[inline(always)]
    pub fn sysdaterr(&self) -> SYSDATERR_R {
        SYSDATERR_R::new(((self.bits >> 4) & 0x01) != 0)
    }
    #[doc = "Bits 8:9 - Data Transfer Buffer Status"]
    #[inline(always)]
    pub fn dxs(&self) -> DXS_R {
        DXS_R::new(((self.bits >> 8) & 0x03) as u8)
    }
}
