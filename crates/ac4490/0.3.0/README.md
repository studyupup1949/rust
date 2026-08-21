# ac4490-rs

This is a driver for Ezurio's (previously Laird, previously Aerocomm) AC4490 transceiver.
The driver is interface-agnostic and so can be used with embedded or desktop applications.

View the docs [here](https://docs.rs/ac4490).

## Quick start

For a device using `embassy_stm32::usart::Uart`:

```rust
struct MyDeviceInterface(Uart<'static, embassy_stm32::mode::Async>);

impl DeviceInterface for MyDeviceInterface {
    async fn write(&mut self, data: &[u8]) -> Result<(), ac4490::Error> {
        self.0.write(data).await.map_err(|_| ac4490::Error::WriteError)
    }

    async fn read(&mut self, data: &mut [u8]) -> Result<(), ac4490::Error> {
        self.0.read(data).await.map_err(|_| ac4490::Error::ReadError)
    }
}

// ...

    let usart6 = usart::Uart::new(
        p.USART6,
        p.PC7,
        p.PC6,
        Irqs,
        p.GPDMA1_CH0,
        p.GPDMA1_CH1,
        uart_config,
    )
    .expect("Failed to initialize USART6");

    // Init AC4490
    let mut ac4490 = AC4490::new(
        Ac4490UartInterface(usart6)
    );

```

For a device using `serialport::SerialPort`:

```rust
use ac4490::DeviceInterface;
use serialport::SerialPort;

struct SerialPortDeviceInterface(Box<dyn serialport::SerialPort>);

impl DeviceInterface for SerialPortDeviceInterface {
    async fn write(&mut self, data: &[u8]) -> std::result::Result<(), ac4490::Error> {
        self.0.write_all(data).map_err(|_| ac4490::Error::WriteError)?;
        Ok(())
    }

    async fn read(&mut self, buf: &mut [u8]) -> std::result::Result<(), ac4490::Error> {
        self.0.read(buf).map_err(|_| ac4490::Error::ReadError)?;
        Ok(())
    }
}

async fn main() -> Result<()> {
    let port = serialport::new("/dev/ttyUSB0", 9600).open()?;
    let mut transceiver = AC4490::new(SerialPortDeviceInterface(port));
    Ok(())
} 

```