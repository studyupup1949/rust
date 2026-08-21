use modular_bitfield::{bitfield, prelude::{B1, B2, B3, B4, B5}, Specifier};
use super::{BW_RATE_ADDR, FIFO_CTL_ADDR, DATA_FORMAT_ADDR, REGISTER_SIZE};

/// Bit field for the BW_RATE register. Configures both the ODR and power consumption settings.
/// 
/// # Fields
/// 
/// - `rate` (`OutputDataRate`) - output data rate (default value 0xA)
/// - `low_power` (`B1`) - 0 selects normal ops and 1 selects reduced power ops (default value 0b0)
/// - `#[skip] __` (`B3`) - non-functional and must remain equal to 0
#[derive(Clone, Copy)]
#[bitfield(bits = 8)]
pub struct BW_RATE{
    pub odr : OutputDataRate,
    pub low_power: B1,
    #[skip]
    __: B3, // reserved/unused bits
}

impl BW_RATE  {
    pub fn address(&self) -> u8 {
        BW_RATE_ADDR
    }
}

impl Default for BW_RATE {
    fn default() -> Self {
        BW_RATE::new().with_odr(OutputDataRate::default())
    }
}

/// 
/// # Variants
/// 
/// - `Hz3200 = 0b1111` 
/// - `Hz1600 = 0b1110`
/// - `Hz800 = 0b1101` 
/// - `Hz400 = 0b1100` 
/// - `Hz200 = 0b1011` 
/// - `#[default] Hz100 = 0b1010`
/// - `Hz50 = 0b1001` 
/// - `Hz25 = 0b1000` 
/// - `Hz12_5 = 0b0111` 
/// - `Hz6_25 = 0b0110` 
/// - `Hz3_13 = 0b0101` 
/// - `Hz1_56 = 0b0100` 
/// - `Hz0_78 = 0b0011` 
/// - `Hz0_39 = 0b0010` 
/// - `Hz0_20 = 0b0001` 
/// - `Hz0_10 = 0b0000` 
#[derive(Default, Specifier, Clone, Copy)]
pub enum OutputDataRate {
    Hz3200 = 0b1111,
    Hz1600 = 0b1110,
    Hz800  = 0b1101,
    Hz400  = 0b1100,
    Hz200  = 0b1011,
    #[default]
    Hz100  = 0b1010,
    Hz50   = 0b1001,
    Hz25   = 0b1000,
    Hz12_5 = 0b0111,
    Hz6_25 = 0b0110,
    Hz3_13 = 0b0101,
    Hz1_56 = 0b0100,
    Hz0_78 = 0b0011,
    Hz0_39 = 0b0010,
    Hz0_20 = 0b0001,
    Hz0_10 = 0b0000,
}

/// Configure whether the device will start measuring or not.
/// 
/// # Fields
/// 
/// - `#[skip] wakeup` (`SLEEP_MODE_ODR`)  
/// - `#[skip] sleep` (`B1`)  
/// - `measure` (`B1`) 
/// - `#[skip] autosleep` (`B1`) 
/// - `#[skip] link` (`B1`) 
/// - `#[skip] __` (`B2`) 

#[bitfield(bits = 8)]
pub struct POWER_CTL{
    #[skip]
    wakeup: SLEEP_MODE_ODR,
    #[skip]
    sleep: B1,

    pub measure: B1, 

    #[skip]
    autosleep: B1,
    #[skip]
    link: B1,
    #[skip]
    __: B2, // reserved/unused bits
}

impl POWER_CTL  {
    pub fn address(&self) -> u8 {
        BW_RATE_ADDR
    }
}

impl Default for POWER_CTL {
    fn default() -> Self {
        POWER_CTL::new()
    }
}

#[derive(Default, Specifier)]
pub enum SLEEP_MODE_ODR{
    #[default]
    _8Hz = 0b00,
    _4Hz = 0b01,
    _2Hz = 0b10,
    _1Hz = 0b11
}

/// FIFO buffer configuration
/// 
/// # Fields
/// 
/// - `#[skip] samples` (`B5`) - must remain zero
/// - `#[skip] samples` (`B1`) - controls the mapping of the trigger event to the interrupt line;
/// 0 -> int line 1, 1 -> int line 2
/// - `fifo_mode` (`FIFO_MODE`) - default, FIFO, STREAM, Trigger

#[bitfield(bits = 8)]
pub struct FIFO_CTL{
    #[skip]
    samples: B5,
    #[skip]
    samples: B1,

    fifo_mode: FIFOMode
}

impl FIFO_CTL  {
    pub fn address(&self) -> u8 {
        FIFO_CTL_ADDR
    }
}

impl Default for FIFO_CTL {
    fn default() -> Self {
        FIFO_CTL::new()
    }
}

/// 
/// # Variants
/// 
/// - `#[default] BYPASS = 0b00` 
/// - `FIFO = 0b01` 
/// - `STREAM = 0b10`
#[derive(Default, Specifier)]
#[bits = 2]
pub enum FIFOMode{
    #[default]
    BYPASS = 0b00,
    FIFO = 0b01,
    STREAM = 0b10,
}

/// Bit field for the DATA_Format register. Configures the range, justification, and resolution
/// 
/// # Fields
/// 
/// - `range` (`AccelRange`) 
/// - `justisfy` (`Alignment`) 
/// - `full_res` (`FullRes`)
/// - `#[skip] __` (`B1`) 
/// - `#[skip] int_invert` (`B1`) 
/// - `#[skip] spi_mode` (`B1`) 
/// - `#[skip] self_test` (`B1`)
#[bitfield(bits = 8)]
pub struct DATA_FORMAT{
    pub range: AccelRange,
    pub justisfy: Alignment,
    pub full_res: FullRes,
    
    #[skip]
    __ : B1,

    #[skip]
    int_invert: B1,

    #[skip]
    pub spi_mode: B1,

    #[skip]
    self_test: B1
}

impl DATA_FORMAT {
    fn address() -> u8{
        DATA_FORMAT_ADDR
    }
}


impl Default for DATA_FORMAT {
    fn default() -> Self {
        DATA_FORMAT::new()
    }
}

/// # Variants
/// 
/// - `#[default] _2g = 0b00` 
/// - `_4g = 0b01` 
/// - `_8g = 0b10` 
/// - `_16g = 0b11`
#[derive(Default, Specifier, Clone, Copy, PartialEq, Eq)]
pub enum AccelRange{
    #[default]
    _2g = 0b00,
    _4g = 0b01,
    _8g = 0b10,
    _16g = 0b11
}

/// # Variants
/// 
/// - `#[default] right = 0b0`
/// - `left = 0b1` 
#[derive(Default, Debug, Specifier, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    #[default]
    right = 0b0,
    left = 0b1
}

/// # Variants
/// 
/// - `#[default] _10bit_res = 0b0` - Describe this variant.
/// - `full_res = 0b1` - Describe this variant.
#[derive(Default, Specifier, Clone, Copy)]
pub enum FullRes {
    #[default]
    _10bit_res = 0b0,
    full_res = 0b1
}




#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn default_reg_setup(){
        assert_eq!(BW_RATE::default().into_bytes()[0], 0b00001010);
        assert_eq!(POWER_CTL::default().into_bytes()[0], 0b00000000);
        assert_eq!(DATA_FORMAT::default().into_bytes()[0], 0b00000000);
    }
    
    #[test]
    fn bw_rate_configs(){
        assert_eq!(
            BW_RATE::default().with_low_power(0x1).with_odr(OutputDataRate::Hz12_5).into_bytes()[0],
            (0b1 << 4) | 0b0111
        );
    }

    #[test]
    fn pwr_ctl_config(){
        assert_eq!(
            POWER_CTL::default().with_measure(0b1).into_bytes()[0],
            0b1 << 3
        );
    }

    #[test]
    fn data_format_configs(){
        assert_eq!(
            DATA_FORMAT::default()
            .with_full_res(FullRes::full_res)
            .with_justisfy(Alignment::left)
            .with_range(AccelRange::_8g).into_bytes()[0],
            (0b1 << 3) | (0b1 << 2) | 0b10
        );
    }

    
}




