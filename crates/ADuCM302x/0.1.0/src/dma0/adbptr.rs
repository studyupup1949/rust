#[doc = "Reader of register ADBPTR"]
pub type R = crate::R<u32, super::ADBPTR>;
#[doc = "Reader of field `ADDR`"]
pub type ADDR_R = crate::R<u32, u32>;
impl R {
    #[doc = "Bits 0:31 - Base Address of the Alternate Data Structure"]
    #[inline(always)]
    pub fn addr(&self) -> ADDR_R {
        ADDR_R::new((self.bits & 0xffff_ffff) as u32)
    }
}
