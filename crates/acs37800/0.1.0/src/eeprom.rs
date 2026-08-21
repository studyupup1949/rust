use bitfield_struct::bitfield;
use bon::Builder;

#[cfg(feature = "async")]
use embedded_hal_async::i2c::I2c;

#[cfg(not(feature = "async"))]
use embedded_hal::i2c::I2c;

use crate::{Acs37800, Acs37800ReadError};

/// EEPROM register 0x0B (ACS37800_REGISTER_0B_t)
/// Bits (LSB0):
///   0..=8   : qvo_fine   (9 bits)
///   9..=18  : sns_fine   (10 bits)
///   19..=21 : crs_sns    (3 bits)
///   22      : iavgselen  (1 bit)
///   23      : pavgselen  (1 bit)
///   24..=25 : reserved   (2 bits)
///   26..=31 : ECC        (6 bits)
#[bitfield(u32, order = Lsb)]
pub struct Eeprom0bRaw {
    #[bits(9)]
    qvo_fine: u16,

    #[bits(10)]
    sns_fine: u16,

    #[bits(3)]
    crs_sns: u8,

    iavgselen: bool,

    pavgselen: bool,

    #[bits(2)]
    _reserved: u8,

    #[bits(6)]
    ecc: u8,
}

/// EEPROM register 0x0C (ACS37800_REGISTER_0C_t)
/// Bits:
///   0..=6   : rms_avg_1         (7 bits)
///   7..=16  : rms_avg_2         (10 bits)
///   17..=24 : vchan_offset_code (8 bits)
///   25      : reserved          (1 bit)
///   26..=31 : ECC               (6 bits)
#[bitfield(u32, order = Lsb)]
pub struct Eeprom0cRaw {
    #[bits(7)]
    rms_avg_1: u8,

    #[bits(10)]
    rms_avg_2: u16,

    #[bits(8)]
    vchan_offset_code: u8,

    _reserved: bool,

    #[bits(6)]
    ecc: u8,
}

/// EEPROM register 0x0D (ACS37800_REGISTER_0D_t)
/// Bits:
///   0..=6   : reserved1   (7 bits)
///   7       : ichan_del_en
///   8       : reserved2
///   9..=11  : chan_del_sel (3 bits)
///   12      : reserved3
///   13..=20 : fault        (8 bits)
///   21..=23 : fltdly       (3 bits)
///   24..=25 : reserved4    (2 bits)
///   26..=31 : ECC          (6 bits)
#[bitfield(u32, order = Lsb)]
pub struct Eeprom0dRaw {
    #[bits(7)]
    _reserved1: u8,

    ichan_del_en: bool,

    reserved2: bool,

    #[bits(3)]
    chan_del_sel: u8,

    _reserved3: bool,

    #[bits(8)]
    fault: u8,

    #[bits(3)]
    fltdly: u8,

    #[bits(2)]
    _reserved4: u8,

    #[bits(6)]
    ecc: u8,
}

/// EEPROM register 0x0E (ACS37800_REGISTER_0E_t)
/// Bits:
///   0..=5   : vevent_cycs       (6 bits)
///   6..=7   : reserved1         (2 bits)
///   8..=13  : overvreg          (6 bits)
///   14..=19 : undervreg         (6 bits)
///   20      : delaycnt_sel
///   21      : halfcycle_en
///   22      : squarewave_en
///   23      : zerocrosschansel
///   24      : zerocrossedgesel
///   25      : reserved2
///   26..=31 : ECC               (6 bits)
#[bitfield(u32, order = Lsb)]
pub struct Eeprom0eRaw {
    #[bits(6)]
    vevent_cycs: u8,

    #[bits(2)]
    _reserved1: u8,

    #[bits(6)]
    overvreg: u8,

    #[bits(6)]
    undervreg: u8,

    delaycnt_sel: bool,

    halfcycle_en: bool,

    squarewave_en: bool,

    zerocrosschansel: bool,

    zerocrossedgesel: bool,

    _reserved2: bool,

    #[bits(6)]
    ecc: u8,
}

/// EEPROM register 0x0F (ACS37800_REGISTER_0F_t)
/// Bits:
///   0..=1   : reserved1     (2 bits)
///   2..=8   : i2c_slv_addr  (7 bits)
///   9       : i2c_dis_slv_addr
///   10..=11 : dio_0_sel     (2 bits)
///   12..=13 : dio_1_sel     (2 bits)
///   14..=23 : n             (10 bits)
///   24      : bypass_n_en
///   25      : reserved2
///   26..=31 : ECC           (6 bits)
#[bitfield(u32, order = Lsb)]
pub struct Eeprom0fRaw {
    #[bits(2)]
    _reserved1: u8,

    #[bits(7)]
    i2c_slv_addr: u8,

    i2c_dis_slv_addr: bool,

    #[bits(2)]
    dio_0_sel: u8,

    #[bits(2)]
    dio_1_sel: u8,

    #[bits(10)]
    n: u16,

    bypass_n_en: bool,

    _reserved2: bool,

    #[bits(6)]
    ecc: u8,
}

#[derive(Builder, Debug)]
pub struct Acs37800EepromRaw {
    #[builder(into)]
    pub r0b: Eeprom0bRaw,
    #[builder(into)]
    pub r0c: Eeprom0cRaw,
    #[builder(into)]
    pub r0d: Eeprom0dRaw,
    #[builder(into)]
    pub r0e: Eeprom0eRaw,
    #[builder(into)]
    pub r0f: Eeprom0fRaw,
}

impl<I2C: I2c> Acs37800<I2C> {
    #[cfg(feature = "async")]
    pub async fn read_eeprom_0b(&mut self) -> Result<Eeprom0bRaw, Acs37800ReadError<I2C>> {
        let r0b = Eeprom0bRaw(self.read_reg32(0x0B).await?);
        Ok(r0b)
    }

    #[cfg(not(feature = "async"))]
    pub fn read_eeprom_0b(&mut self) -> Result<Eeprom0bRaw, Acs37800ReadError<I2C>> {
        let r0b = Eeprom0bRaw(self.read_reg32(0x0B)?);
        Ok(r0b)
    }

    #[cfg(feature = "async")]
    pub async fn read_eeprom_0c(&mut self) -> Result<Eeprom0cRaw, Acs37800ReadError<I2C>> {
        let r0c = Eeprom0cRaw(self.read_reg32(0x0C).await?);
        Ok(r0c)
    }

    #[cfg(not(feature = "async"))]
    pub fn read_eeprom_0c(&mut self) -> Result<Eeprom0cRaw, Acs37800ReadError<I2C>> {
        let r0c = Eeprom0cRaw(self.read_reg32(0x0C)?);
        Ok(r0c)
    }

    #[cfg(feature = "async")]
    pub async fn read_eeprom_0d(&mut self) -> Result<Eeprom0dRaw, Acs37800ReadError<I2C>> {
        let r0d = Eeprom0dRaw(self.read_reg32(0x0D).await?);
        Ok(r0d)
    }

    #[cfg(not(feature = "async"))]
    pub fn read_eeprom_0d(&mut self) -> Result<Eeprom0dRaw, Acs37800ReadError<I2C>> {
        let r0d = Eeprom0dRaw(self.read_reg32(0x0D)?);
        Ok(r0d)
    }

    #[cfg(feature = "async")]
    pub async fn read_eeprom_0e(&mut self) -> Result<Eeprom0eRaw, Acs37800ReadError<I2C>> {
        let r0e = Eeprom0eRaw(self.read_reg32(0x0E).await?);
        Ok(r0e)
    }

    #[cfg(not(feature = "async"))]
    pub fn read_eeprom_0e(&mut self) -> Result<Eeprom0eRaw, Acs37800ReadError<I2C>> {
        let r0e = Eeprom0eRaw(self.read_reg32(0x0E)?);
        Ok(r0e)
    }

    #[cfg(feature = "async")]
    pub async fn read_eeprom_0f(&mut self) -> Result<Eeprom0fRaw, Acs37800ReadError<I2C>> {
        let r0f = Eeprom0fRaw(self.read_reg32(0x0F).await?);
        Ok(r0f)
    }

    #[cfg(not(feature = "async"))]
    pub fn read_eeprom_0f(&mut self) -> Result<Eeprom0fRaw, Acs37800ReadError<I2C>> {
        let r0f = Eeprom0fRaw(self.read_reg32(0x0F)?);
        Ok(r0f)
    }

    #[cfg(feature = "async")]
    pub async fn read_eeprom_raw(&mut self) -> Result<Acs37800EepromRaw, Acs37800ReadError<I2C>> {
        let r0b = self.read_eeprom_0b().await?;
        let r0c = self.read_eeprom_0c().await?;
        let r0d = self.read_eeprom_0d().await?;
        let r0e = self.read_eeprom_0e().await?;
        let r0f = self.read_eeprom_0f().await?;
        Ok(Acs37800EepromRaw::builder()
            .r0b(r0b)
            .r0c(r0c)
            .r0d(r0d)
            .r0e(r0e)
            .r0f(r0f)
            .build())
    }

    #[cfg(not(feature = "async"))]
    pub fn read_eeprom_raw(&mut self) -> Result<Acs37800EepromRaw, Acs37800ReadError<I2C>> {
        let r0b = self.read_eeprom_0b()?;
        let r0c = self.read_eeprom_0c()?;
        let r0d = self.read_eeprom_0d()?;
        let r0e = self.read_eeprom_0e()?;
        let r0f = self.read_eeprom_0f()?;
        Ok(Acs37800EepromRaw::builder()
            .r0b(r0b)
            .r0c(r0c)
            .r0d(r0d)
            .r0e(r0e)
            .r0f(r0f)
            .build())
    }

    /// Convenience: read and interpret EEPROM in one go.
    #[cfg(feature = "async")]
    pub async fn read_eeprom(&mut self) -> Result<Acs37800Eeprom, Acs37800ReadError<I2C>> {
        Ok(self.read_eeprom_raw().await?.into())
    }

    #[cfg(not(feature = "async"))]
    pub fn read_eeprom(&mut self) -> Result<Acs37800Eeprom, Acs37800ReadError<I2C>> {
        Ok(self.read_eeprom_raw()?.into())
    }
}

/// Higher-level interpreted view of the EEPROM content.
///
/// This turns the raw bitfields into signed types and applies a few simple
/// scale factors straight from the datasheet. Anything that still depends on
/// board-level scaling (e.g. turning thresholds into volts / amps) is left
/// in “codes” and you can apply your own conversion.
#[derive(Debug, Clone, Copy)]
pub struct Acs37800Eeprom {
    // Current-channel trimming (register 0x0B)
    /// QVO_FINE in register codes, sign-extended (-256..=255).
    pub qvo_fine_codes: i16,
    /// QVO_FINE converted into ICODES LSB offset (step = 64 LSB).
    pub qvo_fine_icodes_offset: i32,
    /// SNS_FINE in register codes, sign-extended (-512..=511).
    pub sns_fine_codes: i16,
    /// Coarse sensitivity trim setting (datasheet-specific interpretation).
    pub crs_sns: u8,
    pub iavgsel_enabled: bool,
    pub pavgsel_enabled: bool,

    // RMS averaging & voltage offset (register 0x0C)
    /// Raw averaging config fields (you can turn these into cycles / time
    /// based on the datasheet’s equations and your line frequency).
    pub rms_avg_1: u8,
    pub rms_avg_2: u16,
    /// Signed voltage channel offset code.
    pub vchan_offset_codes: i8,

    // Overcurrent / fault (register 0x0D)
    pub ichan_delay_enabled: bool,
    pub chan_delay_sel: u8,
    /// Fault threshold code (maps into IRMS threshold per datasheet).
    pub fault_threshold_codes: u8,
    /// Fault delay setting (raw).
    pub fault_delay_setting: u8,

    // Voltage events, UV/OV thresholds, zero-crossing (register 0x0E)
    pub vevent_cycles: u8,
    pub overvoltage_threshold_codes: u8,
    pub undervoltage_threshold_codes: u8,
    /// Selected zero-cross pulse width in microseconds (32 or 256).
    pub zerocross_pulse_width_us: u32,
    pub halfcycle_en: bool,
    pub squarewave_en: bool,
    pub zerocross_current_channel: bool,
    pub zerocross_rising_edge: bool,

    // I2C, DIO, N (register 0x0F)
    /// 7-bit I2C address.
    pub i2c_address_7bit: u8,
    /// If true, the EEPROM address is disabled (device uses default address).
    pub i2c_address_disabled: bool,
    pub dio0_sel_raw: u8,
    pub dio1_sel_raw: u8,
    /// N cycles for certain measurements (see datasheet, and BYPASS_N_EN).
    pub n_cycles: u16,
    pub bypass_n_en: bool,
}

impl From<Acs37800EepromRaw> for Acs37800Eeprom {
    fn from(raw: Acs37800EepromRaw) -> Self {
        let Acs37800EepromRaw {
            r0b,
            r0c,
            r0d,
            r0e,
            r0f,
        } = raw;

        // Signed fields
        let qvo_fine_codes = sign_extend(r0b.qvo_fine(), 9);
        let sns_fine_codes = sign_extend(r0b.sns_fine(), 10);
        let vchan_offset_codes = sign_extend(r0c.vchan_offset_code() as u16, 8) as i8;

        // QVO_FINE step is 64 ICODES per datasheet
        let qvo_fine_icodes_offset = i32::from(qvo_fine_codes) * 64;

        // Zero-cross pulse width
        let zerocross_pulse_width_us = if r0e.delaycnt_sel() { 256 } else { 32 };

        Acs37800Eeprom {
            // 0x0B
            qvo_fine_codes,
            qvo_fine_icodes_offset,
            sns_fine_codes,
            crs_sns: r0b.crs_sns(),
            iavgsel_enabled: r0b.iavgselen(),
            pavgsel_enabled: r0b.pavgselen(),

            // 0x0C
            rms_avg_1: r0c.rms_avg_1(),
            rms_avg_2: r0c.rms_avg_2(),
            vchan_offset_codes,

            // 0x0D
            ichan_delay_enabled: r0d.ichan_del_en(),
            chan_delay_sel: r0d.chan_del_sel(),
            fault_threshold_codes: r0d.fault(),
            fault_delay_setting: r0d.fltdly(),

            // 0x0E
            vevent_cycles: r0e.vevent_cycs(),
            overvoltage_threshold_codes: r0e.overvreg(),
            undervoltage_threshold_codes: r0e.undervreg(),
            zerocross_pulse_width_us,
            halfcycle_en: r0e.halfcycle_en(),
            squarewave_en: r0e.squarewave_en(),
            zerocross_current_channel: r0e.zerocrosschansel(),
            zerocross_rising_edge: r0e.zerocrossedgesel(),

            // 0x0F
            i2c_address_7bit: r0f.i2c_slv_addr(),
            i2c_address_disabled: r0f.i2c_dis_slv_addr(),
            dio0_sel_raw: r0f.dio_0_sel(),
            dio1_sel_raw: r0f.dio_1_sel(),
            n_cycles: r0f.n(),
            bypass_n_en: r0f.bypass_n_en(),
        }
    }
}

/// Helper: sign-extend a N-bit unsigned value into i16.
fn sign_extend(val: u16, bits: u8) -> i16 {
    let mask = (1u16 << bits) - 1;
    let sign_bit = 1u16 << (bits - 1);
    let v = val & mask;
    if v & sign_bit != 0 {
        // fill upper bits with 1s
        (v as i16) | !((mask) as i16)
    } else {
        v as i16
    }
}
