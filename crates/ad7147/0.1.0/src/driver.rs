use byte_slice_cast::AsByteSlice;
use embedded_hal::blocking::i2c::{Write, WriteRead};
use embedded_hal_1::delay::DelayUs;

use crate::device::{
    AmbientCompensationControl, CalibrationEnable, DeviceConfiguration, InternalDeviceConfiguration,
};

const STAGE_CONVERSION_RESULT_REGISTER: u16 = 0x0B;
const STAGE_COMPLETED_INTERRUPT_RESULT_REGISTER: u16 = 0x08;
const DEVICE_ID_REGISTER: u16 = 0x17;

pub struct Uninit;
pub struct Initialized;

pub struct Ad7147<I2C, MODE, const STAGES: usize> {
    i2c: I2C,
    address: u8,
    config: InternalDeviceConfiguration<STAGES>,
    _mode: MODE,
}

impl<I2C, E> Ad7147<I2C, Uninit, 0>
where
    I2C: WriteRead<Error = E> + Write<Error = E>,
{
    pub fn new(i2c: I2C, address: u8) -> Self {
        Self {
            i2c,
            address,
            config: InternalDeviceConfiguration::default(),
            _mode: Uninit,
        }
    }

    pub fn release(self) -> I2C {
        self.i2c
    }

    pub fn init<DELAY: DelayUs, const S: usize>(
        mut self,
        configuration: DeviceConfiguration<S>,
        delay: &mut DELAY,
    ) -> Result<Ad7147<I2C, Initialized, S>, E> {
        let mut data = [0u16; (12 * 8) + 1];
        data[0] = 0x080;
        for (i, stage) in configuration.0.stages.iter().enumerate() {
            let data_idx = (i * 8) + 1;
            data[data_idx..data_idx + 8].copy_from_slice(&stage.to_reg_value());
        }
        //println!("writing to device: addr: {:016b} stage0: {:016b}, stage0: {:016b}", data[0], data[1], data[2]);
        self.i2c
            .write(self.address, to_be_array(data).as_byte_slice())?;

        let mut data = [0u16; 9];
        data[0] = 0x000;
        data[1..9].copy_from_slice(&configuration.0.to_reg_value_for_init());
        //println!("writing to device: addr: {:016b} pwr_cfg: {:016b}, stage_cal: {:016b}, amb_comp_1: {:016b}, amb_comp_2: {:016b}, amb_comp_3: {:016b}, stage_low_int: {:016b}, stage_high_int: {:016b}, stage_complete_int: {:016b}", data[0], data[1], data[2], data[3], data[4], data[5],  data[6], data[7], data[8]);
        self.i2c
            .write(self.address, to_be_array(data).as_byte_slice())?;

        delay.delay_ms(100);

        let mut data = [0u16; 2];
        data[0] = CalibrationEnable::REGISTER;
        data[1] = configuration.0.calibration_enable.to_reg_value();
        //println!("writing to device: addr: {:016b} stage_cal: {:016b}", data[0], data[1]);
        self.i2c
            .write(self.address, to_be_array(data).as_byte_slice())?;

        Ok(Ad7147 {
            i2c: self.i2c,
            address: self.address,
            config: configuration.0,
            _mode: Initialized,
        })
    }
}

impl<I2C, const S: usize, E> Ad7147<I2C, Initialized, S>
where
    I2C: WriteRead<Error = E> + Write<Error = E>,
{
    pub fn read_all_stages(&mut self) -> Result<[u16; S], E> {
        let mut result = [0u8; 12 * 2];
        self.i2c.write_read(
            self.address,
            to_be_array([STAGE_CONVERSION_RESULT_REGISTER]).as_byte_slice(),
            &mut result,
        )?;
        //println!("read from device {:x?}", result);
        Ok(to_le_u16_array(&result))
    }

    pub fn read_interrupt_registers(&mut self) -> Result<[u16; 3], E> {
        let mut data = [0u8; 6];
        self.i2c.write_read(
            self.address,
            to_be_array([STAGE_COMPLETED_INTERRUPT_RESULT_REGISTER]).as_byte_slice(),
            &mut data,
        )?;
        //println!("read from device {:x?}", data);
        Ok(to_le_u16_array(&data))
    }

    pub fn read_device_id(&mut self) -> Result<u16, E> {
        let mut data = [0u8; 2];
        self.i2c.write_read(
            self.address,
            to_be_array([DEVICE_ID_REGISTER]).as_byte_slice(),
            &mut data,
        )?;
        Ok(u16::from_be_bytes(data))
    }

    pub fn reset(&mut self) -> Result<(), E> {
        let mut acc_reg = self.config.ambient_comp_control;
        acc_reg.conversion_reset = true;
        let mut data = [0u16; 4];
        data[0] = AmbientCompensationControl::REGISTER;
        data[1..4].copy_from_slice(&acc_reg.to_reg_value());
        self.i2c
            .write(self.address, to_be_array(data).as_byte_slice())
    }
}

fn to_be_array<const N: usize>(data: [u16; N]) -> [u16; N] {
    let mut result = [0u16; N];
    for (i, word) in data.into_iter().enumerate() {
        result[i] = word.to_be();
    }
    result
}

fn to_le_u16_array<const N: usize>(data: &[u8]) -> [u16; N] {
    let mut result = [0u16; N];
    for i in 0..N {
        let (int_bytes, _) = data[i * 2..].split_at(core::mem::size_of::<u16>());
        result[i] = u16::from_be_bytes(int_bytes.try_into().unwrap());
    }
    result
}
