#[doc = "Reader of register STAT"]
pub type R = crate::R<u32, super::STAT>;
#[doc = "Reader of field `MEN`"]
pub type MEN_R = crate::R<bool, bool>;
#[doc = "Reader of field `CHANM1`"]
pub type CHANM1_R = crate::R<u8, u8>;
impl R {
    #[doc = "Bit 0 - Enable Status of the Controller"]
    #[inline(always)]
    pub fn men(&self) -> MEN_R {
        MEN_R::new((self.bits & 0x01) != 0)
    }
    #[doc = "Bits 16:20 - Number of Available DMA Channels Minus 1"]
    #[inline(always)]
    pub fn chanm1(&self) -> CHANM1_R {
        CHANM1_R::new(((self.bits >> 16) & 0x1f) as u8)
    }
}
