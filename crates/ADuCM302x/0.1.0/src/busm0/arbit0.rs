#[doc = "Reader of register ARBIT0"]
pub type R = crate::R<u32, super::ARBIT0>;
#[doc = "Writer for register ARBIT0"]
pub type W = crate::W<u32, super::ARBIT0>;
#[doc = "Register ARBIT0 `reset()`'s with value 0x0024_0024"]
impl crate::ResetValue for super::ARBIT0 {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x0024_0024
    }
}
#[doc = "Reader of field `FLSH_DCODE`"]
pub type FLSH_DCODE_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `FLSH_DCODE`"]
pub struct FLSH_DCODE_W<'a> {
    w: &'a mut W,
}
impl<'a> FLSH_DCODE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x03) | ((value as u32) & 0x03);
        self.w
    }
}
#[doc = "Reader of field `FLSH_SBUS`"]
pub type FLSH_SBUS_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `FLSH_SBUS`"]
pub struct FLSH_SBUS_W<'a> {
    w: &'a mut W,
}
impl<'a> FLSH_SBUS_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 2)) | (((value as u32) & 0x03) << 2);
        self.w
    }
}
#[doc = "Reader of field `FLSH_DMA0`"]
pub type FLSH_DMA0_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `FLSH_DMA0`"]
pub struct FLSH_DMA0_W<'a> {
    w: &'a mut W,
}
impl<'a> FLSH_DMA0_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 4)) | (((value as u32) & 0x03) << 4);
        self.w
    }
}
#[doc = "Reader of field `SRAM0_DCODE`"]
pub type SRAM0_DCODE_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `SRAM0_DCODE`"]
pub struct SRAM0_DCODE_W<'a> {
    w: &'a mut W,
}
impl<'a> SRAM0_DCODE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 16)) | (((value as u32) & 0x03) << 16);
        self.w
    }
}
#[doc = "Reader of field `SRAM0_SBUS`"]
pub type SRAM0_SBUS_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `SRAM0_SBUS`"]
pub struct SRAM0_SBUS_W<'a> {
    w: &'a mut W,
}
impl<'a> SRAM0_SBUS_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 18)) | (((value as u32) & 0x03) << 18);
        self.w
    }
}
#[doc = "Reader of field `SRAM0_DMA0`"]
pub type SRAM0_DMA0_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `SRAM0_DMA0`"]
pub struct SRAM0_DMA0_W<'a> {
    w: &'a mut W,
}
impl<'a> SRAM0_DMA0_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 20)) | (((value as u32) & 0x03) << 20);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:1 - Flash priority for DCODE"]
    #[inline(always)]
    pub fn flsh_dcode(&self) -> FLSH_DCODE_R {
        FLSH_DCODE_R::new((self.bits & 0x03) as u8)
    }
    #[doc = "Bits 2:3 - Flash priority for SBUS"]
    #[inline(always)]
    pub fn flsh_sbus(&self) -> FLSH_SBUS_R {
        FLSH_SBUS_R::new(((self.bits >> 2) & 0x03) as u8)
    }
    #[doc = "Bits 4:5 - Flash priority for DMA0"]
    #[inline(always)]
    pub fn flsh_dma0(&self) -> FLSH_DMA0_R {
        FLSH_DMA0_R::new(((self.bits >> 4) & 0x03) as u8)
    }
    #[doc = "Bits 16:17 - SRAM0 priority for Dcode"]
    #[inline(always)]
    pub fn sram0_dcode(&self) -> SRAM0_DCODE_R {
        SRAM0_DCODE_R::new(((self.bits >> 16) & 0x03) as u8)
    }
    #[doc = "Bits 18:19 - SRAM0 priority for SBUS"]
    #[inline(always)]
    pub fn sram0_sbus(&self) -> SRAM0_SBUS_R {
        SRAM0_SBUS_R::new(((self.bits >> 18) & 0x03) as u8)
    }
    #[doc = "Bits 20:21 - SRAM0 priority for DMA0"]
    #[inline(always)]
    pub fn sram0_dma0(&self) -> SRAM0_DMA0_R {
        SRAM0_DMA0_R::new(((self.bits >> 20) & 0x03) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Flash priority for DCODE"]
    #[inline(always)]
    pub fn flsh_dcode(&mut self) -> FLSH_DCODE_W {
        FLSH_DCODE_W { w: self }
    }
    #[doc = "Bits 2:3 - Flash priority for SBUS"]
    #[inline(always)]
    pub fn flsh_sbus(&mut self) -> FLSH_SBUS_W {
        FLSH_SBUS_W { w: self }
    }
    #[doc = "Bits 4:5 - Flash priority for DMA0"]
    #[inline(always)]
    pub fn flsh_dma0(&mut self) -> FLSH_DMA0_W {
        FLSH_DMA0_W { w: self }
    }
    #[doc = "Bits 16:17 - SRAM0 priority for Dcode"]
    #[inline(always)]
    pub fn sram0_dcode(&mut self) -> SRAM0_DCODE_W {
        SRAM0_DCODE_W { w: self }
    }
    #[doc = "Bits 18:19 - SRAM0 priority for SBUS"]
    #[inline(always)]
    pub fn sram0_sbus(&mut self) -> SRAM0_SBUS_W {
        SRAM0_SBUS_W { w: self }
    }
    #[doc = "Bits 20:21 - SRAM0 priority for DMA0"]
    #[inline(always)]
    pub fn sram0_dma0(&mut self) -> SRAM0_DMA0_W {
        SRAM0_DMA0_W { w: self }
    }
}
