#![no_std]
#![forbid(unsafe_code)]

use embedded_io::{Read, Write};

// Protocol constants
const STARTCODE: u16 = 0xEF01;
const COMMANDPACKET: u8 = 0x01;
const DATAPACKET: u8 = 0x02;
const ACKPACKET: u8 = 0x07;
const ENDDATAPACKET: u8 = 0x08;

// Command codes
const GETIMAGE: u8 = 0x01;
const IMAGE2TZ: u8 = 0x02;
const COMPARE: u8 = 0x03;
const FINGERPRINTSEARCH: u8 = 0x04;
const REGMODEL: u8 = 0x05;
const STORE: u8 = 0x06;
const LOAD: u8 = 0x07;
const UPLOAD: u8 = 0x08;
const DOWNLOAD: u8 = 0x09;
const UPLOADIMAGE: u8 = 0x0A;
const DOWNLOADIMAGE: u8 = 0x0B;
const DELETE: u8 = 0x0C;
const EMPTY: u8 = 0x0D;
const SETSYSPARA: u8 = 0x0E;
const READSYSPARA: u8 = 0x0F;
const VERIFYPASSWORD: u8 = 0x13;
const HISPEEDSEARCH: u8 = 0x1B;
const TEMPLATECOUNT: u8 = 0x1D;
const TEMPLATEREAD: u8 = 0x1F;
const SETAURA: u8 = 0x35;
const SOFTRESET: u8 = 0x3D;
const GETECHO: u8 = 0x53;

// Packet error codes
pub const OK: u8 = 0x00;
pub const PACKETRECIEVEERR: u8 = 0x01;
pub const NOFINGER: u8 = 0x02;
pub const IMAGEFAIL: u8 = 0x03;
pub const IMAGEMESS: u8 = 0x06;
pub const FEATUREFAIL: u8 = 0x07;
pub const NOMATCH: u8 = 0x08;
pub const NOTFOUND: u8 = 0x09;
pub const ENROLLMISMATCH: u8 = 0x0A;
pub const BADLOCATION: u8 = 0x0B;
pub const DBRANGEFAIL: u8 = 0x0C;
pub const UPLOADFEATUREFAIL: u8 = 0x0D;
pub const PACKETRESPONSEFAIL: u8 = 0x0E;
pub const UPLOADFAIL: u8 = 0x0F;
pub const DELETEFAIL: u8 = 0x10;
pub const DBCLEARFAIL: u8 = 0x11;
pub const PASSFAIL: u8 = 0x13;
pub const INVALIDIMAGE: u8 = 0x15;
pub const FLASHERR: u8 = 0x18;
pub const INVALIDREG: u8 = 0x1A;
pub const ADDRCODE: u8 = 0x20;
pub const PASSVERIFY: u8 = 0x21;
pub const MODULEOK: u8 = 0x55;

/// Maximum size for internal packet buffers
const MAX_PACKET_SIZE: usize = 64;

/// Error type for fingerprint sensor operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error<E> {
    /// UART communication error
    Uart(E),
    /// Invalid start code received
    InvalidStartCode,
    /// Address mismatch
    AddressMismatch,
    /// Invalid packet type received
    InvalidPacketType,
    /// Password verification failed
    PasswordFailed,
    /// Command execution failed with error code
    CommandFailed(u8),
    /// Buffer too small for operation
    BufferTooSmall,
    /// System parameters not initialized
    NotInitialized,
    /// Sensor handshake failed
    HandshakeFailed,
}

/// Data packet size configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataPacketSize {
    Bytes32 = 0,
    Bytes64 = 1,
    Bytes128 = 2,
    Bytes256 = 3,
}

impl DataPacketSize {
    /// Returns the actual byte count for this packet size setting
    pub const fn byte_count(self) -> usize {
        match self {
            DataPacketSize::Bytes32 => 32,
            DataPacketSize::Bytes64 => 64,
            DataPacketSize::Bytes128 => 128,
            DataPacketSize::Bytes256 => 256,
        }
    }

    fn from_raw(value: u16) -> Self {
        match value {
            0 => DataPacketSize::Bytes32,
            1 => DataPacketSize::Bytes64,
            2 => DataPacketSize::Bytes128,
            3 => DataPacketSize::Bytes256,
            _ => DataPacketSize::Bytes32,
        }
    }
}

/// LED color for R503 sensor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedColor {
    Red = 1,
    Blue = 2,
    Purple = 3,
}

/// LED mode for R503 sensor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedMode {
    Breathe = 1,
    Flash = 2,
    On = 3,
    Off = 4,
    FadeOn = 5,
    FadeOff = 6,
}

/// Fingerprint data buffer type
#[derive(Debug, Clone, Copy)]
pub enum SensorBuffer {
    Image,
    Char { slot: u8 },
}

impl SensorBuffer {
    /// Create a character buffer with validated slot (1 or 2)
    pub fn char(slot: u8) -> Self {
        let slot = if slot == 1 || slot == 2 { slot } else { 2 };
        SensorBuffer::Char { slot }
    }
}

/// System parameter identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemParam {
    BaudRate = 4,
    SecurityLevel = 5,
    DataPacketSize = 6,
}

/// System parameters read from the sensor
#[derive(Debug, Clone)]
pub struct SystemParameters {
    pub status_register: u16,
    pub system_id: u16,
    pub library_size: u16,
    pub security_level: u16,
    pub device_address: [u8; 4],
    pub data_packet_size: DataPacketSize,
    pub baudrate: u16,
}

/// Fingerprint sensor device
pub struct FingerprintSensor<UART>
where
    UART: Write + Read,
{
    uart: UART,
    address: [u8; 4],
    password: [u8; 4],

    sys_params: Option<SystemParameters>,

    // Search/match results
    pub finger_id: u16,
    pub confidence: u16,
    pub template_count: u16,
}

impl<UART, E> FingerprintSensor<UART>
where
    UART: Write<Error = E> + Read<Error = E>,
{
    /// Creates a new fingerprint sensor instance with the given UART and password.
    /// Uses default address (0xFF, 0xFF, 0xFF, 0xFF).
    pub fn new(uart: UART, password: [u8; 4]) -> Self {
        FingerprintSensor {
            uart,
            address: [0xFF; 4],
            password,
            sys_params: None,
            finger_id: 0,
            confidence: 0,
            template_count: 0,
        }
    }

    /// Creates a new fingerprint sensor instance with custom address and password.
    pub fn new_with_address(uart: UART, address: [u8; 4], password: [u8; 4]) -> Self {
        FingerprintSensor {
            uart,
            address,
            password,
            sys_params: None,
            finger_id: 0,
            confidence: 0,
            template_count: 0,
        }
    }

    /// Initializes the sensor by verifying password and reading system parameters.
    /// This should be called after creating the sensor instance.
    pub fn init(&mut self) -> Result<(), Error<E>> {
        let result = self.verify_password()?;
        if result != OK {
            return Err(Error::PasswordFailed);
        }

        let result = self.read_sysparam()?;
        if result != OK {
            return Err(Error::CommandFailed(result));
        }

        Ok(())
    }

    /// Returns the library size from cached system parameters.
    pub fn library_size(&self) -> Option<u16> {
        self.sys_params.as_ref().map(|p| p.library_size)
    }

    /// Returns the data packet size setting.
    pub fn data_packet_size(&self) -> Option<DataPacketSize> {
        self.sys_params.as_ref().map(|p| p.data_packet_size)
    }

    /// Returns a reference to the cached system parameters.
    pub fn system_parameters(&self) -> Option<&SystemParameters> {
        self.sys_params.as_ref()
    }

    /// Releases the UART, consuming the sensor.
    pub fn release(self) -> UART {
        self.uart
    }

    /// Checks the state of the fingerprint scanner module.
    /// Returns `Ok(true)` if module is OK.
    pub fn check_module(&mut self) -> Result<bool, Error<E>> {
        self.send_packet(&[GETECHO])?;
        let mut reply = [0u8; 16];
        let len = self.get_packet(&mut reply)?;
        if len == 0 || reply[0] != MODULEOK {
            return Err(Error::HandshakeFailed);
        }
        Ok(true)
    }

    /// Verifies the password with the sensor.
    /// Returns the packet error code (OK on success).
    pub fn verify_password(&mut self) -> Result<u8, Error<E>> {
        let packet = [
            VERIFYPASSWORD,
            self.password[0],
            self.password[1],
            self.password[2],
            self.password[3],
        ];

        self.send_packet(&packet)?;
        let mut reply = [0u8; 16];
        let len = self.get_packet(&mut reply)?;
        if len == 0 {
            return Ok(PACKETRECIEVEERR);
        }
        Ok(reply[0])
    }

    /// Requests the sensor to count the number of templates.
    /// Stores result in `self.template_count`.
    /// Returns the packet error code or OK on success.
    pub fn count_templates(&mut self) -> Result<u8, Error<E>> {
        self.send_packet(&[TEMPLATECOUNT])?;
        let mut reply = [0u8; 16];
        let len = self.get_packet(&mut reply)?;

        if len >= 3 {
            self.template_count = read_u16_be(&reply[1..3]);
        } else {
            self.template_count = 0;
        }

        if len == 0 {
            return Ok(PACKETRECIEVEERR);
        }
        Ok(reply[0])
    }

    /// Reads the system parameters from the sensor.
    /// Returns the packet error code or OK on success.
    pub fn read_sysparam(&mut self) -> Result<u8, Error<E>> {
        self.send_packet(&[READSYSPARA])?;
        let mut reply = [0u8; 32];
        let len = self.get_packet(&mut reply)?;

        if len == 0 || reply[0] != OK {
            return Ok(reply[0]);
        }

        let params = SystemParameters {
            status_register: read_u16_be(&reply[1..3]),
            system_id: read_u16_be(&reply[3..5]),
            library_size: read_u16_be(&reply[5..7]),
            security_level: read_u16_be(&reply[7..9]),
            device_address: [reply[9], reply[10], reply[11], reply[12]],
            data_packet_size: DataPacketSize::from_raw(read_u16_be(&reply[13..15])),
            baudrate: read_u16_be(&reply[15..17]),
        };

        self.sys_params = Some(params);
        Ok(OK)
    }

    /// Sets a system parameter.
    /// Returns the packet error code or OK on success.
    pub fn set_sysparam(&mut self, param: SystemParam, value: u8) -> Result<u8, Error<E>> {
        self.send_packet(&[SETSYSPARA, param as u8, value])?;
        let mut reply = [0u8; 16];
        let len = self.get_packet(&mut reply)?;

        let result = if len > 0 { reply[0] } else { PACKETRECIEVEERR };

        if result == OK {
            // Update cached parameters
            if let Some(ref mut params) = self.sys_params {
                match param {
                    SystemParam::BaudRate => params.baudrate = value as u16,
                    SystemParam::SecurityLevel => params.security_level = value as u16,
                    SystemParam::DataPacketSize => {
                        params.data_packet_size = DataPacketSize::from_raw(value as u16)
                    }
                }
            }
        }

        Ok(result)
    }

    /// Requests the sensor to take an image and store it in memory.
    /// Returns the packet error code or OK on success.
    pub fn get_image(&mut self) -> Result<u8, Error<E>> {
        self.send_packet(&[GETIMAGE])?;
        let mut reply = [0u8; 16];
        let len = self.get_packet(&mut reply)?;
        if len == 0 {
            return Ok(PACKETRECIEVEERR);
        }
        Ok(reply[0])
    }

    /// Requests the sensor to convert the image to a template.
    /// Slot should be 1 or 2.
    /// Returns the packet error code or OK on success.
    pub fn image_2_tz(&mut self, slot: u8) -> Result<u8, Error<E>> {
        self.send_packet(&[IMAGE2TZ, slot])?;
        let mut reply = [0u8; 16];
        let len = self.get_packet(&mut reply)?;
        if len == 0 {
            return Ok(PACKETRECIEVEERR);
        }
        Ok(reply[0])
    }

    /// Requests the sensor to create a model from the templates.
    /// Returns the packet error code or OK on success.
    pub fn create_model(&mut self) -> Result<u8, Error<E>> {
        self.send_packet(&[REGMODEL])?;
        let mut reply = [0u8; 16];
        let len = self.get_packet(&mut reply)?;
        if len == 0 {
            return Ok(PACKETRECIEVEERR);
        }
        Ok(reply[0])
    }

    /// Requests the sensor to store the model into flash memory.
    /// Returns the packet error code or OK on success.
    pub fn store_model(&mut self, location: u16, slot: u8) -> Result<u8, Error<E>> {
        self.send_packet(&[STORE, slot, (location >> 8) as u8, (location & 0xFF) as u8])?;
        let mut reply = [0u8; 16];
        let len = self.get_packet(&mut reply)?;
        if len == 0 {
            return Ok(PACKETRECIEVEERR);
        }
        Ok(reply[0])
    }

    /// Requests the sensor to delete a model from flash memory.
    /// Returns the packet error code or OK on success.
    pub fn delete_model(&mut self, location: u16) -> Result<u8, Error<E>> {
        self.send_packet(&[
            DELETE,
            (location >> 8) as u8,
            (location & 0xFF) as u8,
            0x00,
            0x01,
        ])?;
        let mut reply = [0u8; 16];
        let len = self.get_packet(&mut reply)?;
        if len == 0 {
            return Ok(PACKETRECIEVEERR);
        }
        Ok(reply[0])
    }

    /// Requests the sensor to load a model from flash memory to a buffer slot.
    /// Returns the packet error code or OK on success.
    pub fn load_model(&mut self, location: u16, slot: u8) -> Result<u8, Error<E>> {
        self.send_packet(&[LOAD, slot, (location >> 8) as u8, (location & 0xFF) as u8])?;
        let mut reply = [0u8; 16];
        let len = self.get_packet(&mut reply)?;
        if len == 0 {
            return Ok(PACKETRECIEVEERR);
        }
        Ok(reply[0])
    }

    /// Requests the sensor to transfer the fingerprint image or template.
    /// Data is written to the provided buffer.
    /// Returns the number of bytes written to the buffer.
    pub fn get_fpdata(&mut self, buffer: SensorBuffer, out: &mut [u8]) -> Result<usize, Error<E>> {
        match buffer {
            SensorBuffer::Image => {
                self.send_packet(&[UPLOADIMAGE])?;
            }
            SensorBuffer::Char { slot } => {
                let slot = if slot == 1 || slot == 2 { slot } else { 2 };
                self.send_packet(&[UPLOAD, slot])?;
            }
        }

        let mut reply = [0u8; 16];
        let len = self.get_packet(&mut reply)?;
        if len == 0 || reply[0] != OK {
            return Err(Error::CommandFailed(if len > 0 {
                reply[0]
            } else {
                PACKETRECIEVEERR
            }));
        }

        // Read data packets
        self.get_data(out)
    }

    /// Sends fingerprint data to the sensor.
    /// Returns `Ok(true)` on success.
    pub fn send_fpdata(&mut self, data: &[u8], buffer: SensorBuffer) -> Result<bool, Error<E>> {
        match buffer {
            SensorBuffer::Image => {
                self.send_packet(&[DOWNLOADIMAGE])?;
            }
            SensorBuffer::Char { slot } => {
                let slot = if slot == 1 || slot == 2 { slot } else { 2 };
                self.send_packet(&[DOWNLOAD, slot])?;
            }
        }

        let mut reply = [0u8; 16];
        let len = self.get_packet(&mut reply)?;
        if len == 0 || reply[0] != OK {
            return Err(Error::CommandFailed(if len > 0 {
                reply[0]
            } else {
                PACKETRECIEVEERR
            }));
        }

        self.send_data(data)?;
        Ok(true)
    }

    /// Requests the sensor to delete all models from flash memory.
    /// Returns the packet error code or OK on success.
    pub fn empty_library(&mut self) -> Result<u8, Error<E>> {
        self.send_packet(&[EMPTY])?;
        let mut reply = [0u8; 16];
        let len = self.get_packet(&mut reply)?;
        if len == 0 {
            return Ok(PACKETRECIEVEERR);
        }
        Ok(reply[0])
    }

    /// Reads template locations for a specific page.
    /// Each page represents 256 template slots.
    /// The bitmap buffer should be at least 32 bytes.
    /// Returns the packet error code or OK on success.
    pub fn read_template_page(&mut self, page: u8, bitmap: &mut [u8; 32]) -> Result<u8, Error<E>> {
        self.send_packet(&[TEMPLATEREAD, page])?;
        let mut reply = [0u8; 48];
        let len = self.get_packet(&mut reply)?;

        if len == 0 {
            return Ok(PACKETRECIEVEERR);
        }

        let result = reply[0];
        if result == OK && len >= 33 {
            bitmap.copy_from_slice(&reply[1..33]);
        }

        Ok(result)
    }

    /// Checks if a template exists at the given location using the template bitmap.
    /// Must call `read_template_page` first for the appropriate page.
    pub fn template_exists_in_bitmap(bitmap: &[u8; 32], index_in_page: u8) -> bool {
        let byte_idx = (index_in_page / 8) as usize;
        let bit_idx = index_in_page % 8;
        if byte_idx < 32 {
            (bitmap[byte_idx] & (1 << bit_idx)) != 0
        } else {
            false
        }
    }

    /// High-speed fingerprint search.
    /// Stores results in `self.finger_id` and `self.confidence`.
    /// Returns the packet error code or OK on success.
    pub fn finger_fast_search(&mut self) -> Result<u8, Error<E>> {
        let capacity = self.library_size().ok_or(Error::NotInitialized)?;

        self.send_packet(&[
            HISPEEDSEARCH,
            0x01,
            0x00,
            0x00,
            (capacity >> 8) as u8,
            (capacity & 0xFF) as u8,
        ])?;

        let mut reply = [0u8; 16];
        let len = self.get_packet(&mut reply)?;

        if len >= 5 {
            self.finger_id = read_u16_be(&reply[1..3]);
            self.confidence = read_u16_be(&reply[3..5]);
        }

        if len == 0 {
            return Ok(PACKETRECIEVEERR);
        }
        Ok(reply[0])
    }

    /// Standard fingerprint search starting at slot 1.
    /// Stores results in `self.finger_id` and `self.confidence`.
    /// Returns the packet error code or OK on success.
    pub fn finger_search(&mut self) -> Result<u8, Error<E>> {
        let capacity = self.library_size().ok_or(Error::NotInitialized)?;

        self.send_packet(&[
            FINGERPRINTSEARCH,
            0x01,
            0x00,
            0x00,
            (capacity >> 8) as u8,
            (capacity & 0xFF) as u8,
        ])?;

        let mut reply = [0u8; 16];
        let len = self.get_packet(&mut reply)?;

        if len >= 5 {
            self.finger_id = read_u16_be(&reply[1..3]);
            self.confidence = read_u16_be(&reply[3..5]);
        }

        if len == 0 {
            return Ok(PACKETRECIEVEERR);
        }
        Ok(reply[0])
    }

    /// Compares two fingerprint templates in char buffers 1 and 2.
    /// Stores confidence score in `self.confidence`.
    /// Returns the packet error code or OK on success.
    pub fn compare_templates(&mut self) -> Result<u8, Error<E>> {
        self.send_packet(&[COMPARE])?;
        let mut reply = [0u8; 16];
        let len = self.get_packet(&mut reply)?;

        if len >= 3 {
            self.confidence = read_u16_be(&reply[1..3]);
        }

        if len == 0 {
            return Ok(PACKETRECIEVEERR);
        }
        Ok(reply[0])
    }

    /// Sets the LED state (only for R503 sensor).
    ///
    /// # Arguments
    /// * `color` - LED color
    /// * `mode` - LED animation mode
    /// * `speed` - Animation speed (0-255)
    /// * `cycles` - Number of cycles (0 = infinite, 1-255)
    ///
    /// Returns the packet error code or OK on success.
    pub fn set_led(
        &mut self,
        color: LedColor,
        mode: LedMode,
        speed: u8,
        cycles: u8,
    ) -> Result<u8, Error<E>> {
        self.send_packet(&[SETAURA, mode as u8, speed, color as u8, cycles])?;
        let mut reply = [0u8; 16];
        let len = self.get_packet(&mut reply)?;
        if len == 0 {
            return Ok(PACKETRECIEVEERR);
        }
        Ok(reply[0])
    }

    /// Performs a soft reset of the sensor.
    pub fn soft_reset(&mut self) -> Result<(), Error<E>> {
        self.send_packet(&[SOFTRESET])?;
        let mut reply = [0u8; 16];
        let len = self.get_packet(&mut reply)?;

        if len > 0 && reply[0] == OK {
            let mut buf = [0u8; 1];
            self.read_exact(&mut buf)?;
            if buf[0] != MODULEOK {
                return Err(Error::HandshakeFailed);
            }
        }
        Ok(())
    }

    // ---- Internal methods ----

    fn send_packet(&mut self, data: &[u8]) -> Result<(), Error<E>> {
        let mut packet = [0u8; MAX_PACKET_SIZE];
        let mut idx = 0;

        // Start code
        packet[idx] = (STARTCODE >> 8) as u8;
        idx += 1;
        packet[idx] = (STARTCODE & 0xFF) as u8;
        idx += 1;

        // Address
        packet[idx..idx + 4].copy_from_slice(&self.address);
        idx += 4;

        // Packet type
        packet[idx] = COMMANDPACKET;
        idx += 1;

        // Length
        let length = data.len() + 2;
        packet[idx] = (length >> 8) as u8;
        idx += 1;
        packet[idx] = (length & 0xFF) as u8;
        idx += 1;

        // Data
        let data_len = data.len().min(MAX_PACKET_SIZE - idx - 2);
        packet[idx..idx + data_len].copy_from_slice(&data[..data_len]);
        idx += data_len;

        // Checksum (from packet type onward)
        let mut checksum: u16 = 0;
        for i in 6..idx {
            checksum = checksum.wrapping_add(packet[i] as u16);
        }
        packet[idx] = (checksum >> 8) as u8;
        idx += 1;
        packet[idx] = (checksum & 0xFF) as u8;
        idx += 1;

        self.write_all(&packet[..idx])?;
        Ok(())
    }

    fn get_packet(&mut self, out: &mut [u8]) -> Result<usize, Error<E>> {
        // Read header (9 bytes): start(2) + address(4) + type(1) + length(2)
        let mut header = [0u8; 9];
        self.read_exact(&mut header)?;

        // Verify start code
        let start = read_u16_be(&header[0..2]);
        if start != STARTCODE {
            return Err(Error::InvalidStartCode);
        }

        // Verify address
        if header[2..6] != self.address {
            return Err(Error::AddressMismatch);
        }

        // Check packet type
        let packet_type = header[6];
        if packet_type != ACKPACKET {
            return Err(Error::InvalidPacketType);
        }

        // Get length
        let length = read_u16_be(&header[7..9]) as usize;
        let payload_len = if length >= 2 { length - 2 } else { 0 };

        // Read payload
        let read_len = payload_len.min(out.len());
        self.read_exact(&mut out[..read_len])?;

        // Skip any remaining payload if buffer is too small
        if payload_len > read_len {
            let mut discard = [0u8; 16];
            let mut remaining = payload_len - read_len;
            while remaining > 0 {
                let chunk = remaining.min(16);
                self.read_exact(&mut discard[..chunk])?;
                remaining -= chunk;
            }
        }

        // Read and discard checksum
        let mut checksum_bytes = [0u8; 2];
        self.read_exact(&mut checksum_bytes)?;

        Ok(read_len)
    }

    fn get_data(&mut self, out: &mut [u8]) -> Result<usize, Error<E>> {
        let mut total_written = 0;

        loop {
            // Read header (9 bytes)
            let mut header = [0u8; 9];
            self.read_exact(&mut header)?;

            // Verify start code
            let start = read_u16_be(&header[0..2]);
            if start != STARTCODE {
                return Err(Error::InvalidStartCode);
            }

            // Verify address
            if header[2..6] != self.address {
                return Err(Error::AddressMismatch);
            }

            let packet_type = header[6];
            if packet_type != DATAPACKET && packet_type != ENDDATAPACKET {
                return Err(Error::InvalidPacketType);
            }

            let length = read_u16_be(&header[7..9]) as usize;
            let payload_len = if length >= 2 { length - 2 } else { 0 };

            // Read payload into output buffer
            let space_left = out.len().saturating_sub(total_written);
            let read_len = payload_len.min(space_left);

            if read_len > 0 {
                self.read_exact(&mut out[total_written..total_written + read_len])?;
                total_written += read_len;
            }

            // Skip any remaining payload if buffer is full
            if payload_len > read_len {
                let mut discard = [0u8; 32];
                let mut remaining = payload_len - read_len;
                while remaining > 0 {
                    let chunk = remaining.min(32);
                    self.read_exact(&mut discard[..chunk])?;
                    remaining -= chunk;
                }
            }

            // Read checksum
            let mut checksum_bytes = [0u8; 2];
            self.read_exact(&mut checksum_bytes)?;

            // If this was the end packet, we're done
            if packet_type == ENDDATAPACKET {
                break;
            }
        }

        Ok(total_written)
    }

    fn send_data(&mut self, data: &[u8]) -> Result<(), Error<E>> {
        let data_length = self
            .data_packet_size()
            .unwrap_or(DataPacketSize::Bytes32)
            .byte_count();

        let total_chunks = (data.len() + data_length - 1) / data_length;

        for i in 0..total_chunks {
            let start = i * data_length;
            let end = ((i + 1) * data_length).min(data.len());
            let chunk = &data[start..end];
            let is_last = i == total_chunks - 1;

            let mut packet = [0u8; 512]; // Max packet size for 256 byte data + header
            let mut idx = 0;

            // Start code
            packet[idx] = (STARTCODE >> 8) as u8;
            idx += 1;
            packet[idx] = (STARTCODE & 0xFF) as u8;
            idx += 1;

            // Address
            packet[idx..idx + 4].copy_from_slice(&self.address);
            idx += 4;

            // Packet type
            let packet_type = if is_last { ENDDATAPACKET } else { DATAPACKET };
            packet[idx] = packet_type;
            idx += 1;

            // Length (chunk + 2 for checksum)
            let length = chunk.len() + 2;
            packet[idx] = (length >> 8) as u8;
            idx += 1;
            packet[idx] = (length & 0xFF) as u8;
            idx += 1;

            // Data
            packet[idx..idx + chunk.len()].copy_from_slice(chunk);
            idx += chunk.len();

            // Checksum (from packet type onward)
            let mut checksum: u16 = 0;
            for j in 6..idx {
                checksum = checksum.wrapping_add(packet[j] as u16);
            }
            packet[idx] = (checksum >> 8) as u8;
            idx += 1;
            packet[idx] = (checksum & 0xFF) as u8;
            idx += 1;

            self.write_all(&packet[..idx])?;
        }

        Ok(())
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error<E>> {
        let mut offset = 0;
        while offset < buf.len() {
            match self.uart.read(&mut buf[offset..]) {
                Ok(0) => {
                    // No data available, continue trying
                    continue;
                }
                Ok(n) => {
                    offset += n;
                }
                Err(e) => return Err(Error::Uart(e)),
            }
        }
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Error<E>> {
        let mut offset = 0;
        while offset < buf.len() {
            match self.uart.write(&buf[offset..]) {
                Ok(0) => {
                    // No data written, continue trying
                    continue;
                }
                Ok(n) => {
                    offset += n;
                }
                Err(e) => return Err(Error::Uart(e)),
            }
        }
        let _ = self.uart.flush();
        Ok(())
    }
}

/// Read a big-endian u16 from a byte slice
#[inline]
fn read_u16_be(bytes: &[u8]) -> u16 {
    ((bytes[0] as u16) << 8) | (bytes[1] as u16)
}
