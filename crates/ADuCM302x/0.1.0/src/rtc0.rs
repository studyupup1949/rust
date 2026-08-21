#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - RTC Control 0"]
    pub cr0: CR0,
    _reserved1: [u8; 2usize],
    #[doc = "0x04 - RTC Status 0"]
    pub sr0: SR0,
    _reserved2: [u8; 2usize],
    #[doc = "0x08 - RTC Status 1"]
    pub sr1: SR1,
    _reserved3: [u8; 2usize],
    #[doc = "0x0c - RTC Count 0"]
    pub cnt0: CNT0,
    _reserved4: [u8; 2usize],
    #[doc = "0x10 - RTC Count 1"]
    pub cnt1: CNT1,
    _reserved5: [u8; 2usize],
    #[doc = "0x14 - RTC Alarm 0"]
    pub alm0: ALM0,
    _reserved6: [u8; 2usize],
    #[doc = "0x18 - RTC Alarm 1"]
    pub alm1: ALM1,
    _reserved7: [u8; 2usize],
    #[doc = "0x1c - RTC Trim"]
    pub trm: TRM,
    _reserved8: [u8; 2usize],
    #[doc = "0x20 - RTC Gateway"]
    pub gwy: GWY,
    _reserved9: [u8; 6usize],
    #[doc = "0x28 - RTC Control 1"]
    pub cr1: CR1,
    _reserved10: [u8; 2usize],
    #[doc = "0x2c - RTC Status 2"]
    pub sr2: SR2,
    _reserved11: [u8; 2usize],
    #[doc = "0x30 - RTC Snapshot 0"]
    pub snap0: SNAP0,
    _reserved12: [u8; 2usize],
    #[doc = "0x34 - RTC Snapshot 1"]
    pub snap1: SNAP1,
    _reserved13: [u8; 2usize],
    #[doc = "0x38 - RTC Snapshot 2"]
    pub snap2: SNAP2,
    _reserved14: [u8; 2usize],
    #[doc = "0x3c - RTC Modulo"]
    pub mod_: MOD,
    _reserved15: [u8; 2usize],
    #[doc = "0x40 - RTC Count 2"]
    pub cnt2: CNT2,
    _reserved16: [u8; 2usize],
    #[doc = "0x44 - RTC Alarm 2"]
    pub alm2: ALM2,
    _reserved17: [u8; 2usize],
    #[doc = "0x48 - RTC Status 3"]
    pub sr3: SR3,
    _reserved18: [u8; 2usize],
    #[doc = "0x4c - RTC Control 2 for Configuring Input Capture Channels"]
    pub cr2ic: CR2IC,
    _reserved19: [u8; 2usize],
    #[doc = "0x50 - RTC Control 3 for Configuring SensorStrobe Channel"]
    pub cr3ss: CR3SS,
    _reserved20: [u8; 2usize],
    #[doc = "0x54 - RTC Control 4 for Configuring SensorStrobe Channel"]
    pub cr4ss: CR4SS,
    _reserved21: [u8; 2usize],
    #[doc = "0x58 - RTC Mask for SensorStrobe Channel"]
    pub ssmsk: SSMSK,
    _reserved22: [u8; 2usize],
    #[doc = "0x5c - RTC Auto-Reload for SensorStrobe Channel 1"]
    pub ss1arl: SS1ARL,
    _reserved23: [u8; 6usize],
    #[doc = "0x64 - RTC Input Capture Channel 2"]
    pub ic2: IC2,
    _reserved24: [u8; 2usize],
    #[doc = "0x68 - RTC Input Capture Channel 3"]
    pub ic3: IC3,
    _reserved25: [u8; 2usize],
    #[doc = "0x6c - RTC Input Capture Channel 4"]
    pub ic4: IC4,
    _reserved26: [u8; 2usize],
    #[doc = "0x70 - RTC SensorStrobe Channel 1"]
    pub ss1: SS1,
    _reserved27: [u8; 14usize],
    #[doc = "0x80 - RTC Status 4"]
    pub sr4: SR4,
    _reserved28: [u8; 2usize],
    #[doc = "0x84 - RTC Status 5"]
    pub sr5: SR5,
    _reserved29: [u8; 2usize],
    #[doc = "0x88 - RTC Status 6"]
    pub sr6: SR6,
    _reserved30: [u8; 2usize],
    #[doc = "0x8c - RTC SensorStrobe Channel 1 Target"]
    pub ss1tgt: SS1TGT,
    _reserved31: [u8; 2usize],
    #[doc = "0x90 - RTC Freeze Count"]
    pub frzcnt: FRZCNT,
}
#[doc = "RTC Control 0\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [cr0](cr0) module"]
pub type CR0 = crate::Reg<u16, _CR0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CR0;
#[doc = "`read()` method returns [cr0::R](cr0::R) reader structure"]
impl crate::Readable for CR0 {}
#[doc = "`write(|w| ..)` method takes [cr0::W](cr0::W) writer structure"]
impl crate::Writable for CR0 {}
#[doc = "RTC Control 0"]
pub mod cr0;
#[doc = "RTC Status 0\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [sr0](sr0) module"]
pub type SR0 = crate::Reg<u16, _SR0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SR0;
#[doc = "`read()` method returns [sr0::R](sr0::R) reader structure"]
impl crate::Readable for SR0 {}
#[doc = "`write(|w| ..)` method takes [sr0::W](sr0::W) writer structure"]
impl crate::Writable for SR0 {}
#[doc = "RTC Status 0"]
pub mod sr0;
#[doc = "RTC Status 1\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [sr1](sr1) module"]
pub type SR1 = crate::Reg<u16, _SR1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SR1;
#[doc = "`read()` method returns [sr1::R](sr1::R) reader structure"]
impl crate::Readable for SR1 {}
#[doc = "RTC Status 1"]
pub mod sr1;
#[doc = "RTC Count 0\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [cnt0](cnt0) module"]
pub type CNT0 = crate::Reg<u16, _CNT0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CNT0;
#[doc = "`read()` method returns [cnt0::R](cnt0::R) reader structure"]
impl crate::Readable for CNT0 {}
#[doc = "`write(|w| ..)` method takes [cnt0::W](cnt0::W) writer structure"]
impl crate::Writable for CNT0 {}
#[doc = "RTC Count 0"]
pub mod cnt0;
#[doc = "RTC Count 1\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [cnt1](cnt1) module"]
pub type CNT1 = crate::Reg<u16, _CNT1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CNT1;
#[doc = "`read()` method returns [cnt1::R](cnt1::R) reader structure"]
impl crate::Readable for CNT1 {}
#[doc = "`write(|w| ..)` method takes [cnt1::W](cnt1::W) writer structure"]
impl crate::Writable for CNT1 {}
#[doc = "RTC Count 1"]
pub mod cnt1;
#[doc = "RTC Alarm 0\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [alm0](alm0) module"]
pub type ALM0 = crate::Reg<u16, _ALM0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ALM0;
#[doc = "`read()` method returns [alm0::R](alm0::R) reader structure"]
impl crate::Readable for ALM0 {}
#[doc = "`write(|w| ..)` method takes [alm0::W](alm0::W) writer structure"]
impl crate::Writable for ALM0 {}
#[doc = "RTC Alarm 0"]
pub mod alm0;
#[doc = "RTC Alarm 1\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [alm1](alm1) module"]
pub type ALM1 = crate::Reg<u16, _ALM1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ALM1;
#[doc = "`read()` method returns [alm1::R](alm1::R) reader structure"]
impl crate::Readable for ALM1 {}
#[doc = "`write(|w| ..)` method takes [alm1::W](alm1::W) writer structure"]
impl crate::Writable for ALM1 {}
#[doc = "RTC Alarm 1"]
pub mod alm1;
#[doc = "RTC Trim\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [trm](trm) module"]
pub type TRM = crate::Reg<u16, _TRM>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _TRM;
#[doc = "`read()` method returns [trm::R](trm::R) reader structure"]
impl crate::Readable for TRM {}
#[doc = "`write(|w| ..)` method takes [trm::W](trm::W) writer structure"]
impl crate::Writable for TRM {}
#[doc = "RTC Trim"]
pub mod trm;
#[doc = "RTC Gateway\n\nThis register you can [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [gwy](gwy) module"]
pub type GWY = crate::Reg<u16, _GWY>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _GWY;
#[doc = "`write(|w| ..)` method takes [gwy::W](gwy::W) writer structure"]
impl crate::Writable for GWY {}
#[doc = "RTC Gateway"]
pub mod gwy;
#[doc = "RTC Control 1\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [cr1](cr1) module"]
pub type CR1 = crate::Reg<u16, _CR1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CR1;
#[doc = "`read()` method returns [cr1::R](cr1::R) reader structure"]
impl crate::Readable for CR1 {}
#[doc = "`write(|w| ..)` method takes [cr1::W](cr1::W) writer structure"]
impl crate::Writable for CR1 {}
#[doc = "RTC Control 1"]
pub mod cr1;
#[doc = "RTC Status 2\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [sr2](sr2) module"]
pub type SR2 = crate::Reg<u16, _SR2>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SR2;
#[doc = "`read()` method returns [sr2::R](sr2::R) reader structure"]
impl crate::Readable for SR2 {}
#[doc = "`write(|w| ..)` method takes [sr2::W](sr2::W) writer structure"]
impl crate::Writable for SR2 {}
#[doc = "RTC Status 2"]
pub mod sr2;
#[doc = "RTC Snapshot 0\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [snap0](snap0) module"]
pub type SNAP0 = crate::Reg<u16, _SNAP0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SNAP0;
#[doc = "`read()` method returns [snap0::R](snap0::R) reader structure"]
impl crate::Readable for SNAP0 {}
#[doc = "RTC Snapshot 0"]
pub mod snap0;
#[doc = "RTC Snapshot 1\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [snap1](snap1) module"]
pub type SNAP1 = crate::Reg<u16, _SNAP1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SNAP1;
#[doc = "`read()` method returns [snap1::R](snap1::R) reader structure"]
impl crate::Readable for SNAP1 {}
#[doc = "RTC Snapshot 1"]
pub mod snap1;
#[doc = "RTC Snapshot 2\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [snap2](snap2) module"]
pub type SNAP2 = crate::Reg<u16, _SNAP2>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SNAP2;
#[doc = "`read()` method returns [snap2::R](snap2::R) reader structure"]
impl crate::Readable for SNAP2 {}
#[doc = "RTC Snapshot 2"]
pub mod snap2;
#[doc = "RTC Modulo\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [mod_](mod_) module"]
pub type MOD = crate::Reg<u16, _MOD>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _MOD;
#[doc = "`read()` method returns [mod_::R](mod_::R) reader structure"]
impl crate::Readable for MOD {}
#[doc = "RTC Modulo"]
pub mod mod_;
#[doc = "RTC Count 2\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [cnt2](cnt2) module"]
pub type CNT2 = crate::Reg<u16, _CNT2>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CNT2;
#[doc = "`read()` method returns [cnt2::R](cnt2::R) reader structure"]
impl crate::Readable for CNT2 {}
#[doc = "RTC Count 2"]
pub mod cnt2;
#[doc = "RTC Alarm 2\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [alm2](alm2) module"]
pub type ALM2 = crate::Reg<u16, _ALM2>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ALM2;
#[doc = "`read()` method returns [alm2::R](alm2::R) reader structure"]
impl crate::Readable for ALM2 {}
#[doc = "`write(|w| ..)` method takes [alm2::W](alm2::W) writer structure"]
impl crate::Writable for ALM2 {}
#[doc = "RTC Alarm 2"]
pub mod alm2;
#[doc = "RTC Status 3\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [sr3](sr3) module"]
pub type SR3 = crate::Reg<u16, _SR3>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SR3;
#[doc = "`read()` method returns [sr3::R](sr3::R) reader structure"]
impl crate::Readable for SR3 {}
#[doc = "`write(|w| ..)` method takes [sr3::W](sr3::W) writer structure"]
impl crate::Writable for SR3 {}
#[doc = "RTC Status 3"]
pub mod sr3;
#[doc = "RTC Control 2 for Configuring Input Capture Channels\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [cr2ic](cr2ic) module"]
pub type CR2IC = crate::Reg<u16, _CR2IC>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CR2IC;
#[doc = "`read()` method returns [cr2ic::R](cr2ic::R) reader structure"]
impl crate::Readable for CR2IC {}
#[doc = "`write(|w| ..)` method takes [cr2ic::W](cr2ic::W) writer structure"]
impl crate::Writable for CR2IC {}
#[doc = "RTC Control 2 for Configuring Input Capture Channels"]
pub mod cr2ic;
#[doc = "RTC Control 3 for Configuring SensorStrobe Channel\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [cr3ss](cr3ss) module"]
pub type CR3SS = crate::Reg<u16, _CR3SS>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CR3SS;
#[doc = "`read()` method returns [cr3ss::R](cr3ss::R) reader structure"]
impl crate::Readable for CR3SS {}
#[doc = "`write(|w| ..)` method takes [cr3ss::W](cr3ss::W) writer structure"]
impl crate::Writable for CR3SS {}
#[doc = "RTC Control 3 for Configuring SensorStrobe Channel"]
pub mod cr3ss;
#[doc = "RTC Control 4 for Configuring SensorStrobe Channel\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [cr4ss](cr4ss) module"]
pub type CR4SS = crate::Reg<u16, _CR4SS>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CR4SS;
#[doc = "`read()` method returns [cr4ss::R](cr4ss::R) reader structure"]
impl crate::Readable for CR4SS {}
#[doc = "`write(|w| ..)` method takes [cr4ss::W](cr4ss::W) writer structure"]
impl crate::Writable for CR4SS {}
#[doc = "RTC Control 4 for Configuring SensorStrobe Channel"]
pub mod cr4ss;
#[doc = "RTC Mask for SensorStrobe Channel\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ssmsk](ssmsk) module"]
pub type SSMSK = crate::Reg<u16, _SSMSK>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SSMSK;
#[doc = "`read()` method returns [ssmsk::R](ssmsk::R) reader structure"]
impl crate::Readable for SSMSK {}
#[doc = "`write(|w| ..)` method takes [ssmsk::W](ssmsk::W) writer structure"]
impl crate::Writable for SSMSK {}
#[doc = "RTC Mask for SensorStrobe Channel"]
pub mod ssmsk;
#[doc = "RTC Auto-Reload for SensorStrobe Channel 1\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ss1arl](ss1arl) module"]
pub type SS1ARL = crate::Reg<u16, _SS1ARL>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SS1ARL;
#[doc = "`read()` method returns [ss1arl::R](ss1arl::R) reader structure"]
impl crate::Readable for SS1ARL {}
#[doc = "`write(|w| ..)` method takes [ss1arl::W](ss1arl::W) writer structure"]
impl crate::Writable for SS1ARL {}
#[doc = "RTC Auto-Reload for SensorStrobe Channel 1"]
pub mod ss1arl;
#[doc = "RTC Input Capture Channel 2\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ic2](ic2) module"]
pub type IC2 = crate::Reg<u16, _IC2>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _IC2;
#[doc = "`read()` method returns [ic2::R](ic2::R) reader structure"]
impl crate::Readable for IC2 {}
#[doc = "RTC Input Capture Channel 2"]
pub mod ic2;
#[doc = "RTC Input Capture Channel 3\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ic3](ic3) module"]
pub type IC3 = crate::Reg<u16, _IC3>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _IC3;
#[doc = "`read()` method returns [ic3::R](ic3::R) reader structure"]
impl crate::Readable for IC3 {}
#[doc = "RTC Input Capture Channel 3"]
pub mod ic3;
#[doc = "RTC Input Capture Channel 4\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ic4](ic4) module"]
pub type IC4 = crate::Reg<u16, _IC4>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _IC4;
#[doc = "`read()` method returns [ic4::R](ic4::R) reader structure"]
impl crate::Readable for IC4 {}
#[doc = "RTC Input Capture Channel 4"]
pub mod ic4;
#[doc = "RTC SensorStrobe Channel 1\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ss1](ss1) module"]
pub type SS1 = crate::Reg<u16, _SS1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SS1;
#[doc = "`read()` method returns [ss1::R](ss1::R) reader structure"]
impl crate::Readable for SS1 {}
#[doc = "`write(|w| ..)` method takes [ss1::W](ss1::W) writer structure"]
impl crate::Writable for SS1 {}
#[doc = "RTC SensorStrobe Channel 1"]
pub mod ss1;
#[doc = "RTC Status 4\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [sr4](sr4) module"]
pub type SR4 = crate::Reg<u16, _SR4>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SR4;
#[doc = "`read()` method returns [sr4::R](sr4::R) reader structure"]
impl crate::Readable for SR4 {}
#[doc = "RTC Status 4"]
pub mod sr4;
#[doc = "RTC Status 5\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [sr5](sr5) module"]
pub type SR5 = crate::Reg<u16, _SR5>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SR5;
#[doc = "`read()` method returns [sr5::R](sr5::R) reader structure"]
impl crate::Readable for SR5 {}
#[doc = "RTC Status 5"]
pub mod sr5;
#[doc = "RTC Status 6\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [sr6](sr6) module"]
pub type SR6 = crate::Reg<u16, _SR6>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SR6;
#[doc = "`read()` method returns [sr6::R](sr6::R) reader structure"]
impl crate::Readable for SR6 {}
#[doc = "RTC Status 6"]
pub mod sr6;
#[doc = "RTC SensorStrobe Channel 1 Target\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ss1tgt](ss1tgt) module"]
pub type SS1TGT = crate::Reg<u16, _SS1TGT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _SS1TGT;
#[doc = "`read()` method returns [ss1tgt::R](ss1tgt::R) reader structure"]
impl crate::Readable for SS1TGT {}
#[doc = "RTC SensorStrobe Channel 1 Target"]
pub mod ss1tgt;
#[doc = "RTC Freeze Count\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [frzcnt](frzcnt) module"]
pub type FRZCNT = crate::Reg<u16, _FRZCNT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _FRZCNT;
#[doc = "`read()` method returns [frzcnt::R](frzcnt::R) reader structure"]
impl crate::Readable for FRZCNT {}
#[doc = "RTC Freeze Count"]
pub mod frzcnt;
