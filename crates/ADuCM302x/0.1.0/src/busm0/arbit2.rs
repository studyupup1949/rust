#[doc = "Reader of register ARBIT2"]
pub type R = crate::R<u32, super::ARBIT2>;
#[doc = "Writer for register ARBIT2"]
pub type W = crate::W<u32, super::ARBIT2>;
#[doc = "Register ARBIT2 `reset()`'s with value 0x0024_0024"]
impl crate::ResetValue for super::ARBIT2 {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x0024_0024
    }
}
#[doc = "Reader of field `APB32_DCODE`"]
pub type APB32_DCODE_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `APB32_DCODE`"]
pub struct APB32_DCODE_W<'a> {
    w: &'a mut W,
}
impl<'a> APB32_DCODE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !0x03) | ((value as u32) & 0x03);
        self.w
    }
}
#[doc = "Reader of field `APB32_SBUS`"]
pub type APB32_SBUS_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `APB32_SBUS`"]
pub struct APB32_SBUS_W<'a> {
    w: &'a mut W,
}
impl<'a> APB32_SBUS_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 2)) | (((value as u32) & 0x03) << 2);
        self.w
    }
}
#[doc = "Reader of field `APB32_DMA0`"]
pub type APB32_DMA0_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `APB32_DMA0`"]
pub struct APB32_DMA0_W<'a> {
    w: &'a mut W,
}
impl<'a> APB32_DMA0_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 4)) | (((value as u32) & 0x03) << 4);
        self.w
    }
}
#[doc = "Reader of field `APB16_DCODE`"]
pub type APB16_DCODE_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `APB16_DCODE`"]
pub struct APB16_DCODE_W<'a> {
    w: &'a mut W,
}
impl<'a> APB16_DCODE_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 16)) | (((value as u32) & 0x03) << 16);
        self.w
    }
}
#[doc = "Reader of field `APB16_SBUS`"]
pub type APB16_SBUS_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `APB16_SBUS`"]
pub struct APB16_SBUS_W<'a> {
    w: &'a mut W,
}
impl<'a> APB16_SBUS_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 18)) | (((value as u32) & 0x03) << 18);
        self.w
    }
}
#[doc = "Reader of field `APB16_DMA0`"]
pub type APB16_DMA0_R = crate::R<u8, u8>;
#[doc = "Write proxy for field `APB16_DMA0`"]
pub struct APB16_DMA0_W<'a> {
    w: &'a mut W,
}
impl<'a> APB16_DMA0_W<'a> {
    #[doc = r"Writes raw bits to the field"]
    #[inline(always)]
    pub fn bits(self, value: u8) -> &'a mut W {
        self.w.bits = (self.w.bits & !(0x03 << 20)) | (((value as u32) & 0x03) << 20);
        self.w
    }
}
impl R {
    #[doc = "Bits 0:1 - APB32 priority for DCODE"]
    #[inline(always)]
    pub fn apb32_dcode(&self) -> APB32_DCODE_R {
        APB32_DCODE_R::new((self.bits & 0x03) as u8)
    }
    #[doc = "Bits 2:3 - APB32 priority for SBUS"]
    #[inline(always)]
    pub fn apb32_sbus(&self) -> APB32_SBUS_R {
        APB32_SBUS_R::new(((self.bits >> 2) & 0x03) as u8)
    }
    #[doc = "Bits 4:5 - APB32 priority for DMA0"]
    #[inline(always)]
    pub fn apb32_dma0(&self) -> APB32_DMA0_R {
        APB32_DMA0_R::new(((self.bits >> 4) & 0x03) as u8)
    }
    #[doc = "Bits 16:17 - APB16 priority for DCODE"]
    #[inline(always)]
    pub fn apb16_dcode(&self) -> APB16_DCODE_R {
        APB16_DCODE_R::new(((self.bits >> 16) & 0x03) as u8)
    }
    #[doc = "Bits 18:19 - APB16 priority for SBUS"]
    #[inline(always)]
    pub fn apb16_sbus(&self) -> APB16_SBUS_R {
        APB16_SBUS_R::new(((self.bits >> 18) & 0x03) as u8)
    }
    #[doc = "Bits 20:21 - APB16 priority for DMA0"]
    #[inline(always)]
    pub fn apb16_dma0(&self) -> APB16_DMA0_R {
        APB16_DMA0_R::new(((self.bits >> 20) & 0x03) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - APB32 priority for DCODE"]
    #[inline(always)]
    pub fn apb32_dcode(&mut self) -> APB32_DCODE_W {
        APB32_DCODE_W { w: self }
    }
    #[doc = "Bits 2:3 - APB32 priority for SBUS"]
    #[inline(always)]
    pub fn apb32_sbus(&mut self) -> APB32_SBUS_W {
        APB32_SBUS_W { w: self }
    }
    #[doc = "Bits 4:5 - APB32 priority for DMA0"]
    #[inline(always)]
    pub fn apb32_dma0(&mut self) -> APB32_DMA0_W {
        APB32_DMA0_W { w: self }
    }
    #[doc = "Bits 16:17 - APB16 priority for DCODE"]
    #[inline(always)]
    pub fn apb16_dcode(&mut self) -> APB16_DCODE_W {
        APB16_DCODE_W { w: self }
    }
    #[doc = "Bits 18:19 - APB16 priority for SBUS"]
    #[inline(always)]
    pub fn apb16_sbus(&mut self) -> APB16_SBUS_W {
        APB16_SBUS_W { w: self }
    }
    #[doc = "Bits 20:21 - APB16 priority for DMA0"]
    #[inline(always)]
    pub fn apb16_dma0(&mut self) -> APB16_DMA0_W {
        APB16_DMA0_W { w: self }
    }
}
