# AD7190 Driver for Rust Embedded

This is a driver for the Analog Devices AD7190 4-channel, 24-bit sigma-delta ADC,
written in Rust for embedded systems. It supports asynchronous operations using the
`embedded-hal-async` traits.

## Example Usage

I primarily built this to work with the [AD7190 strain bridge module](https://aliexpress.com/item/32329959263.html) available on AliExpress.

```rust,no_run
use ad7190::{
    Ad7190, Channels, Config as AdcConfig, ConfigFlags, ContinuousCfg, FilterSel, Gain, GpioPin,
    Mode, Reg,
};

// ADC reference voltage (as used in the AliExpress module)
const ADC_VREF_V: f32 = 5.0;

// AD7190 gain setting
const ADC_GAIN: Gain = Gain::G128;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
  let p = embassy_stm32::init(Default::default());

  let mut spi_config = Config::default();
  spi_config.mode = embassy_stm32::spi::MODE_3;
  spi_config.frequency = Hertz(1_000_000);

  let spi = Spi::new(
      p.SPI1,
      p.PA5,       // SCK
      p.PA7,       // MOSI
      p.PA6,       // MISO
      p.DMA2_CH3,  // TX DMA
      p.DMA2_CH2,  // RX DMA
      spi_config,
  );
  let spi_bus = Mutex::<CriticalSectionRawMutex, _>::new(spi);

  let cs = Output::new(p.PA4, Level::High, Speed::VeryHigh);
  let spi_dev = SpiDevice::new(&spi_bus, cs);

  let mut adc: Adc = Ad7190::new(spi_dev, None);

  // Reset ADC
  let mut delay = Delay;
  adc.reset(&mut delay).await.unwrap();

  // Turn on excitation source
  adc.gpio_set_high(GpioPin::P3).await.unwrap();

  // Configure channels + gain + buffer, bipolar mode
  let mut cfg = AdcConfig {
      flags: ConfigFlags::BUF,
      channels: Channels::CH1,
      gain: ADC_GAIN,
  };
  cfg = cfg.with_unipolar(false);
  adc.set_config(cfg).await.unwrap();

  // Set continuous conversion mode on channel 1
  let mode = Mode::default()
    .with_dat_sta(true)
    .with_filter(FilterSel::Sinc4)
    .with_sample_rate_hz(50.0);

  let cont_cfg = ContinuousCfg {
      dat_sta: true,
      use_pin_if_available: false,
      use_cread: false,
      poll_max_tries: 50_000,
      poll_delay_us: 10,
  };

  // Start continuous conversion
  let cont = adc.start_continuous(mode, cont_cfg).await.unwrap();  

  // Main loop
  loop {
    let sample = adc.next_sample(&mut delay, &cont).await.unwrap();

    let v = Adc::code_to_volts_bipolar(sample.code, ADC_VREF_V, ADC_GAIN);

    info!("Sample: code={} voltage={:.6} V", sample.code, v);
  }
}
``