use bitfield_struct::bitfield;
use bon::Builder;

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

pub trait Acs37800EepromExt: Acs37800 {
    #[cfg(feature = "async")]
    fn read_eeprom_0b_raw(
        &mut self,
    ) -> impl Future<Output = Result<Eeprom0bRaw, Acs37800ReadError>> + '_ {
        async {
            let r0b = Eeprom0bRaw(self.read_reg32(Acs37800EepromRegister::R0B).await?);
            Ok(r0b)
        }
    }

    #[cfg(not(feature = "async"))]
    fn read_eeprom_0b_raw(&mut self) -> Result<Eeprom0bRaw, Acs37800ReadError> {
        let r0b = Eeprom0bRaw(self.read_reg32(Acs37800EepromRegister::R0B)?);
        Ok(r0b)
    }

    #[cfg(feature = "async")]
    fn read_eeprom_0c_raw(
        &mut self,
    ) -> impl Future<Output = Result<Eeprom0cRaw, Acs37800ReadError>> + '_ {
        async {
            let r0c = Eeprom0cRaw(self.read_reg32(Acs37800EepromRegister::R0C).await?);
            Ok(r0c)
        }
    }

    #[cfg(not(feature = "async"))]
    fn read_eeprom_0c_raw(&mut self) -> Result<Eeprom0cRaw, Acs37800ReadError> {
        let r0c = Eeprom0cRaw(self.read_reg32(Acs37800EepromRegister::R0C)?);
        Ok(r0c)
    }

    #[cfg(feature = "async")]
    fn read_eeprom_0d_raw(
        &mut self,
    ) -> impl Future<Output = Result<Eeprom0dRaw, Acs37800ReadError>> + '_ {
        async {
            let r0d = Eeprom0dRaw(self.read_reg32(Acs37800EepromRegister::R0D).await?);
            Ok(r0d)
        }
    }

    #[cfg(not(feature = "async"))]
    fn read_eeprom_0d_raw(&mut self) -> Result<Eeprom0dRaw, Acs37800ReadError> {
        let r0d = Eeprom0dRaw(self.read_reg32(Acs37800EepromRegister::R0D)?);
        Ok(r0d)
    }

    #[cfg(feature = "async")]
    fn read_eeprom_0e_raw(
        &mut self,
    ) -> impl Future<Output = Result<Eeprom0eRaw, Acs37800ReadError>> + '_ {
        async {
            let r0e = Eeprom0eRaw(self.read_reg32(Acs37800EepromRegister::R0E).await?);
            Ok(r0e)
        }
    }

    #[cfg(not(feature = "async"))]
    fn read_eeprom_0e_raw(&mut self) -> Result<Eeprom0eRaw, Acs37800ReadError> {
        let r0e = Eeprom0eRaw(self.read_reg32(Acs37800EepromRegister::R0E)?);
        Ok(r0e)
    }

    #[cfg(feature = "async")]
    fn read_eeprom_0f_raw(
        &mut self,
    ) -> impl Future<Output = Result<Eeprom0fRaw, Acs37800ReadError>> + '_ {
        async {
            let r0f = Eeprom0fRaw(self.read_reg32(Acs37800EepromRegister::R0F).await?);
            Ok(r0f)
        }
    }

    #[cfg(not(feature = "async"))]
    fn read_eeprom_0f_raw(&mut self) -> Result<Eeprom0fRaw, Acs37800ReadError> {
        let r0f = Eeprom0fRaw(self.read_reg32(Acs37800EepromRegister::R0F)?);
        Ok(r0f)
    }

    #[cfg(feature = "async")]
    fn read_eeprom_raw(
        &mut self,
    ) -> impl Future<Output = Result<Acs37800EepromRaw, Acs37800ReadError>> + '_ {
        async {
            let r0b = self.read_eeprom_0b_raw().await?;
            let r0c = self.read_eeprom_0c_raw().await?;
            let r0d = self.read_eeprom_0d_raw().await?;
            let r0e = self.read_eeprom_0e_raw().await?;
            let r0f = self.read_eeprom_0f_raw().await?;
            Ok(Acs37800EepromRaw::builder()
                .r0b(r0b)
                .r0c(r0c)
                .r0d(r0d)
                .r0e(r0e)
                .r0f(r0f)
                .build())
        }
    }

    #[cfg(not(feature = "async"))]
    fn read_eeprom_raw(&mut self) -> Result<Acs37800EepromRaw, Acs37800ReadError> {
        let r0b = self.read_eeprom_0b_raw()?;
        let r0c = self.read_eeprom_0c_raw()?;
        let r0d = self.read_eeprom_0d_raw()?;
        let r0e = self.read_eeprom_0e_raw()?;
        let r0f = self.read_eeprom_0f_raw()?;
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
    fn read_eeprom(
        &mut self,
    ) -> impl Future<Output = Result<Acs37800Eeprom, Acs37800ReadError>> + '_ {
        async { Ok(self.read_eeprom_raw().await?.into()) }
    }

    #[cfg(not(feature = "async"))]
    fn read_eeprom(&mut self) -> Result<Acs37800Eeprom, Acs37800ReadError> {
        Ok(self.read_eeprom_raw()?.into())
    }
}

impl<T: Acs37800 + ?Sized> Acs37800EepromExt for T {}

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

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Acs37800EepromRegister {
    R0B = 0x0b,
    R0C = 0x0c,
    R0D = 0x0d,
    R0E = 0x0e,
    R0F = 0x0f,
}

#[cfg(test)]
mod test_support {
    use std::collections::HashMap;

    use super::*;

    #[derive(Default)]
    pub(super) struct MockDevice {
        regs: HashMap<Acs37800EepromRegister, u32>,
        fail_on: Option<Acs37800EepromRegister>,
    }

    impl MockDevice {
        pub(super) fn set_reg(&mut self, reg: Acs37800EepromRegister, value: u32) {
            self.regs.insert(reg, value);
        }

        pub(super) fn with_failure(reg: Acs37800EepromRegister) -> Self {
            Self {
                regs: HashMap::new(),
                fail_on: Some(reg),
            }
        }

        fn read_word(&mut self, reg: Acs37800EepromRegister) -> Result<u32, Acs37800ReadError> {
            if self.fail_on == Some(reg) {
                return Err(bus_error());
            }
            self.regs.get(&reg).copied().ok_or_else(bus_error)
        }
    }

    #[cfg(not(feature = "async"))]
    impl Acs37800 for MockDevice {
        fn read_reg32(&mut self, reg: Acs37800EepromRegister) -> Result<u32, Acs37800ReadError> {
            self.read_word(reg)
        }
    }

    #[cfg(feature = "async")]
    impl Acs37800 for MockDevice {
        fn read_reg32(
            &mut self,
            reg: Acs37800EepromRegister,
        ) -> impl Future<Output = Result<u32, Acs37800ReadError>> {
            let result = self.read_word(reg);
            async move { result }
        }
    }

    pub(super) fn pack_r0b(
        qvo_fine: u16,
        sns_fine: u16,
        crs_sns: u8,
        iavg: bool,
        pavg: bool,
    ) -> u32 {
        let mut value = 0u32;
        value |= (qvo_fine as u32) & 0x1ff;
        value |= ((sns_fine as u32) & 0x3ff) << 9;
        value |= ((crs_sns as u32) & 0x7) << 19;
        value |= bit(iavg) << 22;
        value |= bit(pavg) << 23;
        value
    }

    pub(super) fn pack_r0c(rms_avg_1: u8, rms_avg_2: u16, vchan_offset: u8) -> u32 {
        let mut value = 0u32;
        value |= (rms_avg_1 as u32) & 0x7f;
        value |= ((rms_avg_2 as u32) & 0x3ff) << 7;
        value |= ((vchan_offset as u32) & 0xff) << 17;
        value
    }

    pub(super) fn pack_r0d(ichan_en: bool, chan_sel: u8, fault: u8, fltdly: u8) -> u32 {
        let mut value = 0u32;
        value |= bit(ichan_en) << 7;
        value |= ((chan_sel as u32) & 0x7) << 9;
        value |= ((fault as u32) & 0xff) << 13;
        value |= ((fltdly as u32) & 0x7) << 21;
        value
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn pack_r0e(
        vevent: u8,
        overv: u8,
        underv: u8,
        delaycnt_sel: bool,
        halfcycle: bool,
        squarewave: bool,
        zerocross_channel: bool,
        zerocross_edge: bool,
    ) -> u32 {
        let mut value = 0u32;
        value |= (vevent as u32) & 0x3f;
        value |= ((overv as u32) & 0x3f) << 8;
        value |= ((underv as u32) & 0x3f) << 14;
        value |= bit(delaycnt_sel) << 20;
        value |= bit(halfcycle) << 21;
        value |= bit(squarewave) << 22;
        value |= bit(zerocross_channel) << 23;
        value |= bit(zerocross_edge) << 24;
        value
    }

    pub(super) fn pack_r0f(
        i2c_addr: u8,
        disable_addr: bool,
        dio0: u8,
        dio1: u8,
        n_cycles: u16,
        bypass: bool,
    ) -> u32 {
        let mut value = 0u32;
        value |= ((i2c_addr as u32) & 0x7f) << 2;
        value |= bit(disable_addr) << 9;
        value |= ((dio0 as u32) & 0x3) << 10;
        value |= ((dio1 as u32) & 0x3) << 12;
        value |= ((n_cycles as u32) & 0x3ff) << 14;
        value |= bit(bypass) << 24;
        value
    }

    pub(super) fn bus_error() -> Acs37800ReadError {
        #[cfg(feature = "std")]
        {
            Acs37800ReadError::Io("mock".into())
        }
        #[cfg(not(feature = "std"))]
        {
            Acs37800ReadError::Io
        }
    }

    fn bit(flag: bool) -> u32 {
        if flag { 1 } else { 0 }
    }
}

#[cfg(all(test, not(feature = "async")))]
mod tests {
    use crate::{Acs37800EepromExt, test::assert_is_bus_error};

    use super::test_support::*;
    use super::*;

    #[test]
    fn read_eeprom_raw_gathers_all_registers() {
        let mut mock = MockDevice::default();
        mock.set_reg(
            Acs37800EepromRegister::R0B,
            pack_r0b(0x101, 0x155, 0b011, true, false),
        );
        mock.set_reg(Acs37800EepromRegister::R0C, pack_r0c(0x45, 0x155, 0x3A));
        mock.set_reg(
            Acs37800EepromRegister::R0D,
            pack_r0d(true, 0b010, 0x5A, 0b111),
        );
        mock.set_reg(
            Acs37800EepromRegister::R0E,
            pack_r0e(0x12, 0x21, 0x11, false, true, false, true, false),
        );
        mock.set_reg(
            Acs37800EepromRegister::R0F,
            pack_r0f(0x63, false, 0b01, 0b10, 0x12C, true),
        );

        let raw = mock.read_eeprom_raw().expect("raw eeprom");
        assert_eq!(raw.r0b.qvo_fine(), 0x101);
        assert_eq!(raw.r0b.sns_fine(), 0x155);
        assert_eq!(raw.r0c.rms_avg_1(), 0x45);
        assert_eq!(raw.r0d.fault(), 0x5A);
        assert_eq!(raw.r0e.vevent_cycs(), 0x12);
        assert_eq!(raw.r0f.n(), 0x12C);
    }

    #[test]
    fn read_eeprom_interprets_signed_and_flags() {
        let mut mock = MockDevice::default();
        mock.set_reg(
            Acs37800EepromRegister::R0B,
            pack_r0b(0x1A5, 0x2D3, 0b010, true, true),
        );
        mock.set_reg(Acs37800EepromRegister::R0C, pack_r0c(0x40, 0x155, 0xF6));
        mock.set_reg(
            Acs37800EepromRegister::R0D,
            pack_r0d(true, 0b101, 0xAA, 0b110),
        );
        mock.set_reg(
            Acs37800EepromRegister::R0E,
            pack_r0e(0x17, 0x2A, 0x18, true, true, false, true, false),
        );
        mock.set_reg(
            Acs37800EepromRegister::R0F,
            pack_r0f(0x52, true, 0b10, 0b01, 0x1F3, true),
        );

        let eeprom = mock.read_eeprom().expect("parsed eeprom");
        assert_eq!(eeprom.qvo_fine_codes, -91);
        assert_eq!(eeprom.qvo_fine_icodes_offset, -5824);
        assert_eq!(eeprom.sns_fine_codes, -301);
        assert_eq!(eeprom.vchan_offset_codes, -10);
        assert_eq!(eeprom.crs_sns, 0b010);
        assert!(eeprom.iavgsel_enabled);
        assert!(eeprom.pavgsel_enabled);
        assert_eq!(eeprom.rms_avg_1, 0x40);
        assert_eq!(eeprom.rms_avg_2, 0x155);
        assert!(eeprom.ichan_delay_enabled);
        assert_eq!(eeprom.chan_delay_sel, 0b101);
        assert_eq!(eeprom.fault_threshold_codes, 0xAA);
        assert_eq!(eeprom.fault_delay_setting, 0b110);
        assert_eq!(eeprom.vevent_cycles, 0x17);
        assert_eq!(eeprom.overvoltage_threshold_codes, 0x2A);
        assert_eq!(eeprom.undervoltage_threshold_codes, 0x18);
        assert_eq!(eeprom.zerocross_pulse_width_us, 256);
        assert!(eeprom.halfcycle_en);
        assert!(!eeprom.squarewave_en);
        assert!(eeprom.zerocross_current_channel);
        assert!(!eeprom.zerocross_rising_edge);
        assert_eq!(eeprom.i2c_address_7bit, 0x52);
        assert!(eeprom.i2c_address_disabled);
        assert_eq!(eeprom.dio0_sel_raw, 0b10);
        assert_eq!(eeprom.dio1_sel_raw, 0b01);
        assert_eq!(eeprom.n_cycles, 0x1F3);
        assert!(eeprom.bypass_n_en);
    }

    #[test]
    fn read_eeprom_propagates_errors() {
        let mut mock = MockDevice::with_failure(Acs37800EepromRegister::R0C);
        mock.set_reg(Acs37800EepromRegister::R0B, pack_r0b(0, 0, 0, false, false));
        let err = mock.read_eeprom_raw().unwrap_err();
        assert_is_bus_error(&err);
    }
}

#[cfg(all(test, feature = "async"))]
mod async_tests {
    use crate::{Acs37800EepromExt, test::assert_is_bus_error};

    use super::test_support::*;
    use super::*;

    #[tokio::test]
    async fn read_eeprom_raw_gathers_all_registers_async() {
        let mut mock = MockDevice::default();
        mock.set_reg(
            Acs37800EepromRegister::R0B,
            pack_r0b(0x101, 0x155, 0b011, true, false),
        );
        mock.set_reg(Acs37800EepromRegister::R0C, pack_r0c(0x45, 0x155, 0x3A));
        mock.set_reg(
            Acs37800EepromRegister::R0D,
            pack_r0d(true, 0b010, 0x5A, 0b111),
        );
        mock.set_reg(
            Acs37800EepromRegister::R0E,
            pack_r0e(0x12, 0x21, 0x11, false, true, false, true, false),
        );
        mock.set_reg(
            Acs37800EepromRegister::R0F,
            pack_r0f(0x63, false, 0b01, 0b10, 0x12C, true),
        );

        let raw = mock.read_eeprom_raw().await.expect("raw eeprom");
        assert_eq!(raw.r0b.qvo_fine(), 0x101);
        assert_eq!(raw.r0b.sns_fine(), 0x155);
        assert_eq!(raw.r0c.rms_avg_1(), 0x45);
        assert_eq!(raw.r0d.fault(), 0x5A);
        assert_eq!(raw.r0e.vevent_cycs(), 0x12);
        assert_eq!(raw.r0f.n(), 0x12C);
    }

    #[tokio::test]
    async fn read_eeprom_interprets_signed_and_flags_async() {
        let mut mock = MockDevice::default();
        mock.set_reg(
            Acs37800EepromRegister::R0B,
            pack_r0b(0x1A5, 0x2D3, 0b010, true, true),
        );
        mock.set_reg(Acs37800EepromRegister::R0C, pack_r0c(0x40, 0x155, 0xF6));
        mock.set_reg(
            Acs37800EepromRegister::R0D,
            pack_r0d(true, 0b101, 0xAA, 0b110),
        );
        mock.set_reg(
            Acs37800EepromRegister::R0E,
            pack_r0e(0x17, 0x2A, 0x18, true, true, false, true, false),
        );
        mock.set_reg(
            Acs37800EepromRegister::R0F,
            pack_r0f(0x52, true, 0b10, 0b01, 0x1F3, true),
        );

        let eeprom = mock.read_eeprom().await.expect("parsed eeprom");
        assert_eq!(eeprom.qvo_fine_codes, -91);
        assert_eq!(eeprom.qvo_fine_icodes_offset, -5824);
        assert_eq!(eeprom.sns_fine_codes, -301);
        assert_eq!(eeprom.vchan_offset_codes, -10);
        assert_eq!(eeprom.crs_sns, 0b010);
        assert!(eeprom.iavgsel_enabled);
        assert!(eeprom.pavgsel_enabled);
        assert_eq!(eeprom.rms_avg_1, 0x40);
        assert_eq!(eeprom.rms_avg_2, 0x155);
        assert!(eeprom.ichan_delay_enabled);
        assert_eq!(eeprom.chan_delay_sel, 0b101);
        assert_eq!(eeprom.fault_threshold_codes, 0xAA);
        assert_eq!(eeprom.fault_delay_setting, 0b110);
        assert_eq!(eeprom.vevent_cycles, 0x17);
        assert_eq!(eeprom.overvoltage_threshold_codes, 0x2A);
        assert_eq!(eeprom.undervoltage_threshold_codes, 0x18);
        assert_eq!(eeprom.zerocross_pulse_width_us, 256);
        assert!(eeprom.halfcycle_en);
        assert!(!eeprom.squarewave_en);
        assert!(eeprom.zerocross_current_channel);
        assert!(!eeprom.zerocross_rising_edge);
        assert_eq!(eeprom.i2c_address_7bit, 0x52);
        assert!(eeprom.i2c_address_disabled);
        assert_eq!(eeprom.dio0_sel_raw, 0b10);
        assert_eq!(eeprom.dio1_sel_raw, 0b01);
        assert_eq!(eeprom.n_cycles, 0x1F3);
        assert!(eeprom.bypass_n_en);
    }

    #[tokio::test]
    async fn read_eeprom_propagates_errors_async() {
        let mut mock = MockDevice::with_failure(Acs37800EepromRegister::R0C);
        mock.set_reg(Acs37800EepromRegister::R0B, pack_r0b(0, 0, 0, false, false));
        let err = mock.read_eeprom_raw().await.unwrap_err();
        assert_is_bus_error(&err);
    }
}

#[cfg(test)]
mod sign_extend_tests {
    use super::sign_extend;

    #[test]
    fn sign_extend_handles_positive_values() {
        assert_eq!(sign_extend(0b0011, 4), 3);
    }

    #[test]
    fn sign_extend_handles_negative_values() {
        assert_eq!(sign_extend(0b1110, 4), -2);
    }
}
