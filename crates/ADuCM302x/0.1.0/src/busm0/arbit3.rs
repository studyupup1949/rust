#[doc = "Reader of register ARBIT3"]
pub type R = crate::R<u32, super::ARBIT3>;
#[doc = "Writer for register ARBIT3"]
pub type W = crate::W<u32, super::ARBIT3>;
#[doc = "Register ARBIT3 `reset()`'s with value 0x0001_0002"]
impl crate::ResetValue for super::ARBIT3 {
    type Type = u32;
    #[inline(always)]
    fn reset_value() -> Self::Type {
        0x0001_0002
    }
}
#[doc = "Reader of field `APB16_CORE`"]
pub type APB16_CORE_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `APB16_CORE`"]
pub struct APB16_CORE_W<'a> {
    w: &'a mut W,
}
impl<'a> APB16_CORE_W<'a> {
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
#[doc = "Reader of field `APB16_DMA1`"]
pub type APB16_DMA1_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `APB16_DMA1`"]
pub struct APB16_DMA1_W<'a> {
    w: &'a mut W,
}
impl<'a> APB16_DMA1_W<'a> {
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
#[doc = "Reader of field `APB16_4DMA_CORE`"]
pub type APB16_4DMA_CORE_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `APB16_4DMA_CORE`"]
pub struct APB16_4DMA_CORE_W<'a> {
    w: &'a mut W,
}
impl<'a> APB16_4DMA_CORE_W<'a> {
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
#[doc = "Reader of field `APB16_4DMA_DMA1`"]
pub type APB16_4DMA_DMA1_R = crate::R<bool, bool>;
#[doc = "Write proxy for field `APB16_4DMA_DMA1`"]
pub struct APB16_4DMA_DMA1_W<'a> {
    w: &'a mut W,
}
impl<'a> APB16_4DMA_DMA1_W<'a> {
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
impl R {
    #[doc = "Bit 0 - APB16 priority for CORE"]
    #[inline(always)]
    pub fn apb16_core(&self) -> APB16_CORE_R {
        APB16_CORE_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bit 1 - APB16 priority for DMA1"]
    #[inline(always)]
    pub fn apb16_dma1(&self) -> APB16_DMA1_R {
        APB16_DMA1_R::new(((self.bits >> 1) & 0x01) != 0)
    }
    #[doc = "Bit 16 - APB16 for dma priority for CORE"]
    #[inline(always)]
    pub fn apb16_4dma_core(&self) -> APB16_4DMA_CORE_R {
        APB16_4DMA_CORE_R::new(((self.bits >> 16) & 0x01) != 0)
    }
    #[doc = "Bit 17 - APB16 for dma priority for DMA1"]
    #[inline(always)]
    pub fn apb16_4dma_dma1(&self) -> APB16_4DMA_DMA1_R {
        APB16_4DMA_DMA1_R::new(((self.bits >> 17) & 0x01) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - APB16 priority for CORE"]
    #[inline(always)]
    pub fn apb16_core(&mut self) -> APB16_CORE_W {
        APB16_CORE_W { w: self }
    }
    #[doc = "Bit 1 - APB16 priority for DMA1"]
    #[inline(always)]
    pub fn apb16_dma1(&mut self) -> APB16_DMA1_W {
        APB16_DMA1_W { w: self }
    }
    #[doc = "Bit 16 - APB16 for dma priority for CORE"]
    #[inline(always)]
    pub fn apb16_4dma_core(&mut self) -> APB16_4DMA_CORE_W {
        APB16_4DMA_CORE_W { w: self }
    }
    #[doc = "Bit 17 - APB16 for dma priority for DMA1"]
    #[inline(always)]
    pub fn apb16_4dma_dma1(&mut self) -> APB16_4DMA_DMA1_W {
        APB16_4DMA_DMA1_W { w: self }
    }
}
