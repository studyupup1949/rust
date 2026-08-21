#[doc = "Reader of register CTL_B"]
pub type R = crate::R<u32, super::CTL_B>;
#[doc = "Writer for register CTL_B"]
pub type W = crate::W<u32, super::CTL_B>;
#[doc = "Register CTL_B `reset()`'s with value 0"]
impl crate::ResetValue for super::CTL_B {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `SPEN`"]
pub type SPEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SPEN`"]
pub struct SPEN_W<'a> {
    w: &'a mut W,
}
impl<'a> SPEN_W<'a> {
    #[doc = r"Sets the field bit"]
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x01) | ((value as u32) & 0x01);
        self.w
    }
}
#[doc = "Reader of field `LSBF`"]
pub type LSBF_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `LSBF`"]
pub struct LSBF_W<'a> {
    w: &'a mut W,
}
impl<'a> LSBF_W<'a> {
    #[doc = r"Sets the field bit"]
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x01 << 3)) | (((value as u32) & 0x01) << 3);
        self.w
    }
}
#[doc = "Reader of field `SLEN`"]
pub type SLEN_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `SLEN`"]
pub struct SLEN_W<'a> {
    w: &'a mut W,
}
impl<'a> SLEN_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x1f << 4)) | (((value as u32) & 0x1f) << 4);
        self.w
    }
}
#[doc = "Reader of field `ICLK`"]
pub type ICLK_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ICLK`"]
pub struct ICLK_W<'a> {
    w: &'a mut W,
}
impl<'a> ICLK_W<'a> {
    #[doc = r"Sets the field bit"]
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x01 << 10)) | (((value as u32) & 0x01) << 10);
        self.w
    }
}
#[doc = "Reader of field `OPMODE`"]
pub type OPMODE_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `OPMODE`"]
pub struct OPMODE_W<'a> {
    w: &'a mut W,
}
impl<'a> OPMODE_W<'a> {
    #[doc = r"Sets the field bit"]
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x01 << 11)) | (((value as u32) & 0x01) << 11);
        self.w
    }
}
#[doc = "Reader of field `CKRE`"]
pub type CKRE_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `CKRE`"]
pub struct CKRE_W<'a> {
    w: &'a mut W,
}
impl<'a> CKRE_W<'a> {
    #[doc = r"Sets the field bit"]
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x01 << 12)) | (((value as u32) & 0x01) << 12);
        self.w
    }
}
#[doc = "Reader of field `FSR`"]
pub type FSR_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `FSR`"]
pub struct FSR_W<'a> {
    w: &'a mut W,
}
impl<'a> FSR_W<'a> {
    #[doc = r"Sets the field bit"]
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x01 << 13)) | (((value as u32) & 0x01) << 13);
        self.w
    }
}
#[doc = "Reader of field `IFS`"]
pub type IFS_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `IFS`"]
pub struct IFS_W<'a> {
    w: &'a mut W,
}
impl<'a> IFS_W<'a> {
    #[doc = r"Sets the field bit"]
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x01 << 14)) | (((value as u32) & 0x01) << 14);
        self.w
    }
}
#[doc = "Reader of field `DIFS`"]
pub type DIFS_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `DIFS`"]
pub struct DIFS_W<'a> {
    w: &'a mut W,
}
impl<'a> DIFS_W<'a> {
    #[doc = r"Sets the field bit"]
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x01 << 15)) | (((value as u32) & 0x01) << 15);
        self.w
    }
}
#[doc = "Reader of field `LFS`"]
pub type LFS_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `LFS`"]
pub struct LFS_W<'a> {
    w: &'a mut W,
}
impl<'a> LFS_W<'a> {
    #[doc = r"Sets the field bit"]
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x01 << 16)) | (((value as u32) & 0x01) << 16);
        self.w
    }
}
#[doc = "Reader of field `LAFS`"]
pub type LAFS_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `LAFS`"]
pub struct LAFS_W<'a> {
    w: &'a mut W,
}
impl<'a> LAFS_W<'a> {
    #[doc = r"Sets the field bit"]
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x01 << 17)) | (((value as u32) & 0x01) << 17);
        self.w
    }
}
#[doc = "Packing Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum PACK_A {
    #[doc = "0: Disable"]
    CTL_PACK_DIS = 0,
    #[doc = "1: 8-bit packing enable"]
    CTL_PACK_8BIT = 1,
    #[doc = "2: 16-bit packing enable"]
    CTL_PACK_16BIT = 2,
    #[doc = "3: Reserved"]
    CTL_PACK_RSV = 3,
}
impl From<PACK_A> for u8 {
    #[inline(always)]
    fn from(variant: PACK_A) -> Self {
        variant as _
    }
}
#[doc = "Reader of field `PACK`"]
pub type PACK_R = crate::R<u8, PACK_A>;
impl PACK_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PACK_A {
        match self.bits {
            0 => PACK_A::CTL_PACK_DIS,
            1 => PACK_A::CTL_PACK_8BIT,
            2 => PACK_A::CTL_PACK_16BIT,
            3 => PACK_A::CTL_PACK_RSV,
            _ => unreachable!(),
        }
    }
    #[doc = "Checks if the value of the field is `CTL_PACK_DIS`"]
    #[inline(always)]
    pub fn is_ctl_pack_dis(&self) -> bool {
        *self == PACK_A::CTL_PACK_DIS
    }
    #[doc = "Checks if the value of the field is `CTL_PACK_8BIT`"]
    #[inline(always)]
    pub fn is_ctl_pack_8bit(&self) -> bool {
        *self == PACK_A::CTL_PACK_8BIT
    }
    #[doc = "Checks if the value of the field is `CTL_PACK_16BIT`"]
    #[inline(always)]
    pub fn is_ctl_pack_16bit(&self) -> bool {
        *self == PACK_A::CTL_PACK_16BIT
    }
    #[doc = "Checks if the value of the field is `CTL_PACK_RSV`"]
    #[inline(always)]
    pub fn is_ctl_pack_rsv(&self) -> bool {
        *self == PACK_A::CTL_PACK_RSV
    }
}
#[doc = "Write proxy for field `PACK`"]
pub struct PACK_W<'a> {
    w: &'a mut W,
}
impl<'a> PACK_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: PACK_A) -> &'a mut W {
        {
            self.bits(variant.into())
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn ctl_pack_dis(self) -> &'a mut W {
        self.variant(PACK_A::CTL_PACK_DIS)
    }
    #[doc = "8-bit packing enable"]
    #[inline(always)]
    pub fn ctl_pack_8bit(self) -> &'a mut W {
        self.variant(PACK_A::CTL_PACK_8BIT)
    }
    #[doc = "16-bit packing enable"]
    #[inline(always)]
    pub fn ctl_pack_16bit(self) -> &'a mut W {
        self.variant(PACK_A::CTL_PACK_16BIT)
    }
    #[doc = "Reserved"]
    #[inline(always)]
    pub fn ctl_pack_rsv(self) -> &'a mut W {
        self.variant(PACK_A::CTL_PACK_RSV)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 18)) | (((value as u32) & 0x03) << 18);
        self.w
    }
}
#[doc = "Reader of field `FSERRMODE`"]
pub type FSERRMODE_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `FSERRMODE`"]
pub struct FSERRMODE_W<'a> {
    w: &'a mut W,
}
impl<'a> FSERRMODE_W<'a> {
    #[doc = r"Sets the field bit"]
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x01 << 20)) | (((value as u32) & 0x01) << 20);
        self.w
    }
}
#[doc = "Reader of field `GCLKEN`"]
pub type GCLKEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `GCLKEN`"]
pub struct GCLKEN_W<'a> {
    w: &'a mut W,
}
impl<'a> GCLKEN_W<'a> {
    #[doc = r"Sets the field bit"]
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x01 << 21)) | (((value as u32) & 0x01) << 21);
        self.w
    }
}
#[doc = "Reader of field `SPTRAN`"]
pub type SPTRAN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SPTRAN`"]
pub struct SPTRAN_W<'a> {
    w: &'a mut W,
}
impl<'a> SPTRAN_W<'a> {
    #[doc = r"Sets the field bit"]
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x01 << 25)) | (((value as u32) & 0x01) << 25);
        self.w
    }
}
#[doc = "Reader of field `DMAEN`"]
pub type DMAEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `DMAEN`"]
pub struct DMAEN_W<'a> {
    w: &'a mut W,
}
impl<'a> DMAEN_W<'a> {
    #[doc = r"Sets the field bit"]
    #[inline(always)]
    pub fn set_bit(self) -> &'a mut W {
        self.bit(true)
    }
    #[doc = r"Clears the field bit"]
    #[inline(always)]
    pub fn clear_bit(self) -> &'a mut W {
        self.bit(false)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bit(self, value: bool) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x01 << 26)) | (((value as u32) & 0x01) << 26);
        self.w
    }
}
impl R {
    #[doc = "Bit 0 - Serial Port Enable"]
    #[inline(always)]
    pub fn spen(&self) -> SPEN_R {
        SPEN_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 3 - Least-Significant Bit First"]
    #[inline(always)]
    pub fn lsbf(&self) -> LSBF_R {
        LSBF_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bits 4:8 - Serial Word Length"]
    #[inline(always)]
    pub fn slen(&self) -> SLEN_R {
        SLEN_R::new(((self.bits >> 4) & 0x1f) as u8)
    }
    #[doc = "Bit 10 - Internal Clock"]
    #[inline(always)]
    pub fn iclk(&self) -> ICLK_R {
        ICLK_R::new(((self.bits >> 10) & 0x01) != 0)
    }
    #[doc = "Bit 11 - Operation Mode"]
    #[inline(always)]
    pub fn opmode(&self) -> OPMODE_R {
        OPMODE_R::new(((self.bits >> 11) & 0x01) != 0)
    }
    #[doc = "Bit 12 - Clock Rising Edge"]
    #[inline(always)]
    pub fn ckre(&self) -> CKRE_R {
        CKRE_R::new(((self.bits >> 12) & 0x01) != 0)
    }
    #[doc = "Bit 13 - Frame Sync Required"]
    #[inline(always)]
    pub fn fsr(&self) -> FSR_R {
        FSR_R::new(((self.bits >> 13) & 0x01) != 0)
    }
    #[doc = "Bit 14 - Internal Frame Sync"]
    #[inline(always)]
    pub fn ifs(&self) -> IFS_R {
        IFS_R::new(((self.bits >> 14) & 0x01) != 0)
    }
    #[doc = "Bit 15 - Data-Independent Frame Sync"]
    #[inline(always)]
    pub fn difs(&self) -> DIFS_R {
        DIFS_R::new(((self.bits >> 15) & 0x01) != 0)
    }
    #[doc = "Bit 16 - Active-Low Frame Sync"]
    #[inline(always)]
    pub fn lfs(&self) -> LFS_R {
        LFS_R::new(((self.bits >> 16) & 0x01) != 0)
    }
    #[doc = "Bit 17 - Late Frame Sync"]
    #[inline(always)]
    pub fn lafs(&self) -> LAFS_R {
        LAFS_R::new(((self.bits >> 17) & 0x01) != 0)
    }
    #[doc = "Bits 18:19 - Packing Enable"]
    #[inline(always)]
    pub fn pack(&self) -> PACK_R {
        PACK_R::new(((self.bits >> 18) & 0x03) as u8)
    }
    #[doc = "Bit 20 - Frame Sync Error Operation"]
    #[inline(always)]
    pub fn fserrmode(&self) -> FSERRMODE_R {
        FSERRMODE_R::new(((self.bits >> 20) & 0x01) != 0)
    }
    #[doc = "Bit 21 - Gated Clock Enable"]
    #[inline(always)]
    pub fn gclken(&self) -> GCLKEN_R {
        GCLKEN_R::new(((self.bits >> 21) & 0x01) != 0)
    }
    #[doc = "Bit 25 - Serial Port Transfer Direction"]
    #[inline(always)]
    pub fn sptran(&self) -> SPTRAN_R {
        SPTRAN_R::new(((self.bits >> 25) & 0x01) != 0)
    }
    #[doc = "Bit 26 - DMA Enable"]
    #[inline(always)]
    pub fn dmaen(&self) -> DMAEN_R {
        DMAEN_R::new(((self.bits >> 26) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Serial Port Enable"]
    #[inline(always)]
    pub fn spen(&mut self) -> SPEN_W {
        SPEN_W { w: self }
    }
    #[doc = "Bit 3 - Least-Significant Bit First"]
    #[inline(always)]
    pub fn lsbf(&mut self) -> LSBF_W {
        LSBF_W { w: self }
    }
    #[doc = "Bits 4:8 - Serial Word Length"]
    #[inline(always)]
    pub fn slen(&mut self) -> SLEN_W {
        SLEN_W { w: self }
    }
    #[doc = "Bit 10 - Internal Clock"]
    #[inline(always)]
    pub fn iclk(&mut self) -> ICLK_W {
        ICLK_W { w: self }
    }
    #[doc = "Bit 11 - Operation Mode"]
    #[inline(always)]
    pub fn opmode(&mut self) -> OPMODE_W {
        OPMODE_W { w: self }
    }
    #[doc = "Bit 12 - Clock Rising Edge"]
    #[inline(always)]
    pub fn ckre(&mut self) -> CKRE_W {
        CKRE_W { w: self }
    }
    #[doc = "Bit 13 - Frame Sync Required"]
    #[inline(always)]
    pub fn fsr(&mut self) -> FSR_W {
        FSR_W { w: self }
    }
    #[doc = "Bit 14 - Internal Frame Sync"]
    #[inline(always)]
    pub fn ifs(&mut self) -> IFS_W {
        IFS_W { w: self }
    }
    #[doc = "Bit 15 - Data-Independent Frame Sync"]
    #[inline(always)]
    pub fn difs(&mut self) -> DIFS_W {
        DIFS_W { w: self }
    }
    #[doc = "Bit 16 - Active-Low Frame Sync"]
    #[inline(always)]
    pub fn lfs(&mut self) -> LFS_W {
        LFS_W { w: self }
    }
    #[doc = "Bit 17 - Late Frame Sync"]
    #[inline(always)]
    pub fn lafs(&mut self) -> LAFS_W {
        LAFS_W { w: self }
    }
    #[doc = "Bits 18:19 - Packing Enable"]
    #[inline(always)]
    pub fn pack(&mut self) -> PACK_W {
        PACK_W { w: self }
    }
    #[doc = "Bit 20 - Frame Sync Error Operation"]
    #[inline(always)]
    pub fn fserrmode(&mut self) -> FSERRMODE_W {
        FSERRMODE_W { w: self }
    }
    #[doc = "Bit 21 - Gated Clock Enable"]
    #[inline(always)]
    pub fn gclken(&mut self) -> GCLKEN_W {
        GCLKEN_W { w: self }
    }
    #[doc = "Bit 25 - Serial Port Transfer Direction"]
    #[inline(always)]
    pub fn sptran(&mut self) -> SPTRAN_W {
        SPTRAN_W { w: self }
    }
    #[doc = "Bit 26 - DMA Enable"]
    #[inline(always)]
    pub fn dmaen(&mut self) -> DMAEN_W {
        DMAEN_W { w: self }
    }
}
