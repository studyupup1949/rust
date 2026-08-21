#[doc = "Reader of register STAT"]
pub type R = crate::R<u32, super::STAT>;
#[doc = "Writer for register STAT"]
pub type W = crate::W<u32, super::STAT>;
#[doc = "Register STAT `reset()`'s with value 0"]
impl crate::ResetValue for super::STAT {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0
    }
}
#[doc = "Reader of field `CMDBUSY`"]
pub type CMDBUSY_R = crate::R<bool, bool>;
#[doc = "Reader of field `WRCLOSE`"]
pub type WRCLOSE_R = crate::R<bool, bool>;
#[doc = "Reader of field `CMDCOMP`"]
pub type CMDCOMP_R = crate::R<bool, bool>;
#[doc = "Reader of field `WRALCOMP`"]
pub type WRALCOMP_R = crate::R<bool, bool>;
#[doc = "Reader of field `CMDFAIL`"]
pub type CMDFAIL_R = crate::R<u8, u8>;
#[doc = "Reader of field `SLEEPING`"]
pub type SLEEPING_R = crate::R<bool, bool>;
#[doc = "Reader of field `ECCERRCMD`"]
pub type ECCERRCMD_R = crate::R<u8, u8>;
#[doc = "Reader of field `ECCRDERR`"]
pub type ECCRDERR_R = crate::R<u8, u8>;
#[doc = "Reader of field `OVERLAP`"]
pub type OVERLAP_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `OVERLAP`"]
pub struct OVERLAP_W<'a> {
    w: &'a mut W,
}
impl<'a> OVERLAP_W<'a> {
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
#[doc = "Reader of field `SIGNERR`"]
pub type SIGNERR_R = crate::R<bool, bool>;
#[doc = "Reader of field `INIT`"]
pub type INIT_R = crate::R<bool, bool>;
#[doc = "Reader of field `ECCINFOSIGN`"]
pub type ECCINFOSIGN_R = crate::R<u8, u8>;
#[doc = "Reader of field `ECCERRCNT`"]
pub type ECCERRCNT_R = crate::R<u8, u8>;
#[doc = "Reader of field `ECCICODE`"]
pub type ECCICODE_R = crate::R<u8, u8>;
#[doc = "Reader of field `ECCDCODE`"]
pub type ECCDCODE_R = crate::R<u8, u8>;
#[doc = "Reader of field `CACHESRAMPERR`"]
pub type CACHESRAMPERR_R = crate::R<bool, bool>;
impl R {
    #[doc = "Bit 0 - Command Busy"]
    #[inline(always)]
    pub fn cmdbusy(&self) -> CMDBUSY_R {
        CMDBUSY_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - WRITE Registers are Closed"]
    #[inline(always)]
    pub fn wrclose(&self) -> WRCLOSE_R {
        WRCLOSE_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Command Complete"]
    #[inline(always)]
    pub fn cmdcomp(&self) -> CMDCOMP_R {
        CMDCOMP_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Write Almost Complete"]
    #[inline(always)]
    pub fn wralcomp(&self) -> WRALCOMP_R {
        WRALCOMP_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bits 4:5 - Provides Information on Command Failures"]
    #[inline(always)]
    pub fn cmdfail(&self) -> CMDFAIL_R {
        CMDFAIL_R::new(((self.bits >> 4) & 0x03) as u8)
    }
    #[doc = "Bit 6 - Flash Array is in Low Power (Sleep) Mode"]
    #[inline(always)]
    pub fn sleeping(&self) -> SLEEPING_R {
        SLEEPING_R::new(((self.bits >> 6) & 0x01) != 0)
    }
    #[doc = "Bits 7:8 - ECC Errors Detected During User Issued SIGN Command"]
    #[inline(always)]
    pub fn eccerrcmd(&self) -> ECCERRCMD_R {
        ECCERRCMD_R::new(((self.bits >> 7) & 0x03) as u8)
    }
    #[doc = "Bits 9:10 - ECC IRQ Cause"]
    #[inline(always)]
    pub fn eccrderr(&self) -> ECCRDERR_R {
        ECCRDERR_R::new(((self.bits >> 9) & 0x03) as u8)
    }
    #[doc = "Bit 11 - Overlapping Command"]
    #[inline(always)]
    pub fn overlap(&self) -> OVERLAP_R {
        OVERLAP_R::new(((self.bits >> 11) & 0x01) != 0)
    }
    #[doc = "Bit 13 - Signature Check Failure During Initialization"]
    #[inline(always)]
    pub fn signerr(&self) -> SIGNERR_R {
        SIGNERR_R::new(((self.bits >> 13) & 0x01) != 0)
    }
    #[doc = "Bit 14 - Flash Controller Initialization in Progress"]
    #[inline(always)]
    pub fn init(&self) -> INIT_R {
        INIT_R::new(((self.bits >> 14) & 0x01) != 0)
    }
    #[doc = "Bits 15:16 - ECC Status of Flash Initialization"]
    #[inline(always)]
    pub fn eccinfosign(&self) -> ECCINFOSIGN_R {
        ECCINFOSIGN_R::new(((self.bits >> 15) & 0x03) as u8)
    }
    #[doc = "Bits 17:19 - ECC Correction Counter"]
    #[inline(always)]
    pub fn eccerrcnt(&self) -> ECCERRCNT_R {
        ECCERRCNT_R::new(((self.bits >> 17) & 0x07) as u8)
    }
    #[doc = "Bits 25:26 - ICode AHB Bus Error ECC Status"]
    #[inline(always)]
    pub fn eccicode(&self) -> ECCICODE_R {
        ECCICODE_R::new(((self.bits >> 25) & 0x03) as u8)
    }
    #[doc = "Bits 27:28 - DCode AHB Bus Error ECC Status"]
    #[inline(always)]
    pub fn eccdcode(&self) -> ECCDCODE_R {
        ECCDCODE_R::new(((self.bits >> 27) & 0x03) as u8)
    }
    #[doc = "Bit 29 - SRAM Parity Errors in Cache Controller"]
    #[inline(always)]
    pub fn cachesramperr(&self) -> CACHESRAMPERR_R {
        CACHESRAMPERR_R::new(((self.bits >> 29) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 11 - Overlapping Command"]
    #[inline(always)]
    pub fn overlap(&mut self) -> OVERLAP_W {
        OVERLAP_W { w: self }
    }
}
