#![allow(unused)]
use embedded_hal::i2c::{I2c, Error as I2c_Error};
use crate::registers::REGISTER_SIZE;
use crate::{
    registers::{
        self, DEVID_ADDR, BW_RATE_ADDR, DATA_FORMAT_ADDR, DATAX0_ADDR, ADXL343_ADDR, DEVID_REG_VALUE, POWER_CTL_ADDR,
        accel_configs::{self, Alignment, POWER_CTL} 
    },
    utils::settings::ADXL343Settings,
};
use core::fmt::Debug;

use core::{error::Error, fmt::{Display, Pointer}};

/// Device Interface
pub struct ADXL343Interface<I>
where
    I: I2c,
{
    i2c: I,
    settings: ADXL343Settings,
}


impl<I> ADXL343Interface<I>
where
    I: I2c,
{
    /// Returns uninitialized device object with default settings
    pub fn new(i2c: I) -> Self {
        Self {
            i2c,
            settings: Default::default()
        }
    }

    /// Returns uninitialized device object with provided settings
    pub fn with_settings(&mut self, settings: ADXL343Settings) -> Result<(), ADXL343Error<I::Error>>{

        //prevents entering into measurement mode before the configs are specified
        if settings.in_measurement_mode(){
            return Err(ADXL343Error::MeasurementModeBeforeConfig);
        }
        self.settings = settings;
        Ok(())
    }

    /// Initializes the DATA_FORMAT register and BW_RATE registers with the configs located in 
    /// the settings field (type ADXL343Settings). Will not place the device in measurement mode
    pub fn init(&mut self) -> Result<(), ADXL343Error<I::Error>> {
        self.write_to_register(BW_RATE_ADDR, self.settings.BW_RATE_reg_value())?;
        self.write_to_register(DATA_FORMAT_ADDR, self.settings.DATA_FORMAT_reg_value())?;
        Ok(())
    }

    /// Ensures that the device responding to the device address 0xE5 has DEVID 0xE5 
    pub fn confirm_device(&mut self) -> Result<(), ADXL343Error<I::Error>>{

	let returned_value = self.read_register(DEVID_ADDR)?;
		match returned_value{
		    DEVID_REG_VALUE => Ok(()),
		    _ => Err(ADXL343Error::DeviceIdMismatch)
		}
		

    }

    /// toggles measurement bit to 1 in the POWER_CTL register to begin measurements
    /// does nothing in the event that measurement mode is already enabled
    pub fn begin_measurements(&mut self) -> Result<(), ADXL343Error<I::Error>>{
        if (!self.settings.in_measurement_mode()){
            self.settings.toggle_measurement_mode();
            self.write_to_register(
                POWER_CTL_ADDR,
                POWER_CTL::default().with_measure(0x1).into_bytes()[0]
            )?;
        }
        Ok(())
    }

    /// opposite of begin_measurements method
    pub fn turn_off_measurements(&mut self) ->  Result<(), ADXL343Error<I::Error>>{
        if (self.settings.in_measurement_mode()){
            self.settings.toggle_measurement_mode();
            self.write_to_register(
                POWER_CTL_ADDR,
                POWER_CTL::default().with_measure(0x0).into_bytes()[0]
            )?;
        }
        Ok(())
    }


    /// Returns raw accelerometer readings in the format:
    /// [x_low, x_high, y_low, y_high, z_low, z_high] (called DATA_0 and DATA_1 in the datasheet)
   #[inline]
    pub fn read_full_sample(&mut self) -> Result<[u8; 6], ADXL343Error<I::Error>> {
        let mut read_buff = [0u8; 6];
        self.i2c.write_read(ADXL343_ADDR, &[DATAX0_ADDR], &mut read_buff)?;
        Ok(read_buff)
    }

    
    /// converts accel_data (represented as [lowbits, highbits]) into equivalent i16 representation
    #[inline]
    pub fn axis_value_raw(&mut self, accel_data: [u8; 2] ) -> i16{
        let mut axis_value = ((accel_data[1] as u16) << REGISTER_SIZE) | accel_data[0] as u16;
        let shift = 16 - self.settings.resolution_to_bits(); 
    
        //change right aligned reading into a left aligned reading
        if self.settings.get_justification() == Alignment::right {
            axis_value = axis_value << 6;
        };
        (axis_value as i16) >> shift
    }

    /// converts accel_data into its equivalent f32 representation
    #[inline]
    pub fn axis_value(&mut self, accel_data: i16) -> f32{
        (accel_data as f32) * self.settings.g_per_lsb()
    }

    /// accel reading [x_axis, y_axis, z_axis]
    pub fn read_accel(&mut self) -> Result<[f32; 3], ADXL343Error<I::Error>>{
        let binding = self.read_full_sample()?;
        let (axis_samples,_) = binding.as_chunks::<2>();
        let x_raw = self.axis_value_raw(axis_samples[0]);
        let y_raw = self.axis_value_raw(axis_samples[1]);
        let z_raw = self.axis_value_raw(axis_samples[2]);
        
        Ok(
            [
                self.axis_value(x_raw), 
                self.axis_value(y_raw), 
                self.axis_value(z_raw)
            ]
        )
    }

    #[inline]
    pub fn read_register(&mut self, reg_address: u8) -> Result<u8, ADXL343Error<I::Error>> {
        let mut read_buff = [0u8];
        self.i2c.write_read(ADXL343_ADDR, &[reg_address], &mut read_buff)?;
        Ok(read_buff[0])
    }

    fn write_to_register(&mut self, reg_address: u8, value: u8) -> Result<(), ADXL343Error<I::Error>> {
        self.i2c.write(ADXL343_ADDR, &mut [reg_address, value])?;
        Ok(())
    }
    
    /// returns both the i2c bus adapter and the settings struct
    pub fn destroy(mut self) -> (I, ADXL343Settings) {
        self.turn_off_measurements();
        (self.i2c, self.settings)
    }

}

//embedded_hal::i2c::I2c::Error has been labeled as I2c_Error

#[derive(Debug)]
pub enum ADXL343Error<E: I2c_Error>
{
    Interface(E),         // error from I2C/SPI interface
    DeviceIdMismatch,     
    MeasurementModeBeforeConfig
}

impl<E: I2c_Error+ Debug> Error for ADXL343Error<E>{}

impl<E: I2c_Error+ Debug> Display for ADXL343Error<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ADXL343Error::Interface(i2c_error) => i2c_error.fmt(f),
            ADXL343Error::DeviceIdMismatch => {
                f.write_str("Wrong device ID returned")
            },
            ADXL343Error::MeasurementModeBeforeConfig => {
                f.write_str("Attempted to turn on measurement mode prior to configuration")
            }
        }
    }
}

impl<E: I2c_Error> From<E> for ADXL343Error<E> {
    fn from(value: E) -> Self {
        ADXL343Error::Interface(value)
    }
}


