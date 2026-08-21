#![allow(unused, non_snake_case)]
use derive_setters::Setters;

use crate::registers::{
    BW_RATE_ADDR,
    accel_configs::*
};


#[derive(Default, Clone, Copy)]
pub struct ADXL343Settings{
    odr: OutputDataRate ,
    range: AccelRange,
    justification: Alignment,
    resolution: FullRes,
    low_power_mode: bool,
    measurement_mode: bool //value of zero indicates off
}

impl ADXL343Settings {

    ///returns the configured state of the DATA_FORMAT reg IN STRUCT
    pub fn DATA_FORMAT_reg_value(&self) -> u8 {
        DATA_FORMAT::new()
            .with_range(self.range)
            .with_justisfy(self.justification)
            .with_full_res(self.resolution).into_bytes()[0]
    }

    ///returns the configured state of the BW_RATE reg IN STRUCT 
    pub fn BW_RATE_reg_value(&self) -> u8{
        BW_RATE::new()
        .with_low_power(self.low_power_mode as u8)
        .with_odr(self.odr).into_bytes()[0]
    }
    
    ///returns the number of bits used to represent axis reading
    pub fn resolution_to_bits(&self) -> u8 {
        match (self.resolution, self.range) {
            (FullRes::_10bit_res, _) => 10,
            (FullRes::full_res, AccelRange::_2g) => 10,
            (FullRes::full_res, AccelRange::_4g) => 11,
            (FullRes::full_res, AccelRange::_8g) => 12, 
            (FullRes::full_res, AccelRange::_16g) => 13
        }
    }

    /// conversion from g's to lsb's (least standard bits)
    pub fn g_per_lsb(&self) -> f32 {
        match (self.resolution, self.range) {
            (FullRes::full_res, _) => 1.0/256.0,
            (FullRes::_10bit_res, AccelRange::_2g) => 1.0/256.0,
            (FullRes::_10bit_res, AccelRange::_4g) => 1.0/128.0,
            (FullRes::_10bit_res, AccelRange::_8g) => 1.0/64.0, 
            (FullRes::_10bit_res, AccelRange::_16g) => 1.0/32.0
        }
    }

    pub fn lsb_per_g(&self) -> u16 {
        match (self.resolution, self.range) {
            (FullRes::full_res, _) => 256, 
            (FullRes::_10bit_res, AccelRange::_2g) => 256,
            (FullRes::_10bit_res, AccelRange::_4g) => 128,
            (FullRes::_10bit_res, AccelRange::_8g) => 64, 
            (FullRes::_10bit_res, AccelRange::_16g) => 32
        }
    }
	
    pub fn set_justification(&mut self, justification: Alignment)
    {
	self.justification = justification; 
    }
    pub fn get_justification(&self) -> Alignment{
        self.justification
    }
   
    pub fn set_odr(&mut self, odr: OutputDataRate)
    {
	self.odr = odr; 
    }
    
    pub fn get_odr(&self) -> OutputDataRate{
        self.odr
    }

    pub fn in_measurement_mode(&self) -> bool {
        self.measurement_mode
    }

    pub fn toggle_measurement_mode(&mut self){
        self.measurement_mode ^= true;
    }
    
    pub fn set_range(&mut self, range: AccelRange){
	self.range = range;
    }

}







