#[doc = "Reader of register ARBIT1"]
pub type R = crate::R<u32, super::ARBIT1>;
#[doc = "Writer for register ARBIT1"]
pub type W = crate::W<u32, super::ARBIT1>;
#[doc = "Register ARBIT1 `reset()`'s with value 0x0024_0024"]
impl crate::ResetValue for super::ARBIT1 {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x0024_0024
    }
}
#[doc = "Reader of field `SRAM1_DCODE`"]
pub type SRAM1_DCODE_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `SRAM1_DCODE`"]
pub struct SRAM1_DCODE_W<'a> {
    w: &'a mut W,
}
impl<'a> SRAM1_DCODE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x03) | ((value as u32) & 0x03);
        self.w
    }
}
#[doc = "Reader of field `SRAM1_SBUS`"]
pub type SRAM1_SBUS_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `SRAM1_SBUS`"]
pub struct SRAM1_SBUS_W<'a> {
    w: &'a mut W,
}
impl<'a> SRAM1_SBUS_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 2)) | (((value as u32) & 0x03) << 2);
        self.w
    }
}
#[doc = "Reader of field `SRAM1_DMA0`"]
pub type SRAM1_DMA0_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `SRAM1_DMA0`"]
pub struct SRAM1_DMA0_W<'a> {
    w: &'a mut W,
}
impl<'a> SRAM1_DMA0_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 4)) | (((value as u32) & 0x03) << 4);
        self.w
    }
}
#[doc = "Reader of field `SIP_DCODE`"]
pub type SIP_DCODE_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `SIP_DCODE`"]
pub struct SIP_DCODE_W<'a> {
    w: &'a mut W,
}
impl<'a> SIP_DCODE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 16)) | (((value as u32) & 0x03) << 16);
        self.w
    }
}
#[doc = "Reader of field `SIP_SBUS`"]
pub type SIP_SBUS_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `SIP_SBUS`"]
pub struct SIP_SBUS_W<'a> {
    w: &'a mut W,
}
impl<'a> SIP_SBUS_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 18)) | (((value as u32) & 0x03) << 18);
        self.w
    }
}
#[doc = "Reader of field `SIP_DMA0`"]
pub type SIP_DMA0_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `SIP_DMA0`"]
pub struct SIP_DMA0_W<'a> {
    w: &'a mut W,
}
impl<'a> SIP_DMA0_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 20)) | (((value as u32) & 0x03) << 20);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:1 - SRAM1 priority for Dcode"]
    #[inline(always)]
    pub fn sram1_dcode(&self) -> SRAM1_DCODE_R {
        SRAM1_DCODE_R::new((self.bits & 0x03) as u8)
    }
    #[doc = "Bits 2:3 - SRAM1 priority for SBUS"]
    #[inline(always)]
    pub fn sram1_sbus(&self) -> SRAM1_SBUS_R {
        SRAM1_SBUS_R::new(((self.bits >> 2) & 0x03) as u8)
    }
    #[doc = "Bits 4:5 - SRAM1 priority for DMA0"]
    #[inline(always)]
    pub fn sram1_dma0(&self) -> SRAM1_DMA0_R {
        SRAM1_DMA0_R::new(((self.bits >> 4) & 0x03) as u8)
    }
    #[doc = "Bits 16:17 - SIP priority for DCODE"]
    #[inline(always)]
    pub fn sip_dcode(&self) -> SIP_DCODE_R {
        SIP_DCODE_R::new(((self.bits >> 16) & 0x03) as u8)
    }
    #[doc = "Bits 18:19 - SIP priority for SBUS"]
    #[inline(always)]
    pub fn sip_sbus(&self) -> SIP_SBUS_R {
        SIP_SBUS_R::new(((self.bits >> 18) & 0x03) as u8)
    }
    #[doc = "Bits 20:21 - SIP priority for DMA0"]
    #[inline(always)]
    pub fn sip_dma0(&self) -> SIP_DMA0_R {
        SIP_DMA0_R::new(((self.bits >> 20) & 0x03) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - SRAM1 priority for Dcode"]
    #[inline(always)]
    pub fn sram1_dcode(&mut self) -> SRAM1_DCODE_W {
        SRAM1_DCODE_W { w: self }
    }
    #[doc = "Bits 2:3 - SRAM1 priority for SBUS"]
    #[inline(always)]
    pub fn sram1_sbus(&mut self) -> SRAM1_SBUS_W {
        SRAM1_SBUS_W { w: self }
    }
    #[doc = "Bits 4:5 - SRAM1 priority for DMA0"]
    #[inline(always)]
    pub fn sram1_dma0(&mut self) -> SRAM1_DMA0_W {
        SRAM1_DMA0_W { w: self }
    }
    #[doc = "Bits 16:17 - SIP priority for DCODE"]
    #[inline(always)]
    pub fn sip_dcode(&mut self) -> SIP_DCODE_W {
        SIP_DCODE_W { w: self }
    }
    #[doc = "Bits 18:19 - SIP priority for SBUS"]
    #[inline(always)]
    pub fn sip_sbus(&mut self) -> SIP_SBUS_W {
        SIP_SBUS_W { w: self }
    }
    #[doc = "Bits 20:21 - SIP priority for DMA0"]
    #[inline(always)]
    pub fn sip_dma0(&mut self) -> SIP_DMA0_W {
        SIP_DMA0_W { w: self }
    }
}
