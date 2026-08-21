#[doc = "Reader of register CFG"]
pub type R = crate::R<u32, super::CFG>;
#[doc = "Writer for register CFG"]
pub type W = crate::W<u32, super::CFG>;
#[doc = "Register CFG `reset()`'s with value 0x1000_0000"]
impl crate::ResetValue for super::CFG {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x1000_0000
    }
}
#[doc = "Enable Bit for Crypto Block\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BLKEN_A {
    #[doc = "0: Enable Crypto Block"]
    ENABLE = 0,
    #[doc = "1: Disable Crypto Block"]
    DISABLE = 1,
}
impl From<BLKEN_A> for bool {
    #[inline(always)]
    fn from(variant: BLKEN_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `BLKEN`"]
pub type BLKEN_R = crate::R<bool, BLKEN_A>;
impl BLKEN_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> BLKEN_A {
        match self.bits {
            false => BLKEN_A::ENABLE,
            true => BLKEN_A::DISABLE,
        }
    }
    #[doc = "Checks if the value of the field is `ENABLE`"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == BLKEN_A::ENABLE
    }
    #[doc = "Checks if the value of the field is `DISABLE`"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == BLKEN_A::DISABLE
    }
}
#[doc = "Write proxy for field `BLKEN`"]
pub struct BLKEN_W<'a> {
    w: &'a mut W,
}
impl<'a> BLKEN_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: BLKEN_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Enable Crypto Block"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut W {
        self.variant(BLKEN_A::ENABLE)
    }
    #[doc = "Disable Crypto Block"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut W {
        self.variant(BLKEN_A::DISABLE)
    }
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
#[doc = "Reader of field `ENCR`"]
pub type ENCR_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ENCR`"]
pub struct ENCR_W<'a> {
    w: &'a mut W,
}
impl<'a> ENCR_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 1)) | (((value as u32) & 0x01) << 1);
        self.w
    }
}
#[doc = "Enable DMA Channel Request for Input Buffer\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum INDMAEN_A {
    #[doc = "0: Disable DMA Requesting for Input Buffer"]
    DMA_DISABLE_INBUF = 0,
    #[doc = "1: Enable DMA Requesting for Input Buffer"]
    DMA_ENABLE_INBUF = 1,
}
impl From<INDMAEN_A> for bool {
    #[inline(always)]
    fn from(variant: INDMAEN_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `INDMAEN`"]
pub type INDMAEN_R = crate::R<bool, INDMAEN_A>;
impl INDMAEN_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> INDMAEN_A {
        match self.bits {
            false => INDMAEN_A::DMA_DISABLE_INBUF,
            true => INDMAEN_A::DMA_ENABLE_INBUF,
        }
    }
    #[doc = "Checks if the value of the field is `DMA_DISABLE_INBUF`"]
    #[inline(always)]
    pub fn is_dma_disable_inbuf(&self) -> bool {
        *self == INDMAEN_A::DMA_DISABLE_INBUF
    }
    #[doc = "Checks if the value of the field is `DMA_ENABLE_INBUF`"]
    #[inline(always)]
    pub fn is_dma_enable_inbuf(&self) -> bool {
        *self == INDMAEN_A::DMA_ENABLE_INBUF
    }
}
#[doc = "Write proxy for field `INDMAEN`"]
pub struct INDMAEN_W<'a> {
    w: &'a mut W,
}
impl<'a> INDMAEN_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: INDMAEN_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Disable DMA Requesting for Input Buffer"]
    #[inline(always)]
    pub fn dma_disable_inbuf(self) -> &'a mut W {
        self.variant(INDMAEN_A::DMA_DISABLE_INBUF)
    }
    #[doc = "Enable DMA Requesting for Input Buffer"]
    #[inline(always)]
    pub fn dma_enable_inbuf(self) -> &'a mut W {
        self.variant(INDMAEN_A::DMA_ENABLE_INBUF)
    }
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
        self.w.bits = (self.w.bits & !(0x01 << 2)) | (((value as u32) & 0x01) << 2);
        self.w
    }
}
#[doc = "Enable DMA Channel Request for Output Buffer\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OUTDMAEN_A {
    #[doc = "0: Disable DMA Requesting for Output Buffer"]
    DMA_DISABLE_OUTBUF = 0,
    #[doc = "1: Enable DMA Requesting for Output Buffer"]
    DMA_ENABLE_OUTBUF = 1,
}
impl From<OUTDMAEN_A> for bool {
    #[inline(always)]
    fn from(variant: OUTDMAEN_A) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Reader of field `OUTDMAEN`"]
pub type OUTDMAEN_R = crate::R<bool, OUTDMAEN_A>;
impl OUTDMAEN_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> OUTDMAEN_A {
        match self.bits {
            false => OUTDMAEN_A::DMA_DISABLE_OUTBUF,
            true => OUTDMAEN_A::DMA_ENABLE_OUTBUF,
        }
    }
    #[doc = "Checks if the value of the field is `DMA_DISABLE_OUTBUF`"]
    #[inline(always)]
    pub fn is_dma_disable_outbuf(&self) -> bool {
        *self == OUTDMAEN_A::DMA_DISABLE_OUTBUF
    }
    #[doc = "Checks if the value of the field is `DMA_ENABLE_OUTBUF`"]
    #[inline(always)]
    pub fn is_dma_enable_outbuf(&self) -> bool {
        *self == OUTDMAEN_A::DMA_ENABLE_OUTBUF
    }
}
#[doc = "Write proxy for field `OUTDMAEN`"]
pub struct OUTDMAEN_W<'a> {
    w: &'a mut W,
}
impl<'a> OUTDMAEN_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: OUTDMAEN_A) -> &'a mut W {
        {
            self.bit(variant.into())
        }
    }
    #[doc = "Disable DMA Requesting for Output Buffer"]
    #[inline(always)]
    pub fn dma_disable_outbuf(self) -> &'a mut W {
        self.variant(OUTDMAEN_A::DMA_DISABLE_OUTBUF)
    }
    #[doc = "Enable DMA Requesting for Output Buffer"]
    #[inline(always)]
    pub fn dma_enable_outbuf(self) -> &'a mut W {
        self.variant(OUTDMAEN_A::DMA_ENABLE_OUTBUF)
    }
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
#[doc = "Write proxy for field `INFLUSH`"]
pub struct INFLUSH_W<'a> {
    w: &'a mut W,
}
impl<'a> INFLUSH_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 4)) | (((value as u32) & 0x01) << 4);
        self.w
    }
}
#[doc = "Write proxy for field `OUTFLUSH`"]
pub struct OUTFLUSH_W<'a> {
    w: &'a mut W,
}
impl<'a> OUTFLUSH_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 5)) | (((value as u32) & 0x01) << 5);
        self.w
    }
}
#[doc = "Reader of field `AES_BYTESWAP`"]
pub type AES_BYTESWAP_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `AES_BYTESWAP`"]
pub struct AES_BYTESWAP_W<'a> {
    w: &'a mut W,
}
impl<'a> AES_BYTESWAP_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 6)) | (((value as u32) & 0x01) << 6);
        self.w
    }
}
#[doc = "Select Key Length for AES Cipher\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum AESKEYLEN_A {
    #[doc = "0: Uses 128-bit long key"]
    AESKEYLEN128 = 0,
    #[doc = "2: Uses 256-bit long key"]
    AESKEYLEN256 = 2,
}
impl From<AESKEYLEN_A> for u8 {
    #[inline(always)]
    fn from(variant: AESKEYLEN_A) -> Self {
        variant as _
    }
}
#[doc = "Reader of field `AESKEYLEN`"]
pub type AESKEYLEN_R = crate::R<u8, AESKEYLEN_A>;
impl AESKEYLEN_R {
    #[doc = r"Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> crate::Variant<u8, AESKEYLEN_A> {
        use crate::Variant::*;
        match self.bits {
            0 => Val(AESKEYLEN_A::AESKEYLEN128),
            2 => Val(AESKEYLEN_A::AESKEYLEN256),
            i => Res(i),
        }
    }
    #[doc = "Checks if the value of the field is `AESKEYLEN128`"]
    #[inline(always)]
    pub fn is_aeskeylen128(&self) -> bool {
        *self == AESKEYLEN_A::AESKEYLEN128
    }
    #[doc = "Checks if the value of the field is `AESKEYLEN256`"]
    #[inline(always)]
    pub fn is_aeskeylen256(&self) -> bool {
        *self == AESKEYLEN_A::AESKEYLEN256
    }
}
#[doc = "Write proxy for field `AESKEYLEN`"]
pub struct AESKEYLEN_W<'a> {
    w: &'a mut W,
}
impl<'a> AESKEYLEN_W<'a> {
    #[doc = r"Writes `variant` to the field"]
    #[inline(always)]
    pub fn variant(self, variant: AESKEYLEN_A) -> &'a mut W {
        unsafe { self.bits(variant.into()) }
    }
    #[doc = "Uses 128-bit long key"]
    #[inline(always)]
    pub fn aeskeylen128(self) -> &'a mut W {
        self.variant(AESKEYLEN_A::AESKEYLEN128)
    }
    #[doc = "Uses 256-bit long key"]
    #[inline(always)]
    pub fn aeskeylen256(self) -> &'a mut W {
        self.variant(AESKEYLEN_A::AESKEYLEN256)
    }
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub unsafe fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 8)) | (((value as u32) & 0x03) << 8);
        self.w
    }
}
#[doc = "Reader of field `ECBEN`"]
pub type ECBEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `ECBEN`"]
pub struct ECBEN_W<'a> {
    w: &'a mut W,
}
impl<'a> ECBEN_W<'a> {
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
#[doc = "Reader of field `CTREN`"]
pub type CTREN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `CTREN`"]
pub struct CTREN_W<'a> {
    w: &'a mut W,
}
impl<'a> CTREN_W<'a> {
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
#[doc = "Reader of field `CBCEN`"]
pub type CBCEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `CBCEN`"]
pub struct CBCEN_W<'a> {
    w: &'a mut W,
}
impl<'a> CBCEN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 18)) | (((value as u32) & 0x01) << 18);
        self.w
    }
}
#[doc = "Reader of field `CCMEN`"]
pub type CCMEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `CCMEN`"]
pub struct CCMEN_W<'a> {
    w: &'a mut W,
}
impl<'a> CCMEN_W<'a> {
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
        self.w.bits = (self.w.bits & !(0x01 << 19)) | (((value as u32) & 0x01) << 19);
        self.w
    }
}
#[doc = "Reader of field `CMACEN`"]
pub type CMACEN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `CMACEN`"]
pub struct CMACEN_W<'a> {
    w: &'a mut W,
}
impl<'a> CMACEN_W<'a> {
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
#[doc = "Reader of field `SHA256EN`"]
pub type SHA256EN_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SHA256EN`"]
pub struct SHA256EN_W<'a> {
    w: &'a mut W,
}
impl<'a> SHA256EN_W<'a> {
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
#[doc = "Reader of field `SHAINIT`"]
pub type SHAINIT_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `SHAINIT`"]
pub struct SHAINIT_W<'a> {
    w: &'a mut W,
}
impl<'a> SHAINIT_W<'a> {
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
#[doc = "Reader of field `RevID`"]
pub type REVID_R = crate::R<u8, u8>;
impl R {
    #[doc = "Bit 0 - Enable Bit for Crypto Block"]
    #[inline(always)]
    pub fn blken(&self) -> BLKEN_R {
        BLKEN_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - Encrypt or Decrypt"]
    #[inline(always)]
    pub fn encr(&self) -> ENCR_R {
        ENCR_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 2 - Enable DMA Channel Request for Input Buffer"]
    #[inline(always)]
    pub fn indmaen(&self) -> INDMAEN_R {
        INDMAEN_R::new(((self.bits >> 2) & 0x01) != 0)
    }
    #[doc = "Bit 3 - Enable DMA Channel Request for Output Buffer"]
    #[inline(always)]
    pub fn outdmaen(&self) -> OUTDMAEN_R {
        OUTDMAEN_R::new(((self.bits >> 3) & 0x01) != 0)
    }
    #[doc = "Bit 6 - Byte Swap 32 Bit AES Input Data"]
    #[inline(always)]
    pub fn aes_byteswap(&self) -> AES_BYTESWAP_R {
        AES_BYTESWAP_R::new(((self.bits >> 6) & 0x01) != 0)
    }
    #[doc = "Bits 8:9 - Select Key Length for AES Cipher"]
    #[inline(always)]
    pub fn aeskeylen(&self) -> AESKEYLEN_R {
        AESKEYLEN_R::new(((self.bits >> 8) & 0x03) as u8)
    }
    #[doc = "Bit 16 - Enable ECB Mode Operation"]
    #[inline(always)]
    pub fn ecben(&self) -> ECBEN_R {
        ECBEN_R::new(((self.bits >> 16) & 0x01) != 0)
    }
    #[doc = "Bit 17 - Enable CTR Mode Operation"]
    #[inline(always)]
    pub fn ctren(&self) -> CTREN_R {
        CTREN_R::new(((self.bits >> 17) & 0x01) != 0)
    }
    #[doc = "Bit 18 - Enable CBC Mode Operation"]
    #[inline(always)]
    pub fn cbcen(&self) -> CBCEN_R {
        CBCEN_R::new(((self.bits >> 18) & 0x01) != 0)
    }
    #[doc = "Bit 19 - Enable CCM/CCM* Mode Operation"]
    #[inline(always)]
    pub fn ccmen(&self) -> CCMEN_R {
        CCMEN_R::new(((self.bits >> 19) & 0x01) != 0)
    }
    #[doc = "Bit 20 - Enable CMAC Mode Operation"]
    #[inline(always)]
    pub fn cmacen(&self) -> CMACEN_R {
        CMACEN_R::new(((self.bits >> 20) & 0x01) != 0)
    }
    #[doc = "Bit 25 - Enable SHA-256 Operation"]
    #[inline(always)]
    pub fn sha256en(&self) -> SHA256EN_R {
        SHA256EN_R::new(((self.bits >> 25) & 0x01) != 0)
    }
    #[doc = "Bit 26 - Restarts SHA Computation"]
    #[inline(always)]
    pub fn shainit(&self) -> SHAINIT_R {
        SHAINIT_R::new(((self.bits >> 26) & 0x01) != 0)
    }
    #[doc = "Bits 28:31 - Rev ID for Crypto"]
    #[inline(always)]
    pub fn rev_id(&self) -> REVID_R {
        REVID_R::new(((self.bits >> 28) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - Enable Bit for Crypto Block"]
    #[inline(always)]
    pub fn blken(&mut self) -> BLKEN_W {
        BLKEN_W { w: self }
    }
    #[doc = "Bit 1 - Encrypt or Decrypt"]
    #[inline(always)]
    pub fn encr(&mut self) -> ENCR_W {
        ENCR_W { w: self }
    }
    #[doc = "Bit 2 - Enable DMA Channel Request for Input Buffer"]
    #[inline(always)]
    pub fn indmaen(&mut self) -> INDMAEN_W {
        INDMAEN_W { w: self }
    }
    #[doc = "Bit 3 - Enable DMA Channel Request for Output Buffer"]
    #[inline(always)]
    pub fn outdmaen(&mut self) -> OUTDMAEN_W {
        OUTDMAEN_W { w: self }
    }
    #[doc = "Bit 4 - Input Buffer Flush"]
    #[inline(always)]
    pub fn inflush(&mut self) -> INFLUSH_W {
        INFLUSH_W { w: self }
    }
    #[doc = "Bit 5 - Output Buffer Flush"]
    #[inline(always)]
    pub fn outflush(&mut self) -> OUTFLUSH_W {
        OUTFLUSH_W { w: self }
    }
    #[doc = "Bit 6 - Byte Swap 32 Bit AES Input Data"]
    #[inline(always)]
    pub fn aes_byteswap(&mut self) -> AES_BYTESWAP_W {
        AES_BYTESWAP_W { w: self }
    }
    #[doc = "Bits 8:9 - Select Key Length for AES Cipher"]
    #[inline(always)]
    pub fn aeskeylen(&mut self) -> AESKEYLEN_W {
        AESKEYLEN_W { w: self }
    }
    #[doc = "Bit 16 - Enable ECB Mode Operation"]
    #[inline(always)]
    pub fn ecben(&mut self) -> ECBEN_W {
        ECBEN_W { w: self }
    }
    #[doc = "Bit 17 - Enable CTR Mode Operation"]
    #[inline(always)]
    pub fn ctren(&mut self) -> CTREN_W {
        CTREN_W { w: self }
    }
    #[doc = "Bit 18 - Enable CBC Mode Operation"]
    #[inline(always)]
    pub fn cbcen(&mut self) -> CBCEN_W {
        CBCEN_W { w: self }
    }
    #[doc = "Bit 19 - Enable CCM/CCM* Mode Operation"]
    #[inline(always)]
    pub fn ccmen(&mut self) -> CCMEN_W {
        CCMEN_W { w: self }
    }
    #[doc = "Bit 20 - Enable CMAC Mode Operation"]
    #[inline(always)]
    pub fn cmacen(&mut self) -> CMACEN_W {
        CMACEN_W { w: self }
    }
    #[doc = "Bit 25 - Enable SHA-256 Operation"]
    #[inline(always)]
    pub fn sha256en(&mut self) -> SHA256EN_W {
        SHA256EN_W { w: self }
    }
    #[doc = "Bit 26 - Restarts SHA Computation"]
    #[inline(always)]
    pub fn shainit(&mut self) -> SHAINIT_W {
        SHAINIT_W { w: self }
    }
}
