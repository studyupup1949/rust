#![allow(unused, non_camel_case_types)]

use modular_bitfield::{bitfield, prelude::B8};


// register addresses (name in data sheet = address (hexadecimal))
pub const DEVID_ADDR: u8 	  = 0x00;
pub const THRESH_TAP_ADDR: u8     = 0x1D;
pub const OFSX_ADDR: u8           = 0x1E;
pub const OFSY_ADDR: u8           = 0x1F;
pub const OFSZ_ADDR: u8           = 0x20;
pub const DUR_ADDR: u8            = 0x21;
pub const LATENT_ADDR: u8         = 0x22;
pub const WINDOW_ADDR: u8         = 0x23;
pub const THRESH_ACT_ADDR: u8     = 0x24;
pub const THRESH_INACT_ADDR: u8   = 0x25;
pub const TIME_INACT_ADDR: u8     = 0x26;
pub const ACT_INACT_CTL_ADDR: u8  = 0x27;
pub const THRESH_FF_ADDR: u8      = 0x28;
pub const TIME_FF_ADDR: u8        = 0x29;
pub const TAP_AXES_ADDR: u8       = 0x2A;
pub const ACT_TAP_STATUS_ADDR: u8 = 0x2B;
pub const BW_RATE_ADDR: u8        = 0x2C;
pub const POWER_CTL_ADDR: u8      = 0x2D;
pub const INT_ENABLE_ADDR: u8     = 0x2E;
pub const INT_MAP_ADDR: u8        = 0x2F;
pub const INT_SOURCE_ADDR: u8     = 0x30;
pub const DATA_FORMAT_ADDR: u8    = 0x31;
pub const DATAX0_ADDR: u8         = 0x32;
pub const DATAX1_ADDR: u8         = 0x33;
pub const DATAY0_ADDR: u8         = 0x34;
pub const DATAY1_ADDR: u8         = 0x35;
pub const DATAZ0_ADDR: u8         = 0x36;
pub const DATAZ1_ADDR: u8         = 0x37;
pub const FIFO_CTL_ADDR: u8       = 0x38;
pub const FIFO_STATUS_ADDR: u8    = 0x39;

pub const REGISTER_SIZE: u8 = 8;
pub const ADXL343_ADDR: u8 = 0x53; //i2c slave device address when using a qwiic connector
pub const DEVID_REG_VALUE: u8 = 0xE5;
//registers for data rate, power saving modes, justification
pub mod accel_configs; 




