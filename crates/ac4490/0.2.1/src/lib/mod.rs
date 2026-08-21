#![no_std]

use defmt::{debug, info};

/// AC4490 config structs & enums (channel, output power, etc.)
mod types;
mod util;

use num_enum::TryFromPrimitive;

#[allow(clippy::wildcard_imports)]
pub use types::*;

mod eeprom;

use thiserror_no_std::Error;

use util::hex_to_rssi;

/// Error type for AC4490 operations
#[derive(Error, Debug)]
pub enum Error {
    #[error("Error writing to port")]
    WriteError,

    #[error("Error reading from port")]
    ReadError,

    #[error("Invalid response")]
    InvalidResponse,

    #[error("Invalid response length")]
    InvalidResponseLength,

    #[error("Invalid response address")]
    InvalidResponseAddress,

    #[error("Invalid response data")]
    InvalidResponseData,

    #[error("Invalid response checksum")]
    InvalidResponseChecksum,

    #[error("Invalid response command")]
    InvalidResponseCommand,

    #[error("Invalid response status")]
    InvalidResponseStatus,

    #[error("Invalid response channel")]
    InvalidResponseChannel,

    #[error("Invalid response server/client")]
    InvalidResponseServerClient,

    #[error("Invalid response firmware")]
    InvalidResponseFirmware,

    #[error("Invalid response exit command mode")]
    InvalidExitCommandModeResponse,

    #[error("Data too long")]
    DataTooLong,

    #[error("Failed to construct Vec")]
    FailedToConstructVec,

    #[error("Failed to construct String")]
    FailedToConstructString,

    #[error("Parameter out of range")]
    ParameterOutOfRange,

}

/// Device interface trait for AC4490
/// 
/// This trait is used to abstract the device interface, allowing the AC4490 library to be used with different devices.
/// 
/// # Examples
/// 
/// For a device using `embassy_stm32::usart::Uart`:
/// 
/// ```
/// use ac4490::DeviceInterface;
/// use embassy_stm32::usart::Uart;
/// 
/// struct MyDeviceInterface(Uart<'static, embassy_stm32::mode::Async>);
/// 
/// impl DeviceInterface for MyDeviceInterface {
///     async fn write(&mut self, data: &[u8]) -> Result<(), ac4490::Error> {
///         self.0.write(data).await.map_err(|_| ac4490::Error::WriteError)
///     }
///
///     async fn read(&mut self, data: &mut [u8]) -> Result<(), ac4490::Error> {
///         self.0.read(data).await.map_err(|_| ac4490::Error::ReadError)
///     }
/// }
/// 
/// let mut transceiver = AC4490::new(MyDeviceInterface(uart));
/// ```
/// 
/// For a device using `serialport::SerialPort`:
/// 
/// ```
/// use ac4490::DeviceInterface;
/// use serialport::SerialPort;
/// 
/// struct SerialPortDeviceInterface(Box<dyn serialport::SerialPort>);
/// 
/// impl DeviceInterface for SerialPortDeviceInterface {
///     async fn write(&mut self, data: &[u8]) -> std::result::Result<(), ac4490::Error> {
///         self.0.write_all(data).map_err(|_| ac4490::Error::WriteError)?;
///         Ok(())
///     }
/// 
///     async fn read(&mut self, buf: &mut [u8]) -> std::result::Result<(), ac4490::Error> {
///         self.0.read(buf).map_err(|_| ac4490::Error::ReadError)?;
///         Ok(())
///     }
/// }
/// 
/// async fn main() -> Result<()> {
///     let port = serialport::new("/dev/ttyUSB0", 9600).open()?;
///     let mut transceiver = AC4490::new(SerialPortDeviceInterface(port));
///     Ok(())
/// } 
/// 
/// ```
/// 
pub trait DeviceInterface {

    /// Write data to the device interface.
    fn write(&mut self, data: &[u8]) -> impl core::future::Future<Output = Result<(), Error>> + Send;

    /// Read data from the device interface.
    fn read(&mut self, data: &mut [u8]) -> impl core::future::Future<Output = Result<(), Error>> + Send;
}

/// Instance of AC4490 transceiver, containing all configuration methods.
pub struct AC4490<D>
where D: DeviceInterface + Send {
    port: D,
    debug: bool,
}

impl<D: DeviceInterface + Send> AC4490<D> {

    /// The constructor expects a device interface that implements the [`DeviceInterface`] trait.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::{AC4490, Channel};
    /// let mut transceiver = AC4490::new(device_interface);
    /// ```
    /// 
    #[must_use]
    pub const fn new(port: D) -> Self {
        Self { port, debug: false }
    }

    /// Write data to the transceiver.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::{AC4490, Channel};
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.write(&[0x01, 0x02, 0x03, 0x04]).await;
    /// ```
    /// 
    /// # Errors
    ///
    /// Returns an error if the write fails.
    pub async fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        if let Err(_e) = self.port.write(data).await {
            return Err(Error::WriteError);
        }
        // If debug enabled, print hex bytes
        if self.debug {
            debug!("Wrote: {:?}", data);
        }
        Ok(())
    }

    /// Enable / disable debug messages
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.debug(true);
    /// ```
    /// 
    pub fn debug(&mut self, enable: bool) {
        self.debug = enable;
    }

    /// Read data from the transceiver.
    /// 
    /// The number of bytes to read must be specified.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::{AC4490, Channel};
    /// let mut transceiver = AC4490::new(port);
    /// let mut data = [0; 4];
    /// transceiver.read(&mut data).await;
    /// ```
    /// 
    /// # Errors
    ///
    /// Returns an error if the read fails.
    pub async fn read(&mut self, data: &mut [u8]) -> Result<Option<usize>, Error> {
        let result = self.port.read(data).await;
        match result {
            Ok(()) => {
                if self.debug {
                    debug!("Read: {:?}", data);
                }
                Ok(Some(data.len()))
            }
            Err(_e) => Err(Error::ReadError),
        }
    }


    async fn query(&mut self, request: &[u8], response: &mut [u8]) -> Result<(), Error> {
        info!("Query: {:?}", request);
        self.write(request).await?;
        self.read(response).await?;
        info!("Response: {:?}", response);
        Ok(())
    }

    async fn clear_rx_buffer(&mut self) {
        // Empty the read buffer
        let mut buf = [0; 1];
        while let Ok(Some(_)) = self.read(&mut buf).await {}
    }

    /// Enter command mode by sending `AT+++` to the transceiver.
    /// 
    /// ***Note:** The transceiver must be in command mode to send any other configuration commands.*
    /// 
    /// Alternatively, you can set the CMD / Data pin low.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.enter_command_mode().await;
    /// ```
    /// 
    /// # Errors
    ///
    /// Returns an error if the write fails.
    pub async fn enter_command_mode(&mut self) -> Result<(), Error> {
        self.write(b"AT+++\r").await?;
        // std::thread::sleep(Duration::from_millis(100));

        self.clear_rx_buffer().await;

        Ok(())
    }

    /// Exit command mode.
    /// 
    /// The write/read methods can then be used to transmit / receive from other transceivers via the RF connection.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.exit_command_mode().await;
    /// ```
    /// 
    /// # Errors
    ///
    /// Returns an error if the write or read fails, or the response is invalid.
    pub async fn exit_command_mode(&mut self) -> Result<(), Error> {
        let exit_cmd = [0xCC, 0x41, 0x54, 0x4F, 0x0D];
        let response = &mut [0; 4];
        self.query(&exit_cmd, response).await?;
        match response {
            [0xCC, 0x44, 0x41, 0x54] => Ok(()),
            _ => Err(Error::InvalidExitCommandModeResponse),
        }
    }

    /// The OEM Host issues this command to change the channel of the transceiver.
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::{AC4490, Channel};
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.change_channel(Channel::Ch2A).await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the write or read fails, or if the response is invalid.
    ///
    pub async fn set_channel(&mut self, channel: Channel) -> Result<(), Error> {
        let cmd = [0xCC, 0x02, channel as u8];
        let response = &mut [0; 2];
        self.query(&cmd, response).await?;

        // Check that response = [0xCC, channel]
        if response[0] != 0xCC || response[1] != channel as u8 {
            return Err(Error::InvalidResponse);
        }

        Ok(())
    }

    /// The OEM Host issues this command to change the mode of the transceiver from server to client and vice versa.
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::{AC4490, Channel};
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.set_server_client_mode(ServerClientMode::Server).await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the write or read fails, or if the response is invalid.
    ///
    pub async fn set_server_client_mode(&mut self, mode: ServerClientMode) -> Result<(), Error> {
        let cmd = [0xCC, 0x03, mode.u8_for_cmd()];
        let response = &mut [0; 3];
        self.query(&cmd, response).await?;

        // Check that response = [0xCC, _, mode]
        if response[0] != 0xCC || response[2] != mode.u8_for_cmd() {
            return Err(Error::InvalidResponse);
        }

        Ok(())
    }

    /// The OEM Host issues this command to change the sync channel byte and enable sync to channel.
    ///
    /// Sync-to-channel can be used to synchronize server frequency hopping and prevent
    /// interference between collocated systems. A server transceiver with Sync-to-Channel
    /// enabled must have its Sync Channel set to another server’s RF Channel number. A
    /// server with Sync-to-Channel enabled must set its sync channel to a value less than its
    /// RF Channel number. Collocated networks using Sync-to-Channel must use the same
    /// channel set and system ID.
    /// 
    /// ***Note:** Sync-to-Channel Radio Feature must be enabled.*
    /// 
    /// If server A (with sync-to-channel enabled) can’t sync to server B (on the sync channel),
    /// server A will be unable to communicate with its clients. It must wait until it syncs with
    /// server B (at which point `In_Range` is asserted), before establishing communications.
    /// 
    /// Server B will not be affected and can communicate with its clients.
    /// 
    /// # Examples
    ///
    /// ```
    /// // Set sync channel to Ch2A
    /// use ac4490::{AC4490, Channel};
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.set_sync_channel(Some(Channel::Ch2A)).await;
    /// ```
    ///
    /// ```
    /// // Disable sync channel
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.set_sync_channel(None).await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the write or read fails, or if the response is invalid.
    pub async fn set_sync_channel(&mut self, channel: Option<Channel>) -> Result<(), Error> {
        #[allow(clippy::option_if_let_else)]
        let cmd: &[u8] = match channel {
            Some(channel) => &[0xCC, 0x05, channel as u8],
            None => &[0xCC, 0x85],
        };
        let response = &mut [0; 2];
        self.query(cmd, response).await?;

        // Check that response = [0xCC, channel]
        if let Some(channel) = channel {
            if response[0] != 0xCC || response[1] != channel as u8 {
                return Err(Error::InvalidResponse);
            }
        } else if response[0] != 0xCC {
            return Err(Error::InvalidResponse);
        }

        Ok(())
    }

    /// After the host issues this command, the client transceiver will issue its `In_Range` line logic high after entering
    /// power down. A client in Power Down will remain in sync with a server for a minimum of 2 minutes. To
    /// maintain synchronization with the server, the client should re-sync at least once every 2 minutes. This is done
    /// by sending the Power Down Wake Up command and waiting for the `In_Range` line to issue logic low. Once
    /// this occurs, the client is in sync with the server and can be put back into power-down mode.
    ///
    /// ***Note:** This command is valid only for client transceivers.*
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.sleep_walk_power_down().await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the write or read fails, or if the response is invalid.
    pub async fn sleep_walk_power_down(&mut self) -> Result<(), Error> {
        let cmd = [0xCC, 0x06];
        let response = &mut [0; 2];
        self.query(&cmd, response).await?;

        // Check that response = [0xCC, _]
        match response {
            [0xCC, _] => Ok(()),
            _ => Err(Error::InvalidResponse),
        }
    }

    /// The OEM Host issues this command to bring the client transceiver out of Power Down mode.
    ///
    /// ***Note:** This command is valid only for client transceivers.*
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.sleep_walk_wake_up().await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the write or read fails, or if the response is invalid.
    pub async fn sleep_walk_wake_up(&mut self) -> Result<(), Error> {
        let cmd = [0xCC, 0x07];
        let response = &mut [0; 2];
        self.query(&cmd, response).await?;

        // Check that response = [0xCC, _]
        match response {
            [0xCC, _] => Ok(()),
            _ => Err(Error::InvalidResponse),
        }
    }

    /// The OEM Host issues this command to change the transceiver operation between Addressed Packets and
    /// Broadcast Packets.
    ///
    /// If Addressed Packets are selected, the transceiver will send all packets to the transceiver
    /// designated by the Destination Address programmed in the transceiver. If Broadcast Packets are selected, the
    /// transceiver will send its packets to all transceivers on that network.
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.set_broadcast_enable(true).await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the write or read fails, or if the response is invalid.
    pub async fn set_broadcast_enable(&mut self, broadcast: bool) -> Result<(), Error> {
        let cmd = [0xCC, 0x08, u8::from(broadcast)];
        let response = &mut [0; 2];
        self.query(&cmd, response).await?;

        // Check that response = [0xCC, broadcast as u8]
        if response[0] != 0xCC || response[1] != u8::from(broadcast) {
            return Err(Error::InvalidResponse);
        }

        Ok(())
    }

    /// The OEM Host issues this command to the transceiver to change the Destination Address.
    ///
    /// ***Note:** Only the three least significant bytes of the MAC Address are used for packet delivery.*
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.set_destination_address([0xAB, 0xCA, 0xFE]).await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the write or read fails, or if the response is invalid.
    pub async fn set_destination_address(&mut self, address: [u8; 3]) -> Result<(), Error> {
        let cmd = [0xCC, 0x10, address[0], address[1], address[2]];
        let response = &mut [0; 4];
        self.query(&cmd, response).await?;

        // Check that response = [0xCC, address[0], address[1], address[2]]
        if response[0] != 0xCC
            || response[1] != address[0]
            || response[2] != address[1]
            || response[3] != address[2]
        {
            return Err(Error::InvalidResponse);
        }

        Ok(())
    }

    /// The OEM Host issues this command to the transceiver to read the Destination Address.
    ///
    /// ***Note:** Only the three least significant bytes of the MAC Address are used for packet delivery.*
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let address = transceiver.read_destination_address().await;
    /// println!("Destination Address: {:?}", address);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the write or read fails, or if the response is invalid.
    pub async fn read_destination_address(&mut self) -> Result<[u8; 3], Error> {
        let cmd = [0xCC, 0x11];
        let response = &mut [0; 5];
        self.query(&cmd, response).await?;

        // Check that response = [0xCC ...]
        if response[0] != 0xCC {
            return Err(Error::InvalidResponse);
        }

        let mut address = [0; 3];
        address.copy_from_slice(&response[1..4]);

        Ok(address)
    }

    /// The OEM Host issues this command to the transceiver to force a calibration of the transceiver.
    ///
    /// During the recalibration, the radio will not assert CTS high. Recalibration can take up to 3 seconds and the
    /// command response will not be sent to the OEM Host until recalibration is complete
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let status = transceiver.force_calibration().await;
    /// println!("Transceiver Status: {:?}", status);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the write or read fails, or if the response is invalid.
    pub async fn force_calibration(&mut self) -> Result<Status, Error> {
        let cmd = [0xCC, 0x12, 0x00, 0x00];
        let response = &mut [0; 3];
        self.query(&cmd, response).await?;

        // Check that response = [0xCC, ...]
        if response[0] != 0xCC {
            return Err(Error::InvalidResponse);
        }

        Status::try_from_primitive(response[2]).map_err(|_| Error::InvalidResponseStatus)
    }

    /// The OEM Host issues this command to read the state of both digital input lines.
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let (input0, input1) = transceiver.get_digital_input_state().await;
    /// println!("Input 0: {}, Input 1: {}", input0, input1);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the write or read fails, or if the response is invalid.
    pub async fn get_digital_input_state(&mut self) -> Result<(bool, bool), Error> {
        let cmd = [0xCC, 0x20];
        let response = &mut [0; 2];
        self.query(&cmd, response).await?;

        // Check that response = [0xCC, ...]
        if response[0] != 0xCC {
            return Err(Error::InvalidResponse);
        }

        let input0 = response[1] & 0b0000_0001 == 0b0000_0001;
        let input1 = response[1] & 0b0000_0010 == 0b0000_0010;

        Ok((input0, input1))
    }

    /// The OEM Host issues this command to read any of the three onboard 10-bit A/D converters.
    ///
    /// Because the RF is still active in On-the-Fly Command Mode, the transceiver will not process the command until
    /// there is no activity on the network. The Read RSSI command is therefore useful for detecting interfering sources
    /// but will not report the RSSI from a remote transceiver on the network. The equations for converting these 10 bits
    /// into analog values are as follows:
    /// ```text
    /// Analog Voltage = (10 bits / 0x3FF) * 3.3 V
    /// Temperature (o C) = ((Analog Voltage - 0.3) / 0.01) - 30
    /// Instantaneous RSSI value (dBm) = -105 + (0.22 * (0x3FF - 10 bits))
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::{AC4490, AdcPort};
    /// let mut transceiver = AC4490::new(port);
    /// let adc_value = transceiver.get_adc_value(AdcPort::AdIn).await;
    /// println!("ADC Value: {}", adc_value);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the write or read fails, or if the response is invalid.
    pub async fn get_adc_value(&mut self, port: AdcPort) -> Result<u16, Error> {
        let cmd = [0xCC, 0x21, port as u8];
        let response = &mut [0; 3];
        self.query(&cmd, response).await?;

        // Check that response = [0xCC, ...]
        if response[0] != 0xCC {
            return Err(Error::InvalidResponse);
        }

        Ok(u16::from_be_bytes([response[1], response[2]]))
    }

    /// Since RSSI values are only valid when the local transceiver is receiving an RF packet from a remote transceiver,
    /// instantaneous RSSI can be tricky to use. Therefore, the transceiver stores the most recent valid RSSI value as
    /// measured the last time the transceiver received a packet or beacon. The Host issues this command to retrieve
    /// that value.
    ///
    /// ***Note:** This value will default to 0xFF on a client and 0x00 on a server if no valid RSSI measurement has*
    /// been made since power-up.
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let rssi = transceiver.get_last_valid_rssi().await;
    /// println!("Last Valid RSSI: {}", rssi);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the write or read fails, or if the response is invalid.
    pub async fn get_last_valid_rssi(&mut self) -> Result<i8, Error> {
        let cmd = [0xCC, 0x22];
        let response = &mut [0; 2];
        self.query(&cmd, response).await?;

        // Check that response = [0xCC, ...]
        if response[0] != 0xCC {
            return Err(Error::InvalidResponse);
        }

        Ok(hex_to_rssi(response[1]))
    }

    /// The OEM Host issues this command to write both digital output lines to particular states
    ///
    /// ***Note:** This command should only be used when the Protocol Status / Receive ACK EEPROM setting is not set to 0xE3.*
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.set_digital_output_state(true, false).await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the write or read fails, or if the response is invalid.
    pub async fn set_digital_output_state(&mut self, go0: bool, go1: bool) -> Result<(), Error> {
        let cmd = [0xCC, 0x23, u8::from(go0) | u8::from(go1) << 1];
        let response = &mut [0; 2];
        self.query(&cmd, response).await?;

        // Check that response = [0xCC, ...]
        if response[0] != 0xCC {
            return Err(Error::InvalidResponse);
        }

        Ok(())
    }

    /// The OEM Host issues this command to write `DA_Out` to a particular voltage.
    ///
    /// The transceiver uses a PWM (Pulse Width Modulator) to generate the analog voltage. The theory behind a PWM
    /// is that a binary pulse is generated with a fixed rate (<Data 1>) and duty cycle (<Data 2>). As such, this pin
    /// toggles between High & Low. This signal is filtered via an on-board R-C circuit and an analog voltage is generated.
    ///
    /// Duty cycle specifies the ratio of time in one cycle that the pulse spends High proportionate to the amount of
    /// time it spends Low. So, with a duty cycle of 50% (0x80), the pulse is High 50% of the time and Low 50% of
    /// the time; therefore the analog voltage would be half of 3.3V or 1.65V. A broad filter has been implemented
    /// on the transceiver and there is no advantage to using a slower update period. Generally, a faster update
    /// period is preferred.
    ///
    /// `period` is in units of ~0.06s (precisely 14.7456/255 seconds)
    ///
    /// ***Note:** The duty cycle is represented at this pin as an analog voltage. 50% duty cycle is half of 3.3V or 1.65V.*
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.set_dac_output_state(0x80, 0x80).await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the write or read fails, or if the response is invalid.
    pub async fn set_dac_output_state(
        &mut self,
        update_period: u8,
        duty_cycle: u8,
    ) -> Result<(), Error> {
        let cmd = [0xCC, 0x24, update_period, duty_cycle];
        let response = &mut [0; 3];
        self.query(&cmd, response).await?;

        // Check that response = [0xCC, update_period, duty_cycle]
        if response[0] != 0xCC || response[1] != update_period || response[2] != duty_cycle {
            return Err(Error::InvalidResponse);
        }

        Ok(())
    }

    /// The OEM Host issues this command to limit the maximum transmit power emitted by the transceiver.
    ///
    /// This can be useful to minimize current consumption and satisfy certain regulatory requirements.
    ///
    /// ***Note:** The radios are shipped at maximum allowable power.*
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::{AC4490, OutputPower};
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.set_max_power(OutputPower::Plus16_5dBm).await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the write or read fails, or if the response is invalid.
    pub async fn set_max_power(&mut self, power: OutputPower) -> Result<(), Error> {
        let cmd = [0xCC, 0x25, power as u8];
        let response = &mut [0; 2];
        self.query(&cmd, response).await?;

        // Check that response = [0xCC, power]
        if response[0] != 0xCC || response[1] != power as u8 {
            return Err(Error::InvalidResponse);
        }

        Ok(())
    }

    /// Not documented in AC4490 user guide.
    ///
    /// # Errors
    ///
    /// Returns an error if the write or read fails, or if the response is invalid.
    pub async fn get_last_packet_rssi(&mut self) -> Result<u8, Error> {
        let cmd = [0xCC, 0x26];
        let response = &mut [0; 2];
        self.query(&cmd, response).await?;

        // Check that response = [0xCC, ...]
        if response[0] != 0xCC {
            return Err(Error::InvalidResponse);
        }

        Ok(response[1])
    }

    /// The OEM Host issues this command to temporarily enable or disable Long Range Mode in the transceiver.
    ///
    /// ***Note:** Only available on AC4490LR-1000 transceivers with firmware v6.7+*
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.set_long_range_mode(true).await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the write or read fails, or if the response is invalid.
    pub async fn set_long_range_mode(&mut self, enable: bool) -> Result<(), Error> {
        let cmd = [0xCC, 0x27, u8::from(enable)];
        let response = &mut [0; 2];
        self.query(&cmd, response).await?;

        // Check that response = [0xCC, enable]
        if response[0] != 0xCC || response[1] != u8::from(enable) {
            return Err(Error::InvalidResponse);
        }

        Ok(())
    }

    /// The OEM Host issues this command to determine when the RF transmit buffer is empty.
    ///
    /// The Host will not receive the transceiver response until that time.
    ///
    /// # Examples
    ///     
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.wait_for_transmit_buffer_empty().await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the write or read fails, or if the response is invalid.
    pub async fn wait_for_transmit_buffer_empty(&mut self) -> Result<(), Error> {
        let cmd = [0xCC, 0x30];
        let response = &mut [0; 2];
        self.query(&cmd, response).await?;

        // Check that response = [0xCC, 0x00]
        if response[0] != 0xCC || response[1] != 0x00 {
            return Err(Error::InvalidResponse);
        }

        Ok(())
    }

    /// The OEM Host issues this command to put the transceiver into Deep Sleep mode.
    ///
    /// Once in Deep Sleep mode, the transceiver disables all RF communications and will not respond to any
    /// further commands until being reset or power-cycled.
    ///
    /// ***Note:** This command is valid for both servers and clients.*
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.enter_deep_sleep_mode().await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the write or read fails, or if the response is invalid.
    pub async fn enter_deep_sleep_mode(&mut self) -> Result<(), Error> {
        let cmd = [0xCC, 0x86];
        let response = &mut [0; 2];
        self.query(&cmd, response).await?;

        // Check that response = [0xCC, ...]
        if response[0] != 0xCC {
            return Err(Error::InvalidResponse);
        }

        Ok(())
    }

    /// When the OEM Host issues this command, the server transceiver sends out a query every 500 ms. The client
    /// transceivers randomly choose a query to respond to. After responding to a probe, a client transceiver will
    /// wait at least 10 seconds before responding to another probe.
    ///
    /// ***Note:** This command can only be sent from a server radio.*
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.set_probe_enabled(true).await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the write or read fails, or if the response is invalid.
    pub async fn set_probe_enabled(&mut self, enable: bool) -> Result<(), Error> {
        let cmd = [0xCC, 0x8E, u8::from(enable)];
        let response = &mut [0; 2];
        self.query(&cmd, response).await?;

        // Check that response = [0xCC, enable]
        if response[0] != 0xCC || response[1] != u8::from(enable) {
            return Err(Error::InvalidResponse);
        }

        Ok(())
    }

    /// The OEM Host issues this command to read the onboard temperature sensor.
    ///
    /// The reported temperature is in degrees Celsius.
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let temperature = transceiver.get_temperature().await;
    /// println!("Temperature: {}", temperature);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the write or read fails, or if the response is invalid.
    pub async fn get_temperature(&mut self) -> Result<i8, Error> {
        let cmd = [0xCC, 0xA4];
        let response = &mut [0; 2];
        self.query(&cmd, response).await?;

        // Check that response = [0xCC, ...]
        if response[0] != 0xCC {
            return Err(Error::InvalidResponse);
        }

        #[allow(clippy::cast_possible_wrap)]
        Ok(response[1] as i8)
    }

    /// The OEM Host issues this command to read the temperature at the time of its last calibration.
    ///
    /// The reported temperature is in degrees Celsius.
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let temperature = transceiver.get_temperature_at_last_calibration().await;
    /// println!("Temperature at Last Calibration: {}", temperature);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the write or read fails, or if the response is invalid.
    pub async fn get_temperature_at_last_calibration(&mut self) -> Result<i8, Error> {
        let cmd = [0xCC, 0xA5];
        let response = &mut [0; 2];
        self.query(&cmd, response).await?;

        // Check that response = [0xCC, ...]
        if response[0] != 0xCC {
            return Err(Error::InvalidResponse);
        }

        #[allow(clippy::cast_possible_wrap)]
        Ok(response[1] as i8)
    }

    /// The OEM Host issues this command to perform a soft reset of the transceiver.
    ///
    /// Any transceiver settings modified by runtime commands will revert to the values stored in the EEPROM.
    ///
    /// ***Note:** This command will return as soon as the reset command is transmitted.*
    /// The transceiver may not have completed its reset at that time.
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.soft_reset().await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the write fails.
    pub async fn soft_reset(&mut self) -> Result<(), Error> {
        let cmd = [0xCC, 0xFF];
        self.write(&cmd).await
    }

    async fn eeprom_operation_raw<const N: usize>(
        &mut self,
        parameter: eeprom::Parameter,
        write_value: Option<&[u8]>,
    ) -> Result<[u8; N], Error> {
        let cmd = [
            0xCC,
            write_value.map_or(0xC0, |_| 0xC1),
            parameter as u8,
            parameter.length(),
        ];
        self.write(&cmd).await?;
        if let Some(value) = write_value {
            self.write(value).await?;
        }

        let mut response_meta = [0; 2];
        self.read(&mut response_meta).await?;
        if response_meta[0] != parameter as u8
            || response_meta[1] != parameter.length()
        {
            return Err(Error::InvalidResponse);
        }

        let mut response = [0; N];
        self.read(&mut response).await?;
        Ok(response)
    }

    async fn eeprom_read<const N: usize>(
        &mut self,
        parameter: eeprom::Parameter,
    ) -> Result<[u8; N], Error> {
        self.eeprom_operation_raw::<N>(parameter, None).await
    }

    async fn eeprom_read_byte(&mut self, parameter: eeprom::Parameter) -> Result<u8, Error> {
        self.eeprom_operation_raw::<1>(parameter, None)
            .await
            .map(|r| r[0])
    }

    // async fn eeprom_read_type<const N: usize, T: TryFrom<[u8; N], Error = Error>>(
    //     &mut self,
    //     parameter: eeprom::Parameter,
    // ) -> Result<T, Error> {
    //     let response = self.eeprom_operation_raw::<N>(parameter, None).await?;
    //     T::try_from(response).map_err(|_| Error::InvalidResponse)
    // }

    async fn eeprom_read_byte_type<T: TryFromPrimitive<Primitive = u8>>(
        &mut self,
        parameter: eeprom::Parameter,
    ) -> Result<T, Error> {
        let response = self.eeprom_read_byte(parameter).await?;
        T::try_from_primitive(response).map_err(|_| Error::InvalidResponse)
    }

    async fn eeprom_write(
        &mut self,
        parameter: eeprom::Parameter,
        value: &[u8],
    ) -> Result<(), Error> {
        self.eeprom_operation_raw::<1>(parameter, Some(value))
            .await?;
        Ok(())
    }

    async fn eeprom_write_byte(
        &mut self,
        parameter: eeprom::Parameter,
        value: u8,
    ) -> Result<(), Error> {
        self.eeprom_write(parameter, &[value]).await
    }

    /// Product identifier string. Includes revision information for software and hardware.
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let product_id = transceiver.eeprom_get_product_id_string().await;
    /// println!("Product ID: {:?}", product_id);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_product_id_string(&mut self) -> Result<[u8; 40], Error> {
        self.eeprom_read::<40>(eeprom::Parameter::ProductIdString)
            .await
    }

    /// Sets the range refresh EEPROM setting.
    ///
    /// This value represents the number of times a data packet will be transmitted by a
    /// transceiver in Broadcast Mode. The default value is 4 attempts. If communication
    /// is lost and the clients' Link LED is on, try increasing this value in small increments
    /// until communication is reestablished. Valid values for this field are 1 - 255.
    /// Note: All Broadcast Attempts are used whether the packet was received
    /// without error by the receiving radios or not
    ///
    /// **Default:** 0x18
    ///
    /// ***Note:** Do not set to 0x00.*
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_range_refresh(0x18).await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_set_range_refresh(&mut self, value: u8) -> Result<(), Error> {
        self.eeprom_write_byte(eeprom::Parameter::PageRefresh, value)
            .await
    }

    /// Gets the range refresh EEPROM setting.
    ///
    /// This value represents the number of times a data packet will be transmitted by a
    /// transceiver in Broadcast Mode. The default value is 4 attempts. If communication
    /// is lost and the clients' Link LED is on, try increasing this value in small increments
    /// until communication is reestablished. Valid values for this field are 1 - 255.
    /// Note: All Broadcast Attempts are used whether the packet was received
    /// without error by the receiving radios or not
    ///
    /// **Default:** 0x18
    ///
    /// ***Note:** Do not set to 0x00.*
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let page_refresh = transceiver.eeprom_get_range_refresh(0x18).await;
    /// println!("Page Refresh: {}", page_refresh);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_range_refresh(&mut self) -> Result<u8, Error> {
        self.eeprom_operation_raw::<1>(eeprom::Parameter::PageRefresh, None)
            .await
            .map(|r| r[0])
    }

    /// Sets the stop bit delay.
    ///
    /// For systems employing Parity, the serial stop bit might come too early. Stop Bit Delay
    /// controls the width of the last bit before the stop bit occurs.
    ///
    /// ```text
    /// 0xFF = Disable Stop Bit Delay (12 μs)
    /// 0x00 = (256 * 1.6 μs) + 12 μs
    /// 0x01 - 0xFE = (value * 1.6 μs) + 12 μs
    /// ```
    ///
    /// **Default:** 0xFF
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_stop_bit_delay(0xFF).await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_set_stop_bit_delay(&mut self, value: u8) -> Result<(), Error> {
        self.eeprom_write(eeprom::Parameter::StopBitDelay, &[value])
            .await
    }

    /// Gets the stop bit delay.
    ///
    /// For systems employing Parity, the serial stop bit might come too early. Stop Bit Delay
    /// controls the width of the last bit before the stop bit occurs.
    ///
    /// ```text
    /// 0xFF = Disable Stop Bit Delay (12 μs)
    /// 0x00 = (256 * 1.6 μs) + 12 μs
    /// 0x01 - 0xFE = (value * 1.6 μs) + 12 μs
    /// ```
    ///
    /// **Default:** 0xFF
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_stop_bit_delay(0xFF).await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_stop_bit_delay(&mut self) -> Result<u8, Error> {
        self.eeprom_read_byte(eeprom::Parameter::StopBitDelay).await
    }

    /// Sets the channel number used at startup.
    ///
    /// **Default:** 0x00 (1x1/200), 0x10 (1000)
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_channel_number(0x00).await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_set_channel_number(&mut self, value: Channel) -> Result<(), Error> {
        self.eeprom_write_byte(eeprom::Parameter::ChannelNumber, value as u8)
            .await
    }

    /// Gets the channel number used at startup.
    ///
    /// **Default:** 0x00 (1x1/200), 0x10 (1000)
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let channel = transceiver.eeprom_get_channel_number().await;
    /// println!("Channel Number: {:?}", channel);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_channel_number(&mut self) -> Result<Channel, Error> {
        self.eeprom_read_byte_type::<Channel>(eeprom::Parameter::ChannelNumber)
            .await
    }

    /// Sets the server / client mode EEPROM setting.
    /// 
    /// Designates AC4490 type. In each network, there must be only one server. All other
    /// AC4490 units must be programmed as clients. The number of clients in the network
    /// is not limited; however, if performance diminishes, consider additional RF Networks.
    ///
    /// **Default:** `ServerClientMode::Client`
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::{AC4490, ServerClientMode}
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_server_client_mode(ServerClientMode::Server).await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_set_server_client_mode(
        &mut self,
        value: ServerClientMode,
    ) -> Result<(), Error> {
        self.eeprom_write_byte(eeprom::Parameter::ServerClientMode, value.u8_for_eeprom())
            .await
    }

    /// Gets the server / client mode EEPROM setting.
    ///
    /// Designates AC4490 type. In each network, there must be only one server. All other
    /// AC4490 units must be programmed as clients. The number of clients in the network
    /// is not limited; however, if performance diminishes, consider additional RF Networks.
    /// 
    /// **Default:** `ServerClientMode::Client`
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::{AC4490, ServerClientMode}
    /// let mut transceiver = AC4490::new(port);
    /// let mode = transceiver.eeprom_get_server_client_mode().await;
    /// println!("Server / Client Mode: {:?}", mode);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_server_client_mode(&mut self) -> Result<ServerClientMode, Error> {
        let byte = self
            .eeprom_read_byte(eeprom::Parameter::ServerClientMode)
            .await?;

        ServerClientMode::from_eeprom(byte).map_err(|_| Error::InvalidResponse)
    }

    // TODO: Clarify with Ezurio why 0xFC = 57_600 baud rate, before implementing baud rate

    /// Set Control0 register
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::{AC4490, Control0};
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_control0(Control0 {
    ///     one_beacon_mode: false,
    ///     des_encryption: false,
    ///     sync_to_channel: false,
    ///     broadcast_packets: true,
    /// }).await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_set_control0(&mut self, value: Control0) -> Result<(), Error> {
        let current_byte = self.eeprom_read_byte(eeprom::Parameter::Control0).await?;

        // Do not modify "Laird Use Only" bits
        let new_byte = (current_byte & 0b0001_1101) | u8::from(value);

        self.eeprom_write_byte(eeprom::Parameter::Control0, new_byte)
            .await
    }

    /// Get Control0 register
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let control0 = transceiver.eeprom_get_control0().await;
    /// println!("Control0: {:?}", control0);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_control0(&mut self) -> Result<Control0, Error> {
        let byte = self.eeprom_read_byte(eeprom::Parameter::Control0).await?;
        Ok(Control0::from(byte))
    }

    /// Set frequency offset
    ///
    /// Frequency offset is a protocol parameter used in conjunction
    /// with Channel Number to satisfy unique regulations.
    ///
    /// **Default:** 0x01
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_frequency_offset(0x01).await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_set_frequency_offset(&mut self, value: u8) -> Result<(), Error> {
        self.eeprom_write_byte(eeprom::Parameter::FrequencyOffset, value)
            .await
    }

    /// Get frequency offset
    ///
    /// Frequency offset is a protocol parameter used in conjunction
    /// with Channel Number to satisfy unique regulations.
    ///
    /// **Default:** 0x01
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490
    /// let mut transceiver = AC4490::new(port);
    /// let frequency_offset = transceiver.eeprom_get_frequency_offset().await;
    /// println!("Frequency Offset: {}", frequency_offset);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_frequency_offset(&mut self) -> Result<u8, Error> {
        self.eeprom_read_byte(eeprom::Parameter::FrequencyOffset)
            .await
    }

    /// Enable / disable the `CMD/Data RX` pin
    ///
    /// **Default:** true
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_cmd_data_rx_enable(true).await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_set_cmd_data_rx_enable(&mut self, enable: bool) -> Result<(), Error> {
        // Note: the Laird documentation specifies  0xFF = "Disable CMD/Data Rx Disable".
        // Here the values have been inverted to avoid the double-negative in the API.
        self.eeprom_write_byte(
            eeprom::Parameter::CmdDataRxDisable,
            if enable { 0xFF } else { 0xE3 },
        )
        .await
    }

    /// Check whether the `CMD/Data RX` pin is enabled
    ///
    /// **Default:** true
    ///
    /// # Examples
    ///
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let enabled = transceiver.eeprom_get_cmd_data_rx_enable().await;
    /// println!("CMD/Data RX Enabled: {}", enabled);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_cmd_data_rx_enable(&mut self) -> Result<bool, Error> {
        match self
            .eeprom_read_byte(eeprom::Parameter::CmdDataRxDisable)
            .await?
        {
            0xFF => Ok(true),
            0xE3 => Ok(false),
            _ => Err(Error::InvalidResponse),
        }
    }

    /// Set the Transmit Retries EEPROM setting.
    /// 
    /// This value represents the maximum number of times a particular data packet will
    /// be transmitted unsuccessfully, or without an acknowledgement, before the
    /// AC4490 discards the packet. The default value is 16 attempts. If communication
    /// is lost and the client's Link LED is on, try increasing this value in small increments
    /// until communication is reestablished.
    /// 
    /// ***Note:** Do not set to 0.*
    /// 
    /// **Default:** 0x10
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_transmit_retries(0x10).await;
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the value or response is invalid.
    pub async fn eeprom_set_transmit_retries(&mut self, value: u8) -> Result<(), Error> {
        if value == 0 {
            return Err(Error::ParameterOutOfRange);
        }
        self.eeprom_write_byte(eeprom::Parameter::TransmitRetries, value)
            .await
    }

    /// Get the Transmit Retries EEPROM setting.
    /// 
    /// Transmit Retries is the maximum number of times a packet is
    /// transmitted when broadcasting is disabled, and an
    /// acknowledgement is not received.
    /// 
    /// **Default:** 0x10
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let transmit_retries = transceiver.eeprom_get_transmit_retries().await;
    /// println!("Transmit Retries: {}", transmit_retries);
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_transmit_retries(&mut self) -> Result<u8, Error> {
        self.eeprom_read_byte(eeprom::Parameter::TransmitRetries)
            .await
    }

    /// Se the Broadcast Attempts EEPROM setting.
    /// 
    /// This value represents the number of times a data packet will be transmitted by a
    /// transceiver in Broadcast Mode. The default value is 4 attempts. If communication
    /// is lost and the clients' Link LED is on, try increasing this value in small increments
    /// until communication is reestablished. Valid values for this field are 1 - 255.
    /// 
    /// ***Note:** All Broadcast Attempts are used whether the packet was received
    /// without error by the receiving radios or not.*
    /// 
    /// **Default:** 0x04
    /// 
    /// ***Note:** Do not set to 0.*
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_broadcast_attempts(0x04).await;
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the value or response is invalid.
    pub async fn eeprom_set_broadcast_attempts(&mut self, value: u8) -> Result<(), Error> {
        if value == 0 {
            return Err(Error::ParameterOutOfRange);
        }
        self.eeprom_write_byte(eeprom::Parameter::BroadcastAttempts, value)
            .await
    }

    /// Get the Broadcast Attempts EEPROM setting.
    /// 
    /// Broadcast Attempts is the number of times a packet is transmitted
    /// when broadcasting is enabled.
    /// 
    /// **Default:** 0x04
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let broadcast_attempts = transceiver.eeprom_get_broadcast_attempts().await;
    /// println!("Broadcast Attempts: {}", broadcast_attempts);
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_broadcast_attempts(&mut self) -> Result<u8, Error> {
        self.eeprom_read_byte(eeprom::Parameter::BroadcastAttempts)
            .await
    }

    /// Set the API Control EEPROM setting.
    /// 
    /// **Default:**
    /// ```
    /// ApiControl {
    ///     unicast_only: false,        // Disable Unicast Only
    ///     auto_destination: false,    // Use destination address
    ///     client_auto_channel: false, // Disable Auto Channel
    ///     rts_enable: false,          // Ignore RTS
    ///     duplex: true,               // Full Duplex
    ///     auto_config: true,          // Auto Configure Values
    /// }
    /// ```
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::{AC4490, ApiControl};
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_api_control(ApiControl {
    ///     unicast_only: false,
    ///     auto_destination: false,
    ///     client_auto_channel: false,
    ///     rts_enable: false,
    ///     duplex: true,
    ///     auto_config: true,
    /// }).await;
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_set_api_control(&mut self, value: ApiControl) -> Result<(), Error> {
        let current_byte = self.eeprom_read_byte(eeprom::Parameter::ApiControl).await?;

        // Do not modify "Laird Use Only" bits
        let new_byte = (current_byte & 0b1100_0000) | u8::from(value);

        self.eeprom_write_byte(eeprom::Parameter::ApiControl, new_byte)
            .await
    }

    /// Get the API Control EEPROM setting.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let api_control = transceiver.eeprom_get_api_control().await;
    /// println!("API Control: {:?}", api_control);
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_api_control(&mut self) -> Result<ApiControl, Error> {
        let byte = self.eeprom_read_byte(eeprom::Parameter::ApiControl).await?;
        Ok(ApiControl::from(byte))
    }

    /// Set the Interface Timeout EEPROM setting.
    /// 
    /// Interface Timeout specifies a byte gap timeout, used in conjunction with RF Packet Size to
    /// determine when a packet coming over the interface is complete (0.5 ms per increment).
    /// 
    /// When that byte gap is exceeded, the bytes in the transmit buffer are sent out
    /// over the RF as a complete packet. Interface Timeout is adjustable in 0.5 ms
    /// increments and has a tolerance of ±0.5 ms. The Interface Timeout should not be
    /// set below 2. The default value for Interface Timeout is 0x04 (2 ms) and should be
    /// adjusted accordingly when changing the transceiver baud rate.
    /// 
    /// **Default:** 0x04
    /// 
    /// ***Note:** Minimum value of 0x02.*
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_interface_timeout(0x04).await;
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the value or response is invalid.
    pub async fn eeprom_set_interface_timeout(&mut self, value: u8) -> Result<(), Error> {
        if value < 0x02 {
            return Err(Error::ParameterOutOfRange);
        }
        self.eeprom_write_byte(eeprom::Parameter::InterfaceTimeout, value)
            .await
    }

    /// Get the Interface Timeout EEPROM setting.
    /// 
    /// Interface Timeout specifies a byte gap timeout, used in conjunction with RF Packet Size to
    /// determine when a packet coming over the interface is complete (0.5 ms per increment).
    /// 
    /// **Default:** 0x04
    /// 
    /// ***Note:** Minimum value of 0x02.*
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let interface_timeout = transceiver.eeprom_get_interface_timeout().await;
    /// println!("Interface Timeout: {}", interface_timeout);
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_interface_timeout(&mut self) -> Result<u8, Error> {
        self.eeprom_read_byte(eeprom::Parameter::InterfaceTimeout)
            .await
    }

    /// Set the Sync Channel EEPROM setting
    /// 
    /// Sync-to-channel can be used to synchronize server frequency hopping and prevent
    /// interference between collocated systems. A server transceiver with Sync-to-Channel
    /// enabled must have its Sync Channel set to another server’s RF Channel number. A
    /// server with Sync-to-Channel enabled must set its sync channel to a value less than its
    /// RF Channel number. Collocated networks using Sync-to-Channel must use the same
    /// channel set and system ID.
    /// 
    /// ***Note:** Sync-to-Channel Radio Feature must be enabled.*
    /// 
    /// If server A (with sync-to-channel enabled) can’t sync to server B (on the sync channel),
    /// server A will be unable to communicate with its clients. It must wait until it syncs with
    /// server B (at which point `In_Range` is asserted), before establishing communications.
    /// 
    /// Server B will not be affected and can communicate with its clients.
    /// 
    /// **Default:** Ch01
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_sync_channel(Channel::Ch01).await;
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_set_sync_channel(&mut self, value: Channel) -> Result<(), Error> {
        self.eeprom_write_byte(eeprom::Parameter::SyncChannel, value as u8)
            .await
    }

    /// Get the Sync Channel EEPROM setting
    ///
    /// Sync-to-channel can be used to synchronize server frequency hopping and prevent
    /// interference between collocated systems. A server transceiver with Sync-to-Channel
    /// enabled must have its Sync Channel set to another server’s RF Channel number. A
    /// server with Sync-to-Channel enabled must set its sync channel to a value less than its
    /// RF Channel number. Collocated networks using Sync-to-Channel must use the same
    /// channel set and system ID.
    /// 
    /// ***Note:** Sync-to-Channel Radio Feature must be enabled.*
    /// 
    /// If server A (with sync-to-channel enabled) can’t sync to server B (on the sync channel),
    /// server A will be unable to communicate with its clients. It must wait until it syncs with
    /// server B (at which point `In_Range` is asserted), before establishing communications.
    /// 
    /// Server B will not be affected and can communicate with its clients.
    /// 
    /// **Default:** Ch01
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let sync_channel = transceiver.eeprom_get_sync_channel().await;
    /// println!("Sync Channel: {:?}", sync_channel);
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_sync_channel(&mut self) -> Result<Channel, Error> {
        self.eeprom_read_byte_type::<Channel>(eeprom::Parameter::SyncChannel)
            .await
    }

    /// Set the RF Packet Size EEPROM setting
    /// 
    /// Used in conjunction with Interface Timeout; specifies the maximum size of an RF packet.
    /// 
    /// RF Packet Size is used in conjunction with Interface Timeout to determine when
    /// to delineate incoming data as an entire packet based on whichever condition is
    /// met first. When the transceiver receives the number of bytes specified by RF
    /// Packet Size without experiencing a byte gap equal to Interface Timeout, that
    /// block of data is processed as a complete packet. Every packet the transceiver
    /// sends over the RF contains extra header bytes not counted in the RF Packet Size. It
    /// is much more efficient to send a few large packets than to send many short
    /// packets.
    /// 
    /// **Default:** 0x80
    /// 
    /// ***Note:** Must be set to a minimum of 6 in order to send the Enter AT command.*
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_rf_packet_size(0x80).await;
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the value or response is invalid.
    pub async fn eeprom_set_rf_packet_size(&mut self, value: u8) -> Result<(), Error> {
        if value == 0 {
            return Err(Error::ParameterOutOfRange);
        }
        self.eeprom_write_byte(eeprom::Parameter::RfPacketSize, value)
            .await
    }

    /// Get the RF Packet Size EEPROM setting
    /// 
    /// Used in conjunction with Interface Timeout; specifies the maximum size of an RF packet
    /// 
    /// **Default:** 0x80
    /// 
    /// ***Note:** Must be set to a minimum of 6 in order to send the Enter AT command.*
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let rf_packet_size = transceiver.eeprom_get_rf_packet_size().await;
    /// println!("RF Packet Size: {}", rf_packet_size);
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_rf_packet_size(&mut self) -> Result<u8, Error> {
        self.eeprom_read_byte(eeprom::Parameter::RfPacketSize)
            .await
    }

    /// Set the CTS On EEPROM setting.
    /// 
    /// CTS will be deasserted (High) when the transmit buffer contains at least this many characters.
    /// 
    /// **Default:** 0xD2
    /// 
    /// ***Note:** Must be set to a minimum of 0x01.*
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_cts_on(0xD2).await;
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the value or response is invalid.
    pub async fn eeprom_set_cts_on(&mut self, value: u8) -> Result<(), Error> {
        if value == 1 {
            return Err(Error::ParameterOutOfRange);
        }
        self.eeprom_write_byte(eeprom::Parameter::CtsOn, value)
            .await
    }

    /// Get the CTS On EEPROM setting.
    /// 
    /// CTS will be deasserted (High) when the transmit buffer contains at least this many characters.
    /// 
    /// **Default:** 0xD2
    /// 
    /// ***Note:** Must be set to a minimum of 0x01.*
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let cts_on = transceiver.eeprom_get_cts_on().await;
    /// println!("CTS On: {}", cts_on);
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_cts_on(&mut self) -> Result<u8, Error> {
        self.eeprom_read_byte(eeprom::Parameter::CtsOn).await
    }

    /// Set the CTS On Hysterisis EEPROM setting.
    /// 
    /// Once CTS has been deasserted, CTS will be reasserted (Low) when the transmit
    /// buffer is contains this many or less characters.
    /// 
    /// **Default:** 0xAC
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_cts_on_hysterisis(0xAC).await;
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the value or response is invalid.
    pub async fn eeprom_set_cts_on_hysterisis(&mut self, value: u8) -> Result<(), Error> {
        self.eeprom_write_byte(eeprom::Parameter::CtsOnHysterisis, value)
            .await
    }

    /// Set the Max Power EEPROM setting
    /// 
    /// Max Power provides a means for controlling the RF output power of the AC4490.
    /// 
    /// Output power and current consumption can vary by as much as ±10% per
    /// transceiver for a particular Max Power setting. Contact Laird for assistance in
    /// adjusting Max Power.
    /// 
    /// Note: The max power is set during Production and may vary slightly from one
    /// transceiver to another. The max power can be set as low as desired but
    /// should not be set higher than the original factory setting. A backup of the
    /// original power setting is stored in EEPROM address 0x8E, retrievable using
    /// `eeprom_get_original_max_power()`.
    /// 
    /// **Default:** Set in production & can vary
    /// 
    /// ***Note:** The transceivers are shipped at maximum allowable power.*
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::{AC4490, OutputPower};
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_max_power(OutputPower::Plus16_5dBm).await;
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_set_max_power(&mut self, value: OutputPower) -> Result<(), Error> {
        self.eeprom_write_byte(eeprom::Parameter::MaxPower, value as u8)
            .await
    }

    /// Get the Max Power EEPROM setting
    /// 
    /// Max Power provides a means for controlling the RF output power of the AC4490.
    /// 
    /// Output power and current consumption can vary by as much as ±10% per
    /// transceiver for a particular Max Power setting. Contact Laird for assistance in
    /// adjusting Max Power.
    /// 
    /// Note: The max power is set during Production and may vary slightly from one
    /// transceiver to another. The max power can be set as low as desired but
    /// should not be set higher than the original factory setting. A backup of the
    /// original power setting is stored in EEPROM address 0x8E, retrievable using
    /// `eeprom_get_original_max_power()`.
    /// 
    /// **Default:** Set in production & can vary
    /// 
    /// ***Note:** The transceivers are shipped at maximum allowable power.*
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::{AC4490, OutputPower};
    /// let mut transceiver = AC4490::new(port);
    /// let max_power = transceiver.eeprom_get_max_power().await;
    /// println!("Max Power: {:?}", max_power);
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_max_power(&mut self) -> Result<OutputPower, Error> {
        self.eeprom_read_byte_type::<OutputPower>(eeprom::Parameter::MaxPower)
            .await
    }

    /// Enable / disable the Modem Mode EEPROM setting
    /// 
    /// **Default:** false
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_modem_mode_enable(true).await;
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_set_modem_mode_enable(&mut self, enable: bool) -> Result<(), Error> {
        self.eeprom_write_byte(
            eeprom::Parameter::ModemMode,
            if enable { 0xE3 } else { 0xFF },
        )
        .await
    }

    /// Check whether the Modem Mode EEPROM setting is enabled
    /// 
    /// **Default:** false
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let enabled = transceiver.eeprom_get_modem_mode_enable().await;
    /// println!("Modem Mode Enabled: {}", enabled);
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_modem_mode_enable(&mut self) -> Result<bool, Error> {
        match self
            .eeprom_read_byte(eeprom::Parameter::ModemMode)
            .await?
        {
            0xFF => Ok(false),
            0xE3 => Ok(true),
            _ => Err(Error::InvalidResponse),
        }
    }

    /// Enable / disable the Parity setting
    /// 
    /// **Default:** false
    /// 
    /// ***Note:** Enabling parity cuts throughput and the interface buffer size in half.*
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_parity_enable(true).await;
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_set_parity_enable(&mut self, enable: bool) -> Result<(), Error> {
        self.eeprom_write_byte(
            eeprom::Parameter::Parity,
            if enable { 0xE3 } else { 0xFF },
        )
        .await
    }

    /// Check whether the Parity setting is enabled
    /// 
    /// **Default:** false
    /// 
    /// ***Note:** Enabling parity cuts throughput and the interface buffer size in half.*
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let enabled = transceiver.eeprom_get_parity_enable().await;
    /// println!("Parity Enabled: {}", enabled);
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_parity_enable(&mut self) -> Result<bool, Error> {
        match self
            .eeprom_read_byte(eeprom::Parameter::Parity)
            .await?
        {
            0xFF => Ok(false),
            0xE3 => Ok(true),
            _ => Err(Error::InvalidResponse),
        }
    }

    /// Set the destination for RF packets EEPROM setting
    /// 
    /// **Default:** 0x00
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_destination([0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE]).await;
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_set_destination(&mut self, value: [u8; 6]) -> Result<(), Error> {
        self.eeprom_write(eeprom::Parameter::DestinationId, &value)
            .await
    }

    /// Get the destination for RF packets EEPROM setting
    /// 
    /// **Default:** [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let destination = transceiver.eeprom_get_destination().await;
    /// println!("Destination: {:?}", destination);
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_destination(&mut self) -> Result<[u8; 6], Error> {
        self.eeprom_read::<6>(eeprom::Parameter::DestinationId)
            .await
    }

    /// Set the System ID EEPROM setting
    /// 
    /// A number from 0 to 256 that provides added security to each independent network
    /// of AC4490 units. The System ID is used in conjunction with the Channel Number and
    /// serves as an RF password to maintain secure transfers of data. The combination of
    /// the Channel Number and System ID must be unique to each network of AC4490s to
    /// establish communication. Multiple servers in the same coverage area must be
    /// programmed with different Channel Numbers to prevent inoperability of the
    /// networks. The System ID will not prevent inoperability that occurs from locating
    /// multiple servers with the same Channel Number in the same coverage area.
    /// 
    /// ***Note:** Separate Collocated AC4490 networks must operate on different Channel
    /// Networks. All units in a given AC4490 network must have identical Channel
    /// Numbers and System IDs.*
    /// 
    /// **Default:** 0x01
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_system_id(0x01).await;
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the value or response is invalid.
    pub async fn eeprom_set_system_id(&mut self, value: u8) -> Result<(), Error> {
        self.eeprom_write_byte(eeprom::Parameter::SystemId, value)
            .await
    }

    /// Get the System ID EEPROM setting
    /// 
    /// Similar to network password. Radios must have the same
    /// system ID to communicate with each other.
    /// 
    /// **Default:** 0x01
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let system_id = transceiver.eeprom_get_system_id().await;
    /// println!("System ID: {}", system_id);
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_system_id(&mut self) -> Result<u8, Error> {
        self.eeprom_read_byte(eeprom::Parameter::SystemId).await
    }

    /// Get the factory-programmed unique IEEE MAC address.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let mac_address = transceiver.eeprom_get_mac_id().await;
    /// println!("MAC Address: {:?}", mac_address);
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_mac_id(&mut self) -> Result<[u8; 6], Error> {
        self.eeprom_read::<6>(eeprom::Parameter::MacId)
            .await
    }

    /// Get the Original Max Power EEPROM value
    /// 
    /// Copy of max power EEPROM setting. This may be referenced but should not be modified.
    /// 
    /// **Default:** Set in production & can vary
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::{AC4490, OutputPower};
    /// let mut transceiver = AC4490::new(port);
    /// let original_max_power = transceiver.eeprom_get_original_max_power().await;
    /// println!("Original Max Power: {:?}", original_max_power);
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_original_max_power(&mut self) -> Result<OutputPower, Error> {
        self.eeprom_read_byte_type::<OutputPower>(eeprom::Parameter::OriginalMaxPower)
            .await
    }

    /// Get the Product ID EEPROM Value
    /// 
    /// This is a 15-byte identifier for the product.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let product_id = transceiver.eeprom_get_product_id().await;
    /// println!("Product ID: {}", product_id);
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_product_id(&mut self) -> Result<[u8; 15], Error> {
        self.eeprom_read::<15>(eeprom::Parameter::ProductId)
            .await
    }

    /// Enable / disable the Protocol Status / Receive ACK EEPROM setting
    /// 
    /// When enabled, GO0 outputs the Protocol Status, and GO1 outputs the Receive Acknowledgement signal.
    /// 
    /// **Default:** false
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_protocol_status_receive_ack_enable(true).await;
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_set_protocol_status_receive_ack_enable(&mut self, enable: bool) -> Result<(), Error> {
        self.eeprom_write_byte(
            eeprom::Parameter::ProtocolStatusReceiveAck,
            if enable { 0xE3 } else { 0xFF },
        )
        .await
    }

    /// Check whether the Protocol Status / Receive ACK EEPROM setting is enabled
    /// 
    /// When enabled, GO0 outputs the Protocol Status, and GO1 outputs the Receive Acknowledgement signal.
    /// 
    /// **Default:** false
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let enabled = transceiver.eeprom_get_protocol_status_receive_ack_enable().await;
    /// println!("Protocol Status / Receive ACK Enabled: {}", enabled);
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_protocol_status_receive_ack_enable(&mut self) -> Result<bool, Error> {
        match self
            .eeprom_read_byte(eeprom::Parameter::ProtocolStatusReceiveAck)
            .await?
        {
            0xFF => Ok(false),
            0xE3 => Ok(true),
            _ => Err(Error::InvalidResponse),
        }
    }

    /// Enable / disable the Receive API EEPROM setting
    /// 
    /// **Default:** false
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_receive_api_enable(true).await;
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_set_receive_api_enable(&mut self, enable: bool) -> Result<(), Error> {
        self.eeprom_write_byte(
            eeprom::Parameter::ReceiveApi,
            if enable { 0xE3 } else { 0xFF },
        )
        .await
    }

    /// Check whether the Receive API EEPROM setting is enabled
    /// 
    /// **Default:** false
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let enabled = transceiver.eeprom_get_receive_api_enable().await;
    /// println!("Receive API Enabled: {}", enabled);
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_receive_api_enable(&mut self) -> Result<bool, Error> {
        match self
            .eeprom_read_byte(eeprom::Parameter::ReceiveApi)
            .await?
        {
            0xFF => Ok(false),
            0xE3 => Ok(true),
            _ => Err(Error::InvalidResponse),
        }
    }

    /// Set the Enhanced API Control EEPROM setting
    /// 
    /// **Default:**
    /// ```
    /// EnhancedApiControl {
    ///     enhanced_api_control_enable: false,
    ///     send_data_complete_enable: false,
    ///     api_transmit_packet_enable: false,
    ///     enhanced_api_receive_packet_enable: false,
    /// }
    /// ```
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::{AC4490, EnhancedApiControl};
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_enhanced_api_control(EnhancedApiControl {
    ///     enhanced_api_control_enable: false,
    ///     send_data_complete_enable: false
    ///     api_transmit_packet_enable: false,
    ///     enhanced_api_receive_packet_enable: false,
    /// }).await;
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_set_enhanced_api_control(&mut self, value: EnhancedApiControl) -> Result<(), Error> {
        let current_byte = self.eeprom_read_byte(eeprom::Parameter::EnhancedApiControl).await?;

        // Do not modify "Laird Use Only" bits
        let new_byte = (current_byte & 0b0111_1100) | u8::from(value);

        self.eeprom_write_byte(eeprom::Parameter::EnhancedApiControl, new_byte)
            .await
    }

    /// Get the Enhanced API Control EEPROM setting
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let enhanced_api_control = transceiver.eeprom_get_enhanced_api_control().await;
    /// println!("Enhanced API Control: {:?}", enhanced_api_control);
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_enhanced_api_control(&mut self) -> Result<EnhancedApiControl, Error> {
        let byte = self.eeprom_read_byte(eeprom::Parameter::EnhancedApiControl).await?;
        Ok(EnhancedApiControl::from(byte))
    }

    /// Enable / disable Auto Calibrate
    /// 
    /// When enabled, Auto Calibrate causes the radio to measure the temperature every 30 to 60 seconds. If the
    /// temperature changes more than 30ºC from the last calibration, the radio will initiate a recalibration.
    /// 
    /// During the recalibration, the radio will not assert CTS high. Recalibration can take up to 3 seconds and the
    /// command response will not be sent to the OEM Host until recalibration is complete
    /// 
    /// **Default:** false
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_auto_calibrate_enable(true).await;
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_set_auto_calibrate_enable(&mut self, enable: bool) -> Result<(), Error> {
        self.eeprom_write_byte(
            eeprom::Parameter::AutoCalibrate,
            if enable { 0xE3 } else { 0xFF },
        )
        .await
    }

    /// Check whether Auto Calibrate is enabled
    /// 
    /// When enabled, Auto Calibrate causes the radio to measure the temperature every 30 to 60 seconds. If the
    /// temperature changes more than 30ºC from the last calibration, the radio will initiate a recalibration.
    /// 
    /// During the recalibration, the radio will not assert CTS high. Recalibration can take up to 3 seconds and the
    /// command response will not be sent to the OEM Host until recalibration is complete
    /// 
    /// **Default:** false
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let enabled = transceiver.eeprom_get_auto_calibrate_enable().await;
    /// println!("Auto Calibrate Enabled: {}", enabled);
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_auto_calibrate_enable(&mut self) -> Result<bool, Error> {
        match self
            .eeprom_read_byte(eeprom::Parameter::AutoCalibrate)
            .await?
        {
            0xFF => Ok(false),
            0xE3 => Ok(true),
            _ => Err(Error::InvalidResponse),
        }
    }

    /// Set the DES Key EEPROM setting
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// transceiver.eeprom_set_des_key([0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0x01]).await;
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_set_des_key(&mut self, value: [u8; 7]) -> Result<(), Error> {
        self.eeprom_write(eeprom::Parameter::DesKey, &value)
            .await
    }

    /// Get the DES Key EEPROM setting
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ac4490::AC4490;
    /// let mut transceiver = AC4490::new(port);
    /// let des_key = transceiver.eeprom_get_des_key().await;
    /// println!("DES Key: {:?}", des_key);
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns an error if read or write fails, or if the response is invalid.
    pub async fn eeprom_get_des_key(&mut self) -> Result<[u8; 7], Error> {
        self.eeprom_read::<7>(eeprom::Parameter::DesKey)
            .await
    }

}
