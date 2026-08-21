#[doc = "Reader of register DATA"]
pub type R = crate::R<u32, super::DATA>;
#[doc = "Reader of field `VALUE`"]
pub type VALUE_R = crate::R<u8, u8>;
#[doc = "Reader of field `BUFF`"]
pub type BUFF_R = crate::R<u32, u32>;
impl R {
    #[doc = "Bits 0:7 - Value of the CRC Accumulator"]
    #[inline(always)]
    pub fn value(&self) -> VALUE_R {
        VALUE_R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:31 - Buffer for RNG Data"]
    #[inline(always)]
    pub fn buff(&self) -> BUFF_R {
        BUFF_R::new(((self.bits >> 8) & 0x00ff_ffff) as u32)
    }
}
