#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - ADC Configuration"]
    pub cfg: CFG,
    _reserved1: [u8; 2usize],
    #[doc = "0x04 - ADC Power-up Time"]
    pub pwrup: PWRUP,
    _reserved2: [u8; 2usize],
    #[doc = "0x08 - Calibration Word"]
    pub cal_word: CAL_WORD,
    _reserved3: [u8; 2usize],
    #[doc = "0x0c - ADC Conversion Configuration"]
    pub cnv_cfg: CNV_CFG,
    _reserved4: [u8; 2usize],
    #[doc = "0x10 - ADC Conversion Time"]
    pub cnv_time: CNV_TIME,
    _reserved5: [u8; 2usize],
    #[doc = "0x14 - Averaging Configuration"]
    pub avg_cfg: AVG_CFG,
    _reserved6: [u8; 10usize],
    #[doc = "0x20 - Interrupt Enable"]
    pub irq_en: IRQ_EN,
    _reserved7: [u8; 2usize],
    #[doc = "0x24 - ADC Status"]
    pub stat: STAT,
    _reserved8: [u8; 2usize],
    #[doc = "0x28 - Overflow of Output Registers"]
    pub ovf: OVF,
    _reserved9: [u8; 2usize],
    #[doc = "0x2c - Alert Indication"]
    pub alert: ALERT,
    _reserved10: [u8; 2usize],
    #[doc = "0x30 - Conversion Result Channel 0"]
    pub ch0_out: CH0_OUT,
    _reserved11: [u8; 2usize],
    #[doc = "0x34 - Conversion Result Channel 1"]
    pub ch1_out: CH1_OUT,
    _reserved12: [u8; 2usize],
    #[doc = "0x38 - Conversion Result Channel 2"]
    pub ch2_out: CH2_OUT,
    _reserved13: [u8; 2usize],
    #[doc = "0x3c - Conversion Result Channel 3"]
    pub ch3_out: CH3_OUT,
    _reserved14: [u8; 2usize],
    #[doc = "0x40 - Conversion Result Channel 4"]
    pub ch4_out: CH4_OUT,
    _reserved15: [u8; 2usize],
    #[doc = "0x44 - Conversion Result Channel 5"]
    pub ch5_out: CH5_OUT,
    _reserved16: [u8; 2usize],
    #[doc = "0x48 - Conversion Result Channel 6"]
    pub ch6_out: CH6_OUT,
    _reserved17: [u8; 2usize],
    #[doc = "0x4c - Conversion Result Channel 7"]
    pub ch7_out: CH7_OUT,
    _reserved18: [u8; 2usize],
    #[doc = "0x50 - Battery Monitoring Result"]
    pub bat_out: BAT_OUT,
    _reserved19: [u8; 2usize],
    #[doc = "0x54 - Temperature Result"]
    pub tmp_out: TMP_OUT,
    _reserved20: [u8; 2usize],
    #[doc = "0x58 - Temperature Result 2"]
    pub tmp2_out: TMP2_OUT,
    _reserved21: [u8; 2usize],
    #[doc = "0x5c - DMA Output Register"]
    pub dma_out: DMA_OUT,
    _reserved22: [u8; 2usize],
    #[doc = "0x60 - Channel 0 Low Limit"]
    pub lim0_lo: LIM0_LO,
    _reserved23: [u8; 2usize],
    #[doc = "0x64 - Channel 0 High Limit"]
    pub lim0_hi: LIM0_HI,
    _reserved24: [u8; 2usize],
    #[doc = "0x68 - Channel 0 Hysteresis"]
    pub hys0: HYS0,
    _reserved25: [u8; 6usize],
    #[doc = "0x70 - Channel 1 Low Limit"]
    pub lim1_lo: LIM1_LO,
    _reserved26: [u8; 2usize],
    #[doc = "0x74 - Channel 1 High Limit"]
    pub lim1_hi: LIM1_HI,
    _reserved27: [u8; 2usize],
    #[doc = "0x78 - Channel 1 Hysteresis"]
    pub hys1: HYS1,
    _reserved28: [u8; 6usize],
    #[doc = "0x80 - Channel 2 Low Limit"]
    pub lim2_lo: LIM2_LO,
    _reserved29: [u8; 2usize],
    #[doc = "0x84 - Channel 2 High Limit"]
    pub lim2_hi: LIM2_HI,
    _reserved30: [u8; 2usize],
    #[doc = "0x88 - Channel 2 Hysteresis"]
    pub hys2: HYS2,
    _reserved31: [u8; 6usize],
    #[doc = "0x90 - Channel 3 Low Limit"]
    pub lim3_lo: LIM3_LO,
    _reserved32: [u8; 2usize],
    #[doc = "0x94 - Channel 3 High Limit"]
    pub lim3_hi: LIM3_HI,
    _reserved33: [u8; 2usize],
    #[doc = "0x98 - Channel 3 Hysteresis"]
    pub hys3: HYS3,
    _reserved34: [u8; 38usize],
    #[doc = "0xc0 - Reference Buffer Low Power Mode"]
    pub cfg1: CFG1,
}
#[doc = "ADC Configuration\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub type CFG = crate::Reg<u16, _CFG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CFG;
#[doc = "`read()` method returns [cfg::R](cfg::R) reader structure"]
impl crate::Readable for CFG {}
#[doc = "`write(|w| ..)` method takes [cfg::W](cfg::W) writer structure"]
impl crate::Writable for CFG {}
#[doc = "ADC Configuration"]
pub mod cfg;
#[doc = "ADC Power-up Time\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [pwrup](pwrup) module"]
pub type PWRUP = crate::Reg<u16, _PWRUP>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _PWRUP;
#[doc = "`read()` method returns [pwrup::R](pwrup::R) reader structure"]
impl crate::Readable for PWRUP {}
#[doc = "`write(|w| ..)` method takes [pwrup::W](pwrup::W) writer structure"]
impl crate::Writable for PWRUP {}
#[doc = "ADC Power-up Time"]
pub mod pwrup;
#[doc = "Calibration Word\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [cal_word](cal_word) module"]
pub type CAL_WORD = crate::Reg<u16, _CAL_WORD>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CAL_WORD;
#[doc = "`read()` method returns [cal_word::R](cal_word::R) reader structure"]
impl crate::Readable for CAL_WORD {}
#[doc = "`write(|w| ..)` method takes [cal_word::W](cal_word::W) writer structure"]
impl crate::Writable for CAL_WORD {}
#[doc = "Calibration Word"]
pub mod cal_word;
#[doc = "ADC Conversion Configuration\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [cnv_cfg](cnv_cfg) module"]
pub type CNV_CFG = crate::Reg<u16, _CNV_CFG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CNV_CFG;
#[doc = "`read()` method returns [cnv_cfg::R](cnv_cfg::R) reader structure"]
impl crate::Readable for CNV_CFG {}
#[doc = "`write(|w| ..)` method takes [cnv_cfg::W](cnv_cfg::W) writer structure"]
impl crate::Writable for CNV_CFG {}
#[doc = "ADC Conversion Configuration"]
pub mod cnv_cfg;
#[doc = "ADC Conversion Time\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [cnv_time](cnv_time) module"]
pub type CNV_TIME = crate::Reg<u16, _CNV_TIME>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CNV_TIME;
#[doc = "`read()` method returns [cnv_time::R](cnv_time::R) reader structure"]
impl crate::Readable for CNV_TIME {}
#[doc = "`write(|w| ..)` method takes [cnv_time::W](cnv_time::W) writer structure"]
impl crate::Writable for CNV_TIME {}
#[doc = "ADC Conversion Time"]
pub mod cnv_time;
#[doc = "Averaging Configuration\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [avg_cfg](avg_cfg) module"]
pub type AVG_CFG = crate::Reg<u16, _AVG_CFG>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _AVG_CFG;
#[doc = "`read()` method returns [avg_cfg::R](avg_cfg::R) reader structure"]
impl crate::Readable for AVG_CFG {}
#[doc = "`write(|w| ..)` method takes [avg_cfg::W](avg_cfg::W) writer structure"]
impl crate::Writable for AVG_CFG {}
#[doc = "Averaging Configuration"]
pub mod avg_cfg;
#[doc = "Interrupt Enable\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [irq_en](irq_en) module"]
pub type IRQ_EN = crate::Reg<u16, _IRQ_EN>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _IRQ_EN;
#[doc = "`read()` method returns [irq_en::R](irq_en::R) reader structure"]
impl crate::Readable for IRQ_EN {}
#[doc = "`write(|w| ..)` method takes [irq_en::W](irq_en::W) writer structure"]
impl crate::Writable for IRQ_EN {}
#[doc = "Interrupt Enable"]
pub mod irq_en;
#[doc = "ADC Status\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [stat](stat) module"]
pub type STAT = crate::Reg<u16, _STAT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _STAT;
#[doc = "`read()` method returns [stat::R](stat::R) reader structure"]
impl crate::Readable for STAT {}
#[doc = "`write(|w| ..)` method takes [stat::W](stat::W) writer structure"]
impl crate::Writable for STAT {}
#[doc = "ADC Status"]
pub mod stat;
#[doc = "Overflow of Output Registers\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ovf](ovf) module"]
pub type OVF = crate::Reg<u16, _OVF>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _OVF;
#[doc = "`read()` method returns [ovf::R](ovf::R) reader structure"]
impl crate::Readable for OVF {}
#[doc = "`write(|w| ..)` method takes [ovf::W](ovf::W) writer structure"]
impl crate::Writable for OVF {}
#[doc = "Overflow of Output Registers"]
pub mod ovf;
#[doc = "Alert Indication\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [alert](alert) module"]
pub type ALERT = crate::Reg<u16, _ALERT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _ALERT;
#[doc = "`read()` method returns [alert::R](alert::R) reader structure"]
impl crate::Readable for ALERT {}
#[doc = "`write(|w| ..)` method takes [alert::W](alert::W) writer structure"]
impl crate::Writable for ALERT {}
#[doc = "Alert Indication"]
pub mod alert;
#[doc = "Conversion Result Channel 0\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ch0_out](ch0_out) module"]
pub type CH0_OUT = crate::Reg<u16, _CH0_OUT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CH0_OUT;
#[doc = "`read()` method returns [ch0_out::R](ch0_out::R) reader structure"]
impl crate::Readable for CH0_OUT {}
#[doc = "Conversion Result Channel 0"]
pub mod ch0_out;
#[doc = "Conversion Result Channel 1\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ch1_out](ch1_out) module"]
pub type CH1_OUT = crate::Reg<u16, _CH1_OUT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CH1_OUT;
#[doc = "`read()` method returns [ch1_out::R](ch1_out::R) reader structure"]
impl crate::Readable for CH1_OUT {}
#[doc = "Conversion Result Channel 1"]
pub mod ch1_out;
#[doc = "Conversion Result Channel 2\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ch2_out](ch2_out) module"]
pub type CH2_OUT = crate::Reg<u16, _CH2_OUT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CH2_OUT;
#[doc = "`read()` method returns [ch2_out::R](ch2_out::R) reader structure"]
impl crate::Readable for CH2_OUT {}
#[doc = "Conversion Result Channel 2"]
pub mod ch2_out;
#[doc = "Conversion Result Channel 3\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ch3_out](ch3_out) module"]
pub type CH3_OUT = crate::Reg<u16, _CH3_OUT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CH3_OUT;
#[doc = "`read()` method returns [ch3_out::R](ch3_out::R) reader structure"]
impl crate::Readable for CH3_OUT {}
#[doc = "Conversion Result Channel 3"]
pub mod ch3_out;
#[doc = "Conversion Result Channel 4\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ch4_out](ch4_out) module"]
pub type CH4_OUT = crate::Reg<u16, _CH4_OUT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CH4_OUT;
#[doc = "`read()` method returns [ch4_out::R](ch4_out::R) reader structure"]
impl crate::Readable for CH4_OUT {}
#[doc = "Conversion Result Channel 4"]
pub mod ch4_out;
#[doc = "Conversion Result Channel 5\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ch5_out](ch5_out) module"]
pub type CH5_OUT = crate::Reg<u16, _CH5_OUT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CH5_OUT;
#[doc = "`read()` method returns [ch5_out::R](ch5_out::R) reader structure"]
impl crate::Readable for CH5_OUT {}
#[doc = "Conversion Result Channel 5"]
pub mod ch5_out;
#[doc = "Conversion Result Channel 6\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ch6_out](ch6_out) module"]
pub type CH6_OUT = crate::Reg<u16, _CH6_OUT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CH6_OUT;
#[doc = "`read()` method returns [ch6_out::R](ch6_out::R) reader structure"]
impl crate::Readable for CH6_OUT {}
#[doc = "Conversion Result Channel 6"]
pub mod ch6_out;
#[doc = "Conversion Result Channel 7\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ch7_out](ch7_out) module"]
pub type CH7_OUT = crate::Reg<u16, _CH7_OUT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CH7_OUT;
#[doc = "`read()` method returns [ch7_out::R](ch7_out::R) reader structure"]
impl crate::Readable for CH7_OUT {}
#[doc = "Conversion Result Channel 7"]
pub mod ch7_out;
#[doc = "Battery Monitoring Result\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [bat_out](bat_out) module"]
pub type BAT_OUT = crate::Reg<u16, _BAT_OUT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _BAT_OUT;
#[doc = "`read()` method returns [bat_out::R](bat_out::R) reader structure"]
impl crate::Readable for BAT_OUT {}
#[doc = "Battery Monitoring Result"]
pub mod bat_out;
#[doc = "Temperature Result\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [tmp_out](tmp_out) module"]
pub type TMP_OUT = crate::Reg<u16, _TMP_OUT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _TMP_OUT;
#[doc = "`read()` method returns [tmp_out::R](tmp_out::R) reader structure"]
impl crate::Readable for TMP_OUT {}
#[doc = "Temperature Result"]
pub mod tmp_out;
#[doc = "Temperature Result 2\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [tmp2_out](tmp2_out) module"]
pub type TMP2_OUT = crate::Reg<u16, _TMP2_OUT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _TMP2_OUT;
#[doc = "`read()` method returns [tmp2_out::R](tmp2_out::R) reader structure"]
impl crate::Readable for TMP2_OUT {}
#[doc = "Temperature Result 2"]
pub mod tmp2_out;
#[doc = "DMA Output Register\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [dma_out](dma_out) module"]
pub type DMA_OUT = crate::Reg<u16, _DMA_OUT>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _DMA_OUT;
#[doc = "`read()` method returns [dma_out::R](dma_out::R) reader structure"]
impl crate::Readable for DMA_OUT {}
#[doc = "DMA Output Register"]
pub mod dma_out;
#[doc = "Channel 0 Low Limit\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [lim0_lo](lim0_lo) module"]
pub type LIM0_LO = crate::Reg<u16, _LIM0_LO>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _LIM0_LO;
#[doc = "`read()` method returns [lim0_lo::R](lim0_lo::R) reader structure"]
impl crate::Readable for LIM0_LO {}
#[doc = "`write(|w| ..)` method takes [lim0_lo::W](lim0_lo::W) writer structure"]
impl crate::Writable for LIM0_LO {}
#[doc = "Channel 0 Low Limit"]
pub mod lim0_lo;
#[doc = "Channel 0 High Limit\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [lim0_hi](lim0_hi) module"]
pub type LIM0_HI = crate::Reg<u16, _LIM0_HI>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _LIM0_HI;
#[doc = "`read()` method returns [lim0_hi::R](lim0_hi::R) reader structure"]
impl crate::Readable for LIM0_HI {}
#[doc = "`write(|w| ..)` method takes [lim0_hi::W](lim0_hi::W) writer structure"]
impl crate::Writable for LIM0_HI {}
#[doc = "Channel 0 High Limit"]
pub mod lim0_hi;
#[doc = "Channel 0 Hysteresis\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [hys0](hys0) module"]
pub type HYS0 = crate::Reg<u16, _HYS0>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _HYS0;
#[doc = "`read()` method returns [hys0::R](hys0::R) reader structure"]
impl crate::Readable for HYS0 {}
#[doc = "`write(|w| ..)` method takes [hys0::W](hys0::W) writer structure"]
impl crate::Writable for HYS0 {}
#[doc = "Channel 0 Hysteresis"]
pub mod hys0;
#[doc = "Channel 1 Low Limit\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [lim1_lo](lim1_lo) module"]
pub type LIM1_LO = crate::Reg<u16, _LIM1_LO>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _LIM1_LO;
#[doc = "`read()` method returns [lim1_lo::R](lim1_lo::R) reader structure"]
impl crate::Readable for LIM1_LO {}
#[doc = "`write(|w| ..)` method takes [lim1_lo::W](lim1_lo::W) writer structure"]
impl crate::Writable for LIM1_LO {}
#[doc = "Channel 1 Low Limit"]
pub mod lim1_lo;
#[doc = "Channel 1 High Limit\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [lim1_hi](lim1_hi) module"]
pub type LIM1_HI = crate::Reg<u16, _LIM1_HI>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _LIM1_HI;
#[doc = "`read()` method returns [lim1_hi::R](lim1_hi::R) reader structure"]
impl crate::Readable for LIM1_HI {}
#[doc = "`write(|w| ..)` method takes [lim1_hi::W](lim1_hi::W) writer structure"]
impl crate::Writable for LIM1_HI {}
#[doc = "Channel 1 High Limit"]
pub mod lim1_hi;
#[doc = "Channel 1 Hysteresis\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [hys1](hys1) module"]
pub type HYS1 = crate::Reg<u16, _HYS1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _HYS1;
#[doc = "`read()` method returns [hys1::R](hys1::R) reader structure"]
impl crate::Readable for HYS1 {}
#[doc = "`write(|w| ..)` method takes [hys1::W](hys1::W) writer structure"]
impl crate::Writable for HYS1 {}
#[doc = "Channel 1 Hysteresis"]
pub mod hys1;
#[doc = "Channel 2 Low Limit\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [lim2_lo](lim2_lo) module"]
pub type LIM2_LO = crate::Reg<u16, _LIM2_LO>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _LIM2_LO;
#[doc = "`read()` method returns [lim2_lo::R](lim2_lo::R) reader structure"]
impl crate::Readable for LIM2_LO {}
#[doc = "`write(|w| ..)` method takes [lim2_lo::W](lim2_lo::W) writer structure"]
impl crate::Writable for LIM2_LO {}
#[doc = "Channel 2 Low Limit"]
pub mod lim2_lo;
#[doc = "Channel 2 High Limit\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [lim2_hi](lim2_hi) module"]
pub type LIM2_HI = crate::Reg<u16, _LIM2_HI>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _LIM2_HI;
#[doc = "`read()` method returns [lim2_hi::R](lim2_hi::R) reader structure"]
impl crate::Readable for LIM2_HI {}
#[doc = "`write(|w| ..)` method takes [lim2_hi::W](lim2_hi::W) writer structure"]
impl crate::Writable for LIM2_HI {}
#[doc = "Channel 2 High Limit"]
pub mod lim2_hi;
#[doc = "Channel 2 Hysteresis\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [hys2](hys2) module"]
pub type HYS2 = crate::Reg<u16, _HYS2>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _HYS2;
#[doc = "`read()` method returns [hys2::R](hys2::R) reader structure"]
impl crate::Readable for HYS2 {}
#[doc = "`write(|w| ..)` method takes [hys2::W](hys2::W) writer structure"]
impl crate::Writable for HYS2 {}
#[doc = "Channel 2 Hysteresis"]
pub mod hys2;
#[doc = "Channel 3 Low Limit\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [lim3_lo](lim3_lo) module"]
pub type LIM3_LO = crate::Reg<u16, _LIM3_LO>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _LIM3_LO;
#[doc = "`read()` method returns [lim3_lo::R](lim3_lo::R) reader structure"]
impl crate::Readable for LIM3_LO {}
#[doc = "`write(|w| ..)` method takes [lim3_lo::W](lim3_lo::W) writer structure"]
impl crate::Writable for LIM3_LO {}
#[doc = "Channel 3 Low Limit"]
pub mod lim3_lo;
#[doc = "Channel 3 High Limit\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [lim3_hi](lim3_hi) module"]
pub type LIM3_HI = crate::Reg<u16, _LIM3_HI>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _LIM3_HI;
#[doc = "`read()` method returns [lim3_hi::R](lim3_hi::R) reader structure"]
impl crate::Readable for LIM3_HI {}
#[doc = "`write(|w| ..)` method takes [lim3_hi::W](lim3_hi::W) writer structure"]
impl crate::Writable for LIM3_HI {}
#[doc = "Channel 3 High Limit"]
pub mod lim3_hi;
#[doc = "Channel 3 Hysteresis\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [hys3](hys3) module"]
pub type HYS3 = crate::Reg<u16, _HYS3>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _HYS3;
#[doc = "`read()` method returns [hys3::R](hys3::R) reader structure"]
impl crate::Readable for HYS3 {}
#[doc = "`write(|w| ..)` method takes [hys3::W](hys3::W) writer structure"]
impl crate::Writable for HYS3 {}
#[doc = "Channel 3 Hysteresis"]
pub mod hys3;
#[doc = "Reference Buffer Low Power Mode\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [cfg1](cfg1) module"]
pub type CFG1 = crate::Reg<u16, _CFG1>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _CFG1;
#[doc = "`read()` method returns [cfg1::R](cfg1::R) reader structure"]
impl crate::Readable for CFG1 {}
#[doc = "`write(|w| ..)` method takes [cfg1::W](cfg1::W) writer structure"]
impl crate::Writable for CFG1 {}
#[doc = "Reference Buffer Low Power Mode"]
pub mod cfg1;
