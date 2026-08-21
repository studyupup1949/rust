#[doc = "Reader of register WR_ABORT_ADDR"]
pub type R = crate::R<u32, super::WR_ABORT_ADDR>;
#[doc = "Reader of field `VALUE`"]
pub type VALUE_R = crate::R<u32, u32>;
impl R {
    #[doc = "Bits 0:31 - Address Targeted by an Ongoing Write Command"]
    #[inline(always)]
    pub fn value(&self) -> VALUE_R {
        VALUE_R::new((self.bits & 0xffff_ffff) as u32)
    }
}
