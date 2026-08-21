#[doc = "Reader of register SS1TGT"]
pub type R = crate::R<u16, super::SS1TGT>;
#[doc = "Reader of field `SS1TGT`"]
pub type SS1TGT_R = crate::R<u16, u16>;
impl R {
    #[doc = "Bits 0:15 - Current Target Value for the SensorStrobe Channel 1"]
    #[inline(always)]
    pub fn ss1tgt(&self) -> SS1TGT_R {
        SS1TGT_R::new((self.bits & 0xffff) as u16)
    }
}
