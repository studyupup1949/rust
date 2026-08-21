/// Default I2C address for ACS37800 device.
///
/// Value: 0x60 [96]
/// Reference: ACS37800 Datasheet, Page 31
///
/// When the device first poweres on, it measures the voltage level ont he two DIO pins.
/// It converts both voltage levels into a 4-bit ode for a total of sixteen peripheral addresses.
/// If both DIO pins are either unconnected or pulled to ground, the default I²C address is
/// `0x60` (96 in decimal).
///
/// ## See Also
/// - [`I2C_PROGRAMMED_ADDRESS`] - Default I²C address used when both DIO pins are pulled to Vcc.
pub const I2C_ADDRESS: u8 = 0x60;

/// Default I²C address programmed in the EEPROM at the factory.
///
/// Value: 0x7F [127]
/// Reference: ACS37800 Datasheet, Page 31
///
/// If both DIO pins are pulled to Vcc during power-on, the device uses the I²C address stored in
/// its EEPROM. The default factory-programmed address is `0x7F` (127 in decimal), but it can be
/// reprogrammed.
///
/// ## See Also
/// - [`I2C_ADDRESS`] -- Default I²C address when DIO pins are not connected or pulled to ground.
pub const I2C_PROGRAMMED_ADDRESS: u8 = 0x7F;
