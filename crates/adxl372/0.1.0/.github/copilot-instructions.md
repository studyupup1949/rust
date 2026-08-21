# GitHub Copilot Instructions for this Workspace

## Project context
This workspace contains a `no_std` Rust driver for the Analog Devices **ADXL372** high-g accelerometer.

The driver must:
- Run on microcontrollers (ARM Cortex-M, RISC-V, ESP32C3, NRF, STM32…)
- Use **embedded-hal** traits for bus access (SPI now, I2C in the future)
- Have **no dynamic allocation** in library code
- Prefer static, compile-time safety over runtime checks
- Cleanly implement silicon errata and required workarounds

The architecture is modular:
- `device.rs` → high-level driver (`Adxl372<IFACE>`)
- `interface/` → low-level bus access (SPI now, I2C later)
- `registers.rs` → register addresses + bitfield helpers
- `config.rs` → `Config`, validation logic and typed parameters
- `fifo.rs` → FIFO decoding + errata handling
- `self_test.rs` → complex self-test logic (errata er001)
- `params.rs` → enums for ODR, Bandwidth, PowerMode, FIFO formats, etc.

## Coding style
- All public functions must use `Result<T, Error<E>>` rather than panicking.
- Avoid `.unwrap()`, `.expect()`, panics or `unreachable!()` in library code.
- Use explicit types and avoid inference for public API boundaries.
- Prefer small, composable functions with clear responsibilities.
- Use Rust doc comments (`///`) for every public item.
- Avoid unnecessary generics; use generics only when they improve clarity or safety.
- Prefer `enum` over untyped integers for configuration.
- Expose only high-level APIs to users; hide register detail behind helpers.

## no_std constraints
- `std` must NOT be used.
- No heap and no global allocators in the core driver.
- No unbounded blocking waits.
- The driver must not allocate large internal buffers; callers provide buffers via `&mut [T]`.
- Small, fixed-size stack buffers are allowed internally, but must remain modest and constant-size.
- No mandatory logging or RTT dependencies in the core library.

## Async support
- The primary implementation is **synchronous** over `embedded-hal` traits.
- Async support, if generated, must be:
  - Based on `embedded-hal-async` traits.
  - Guarded behind a **feature flag** (e.g. `async` or `embedded-hal-async`).
  - Implemented as thin wrappers around the sync core where possible.
- Copilot should NOT introduce external async runtimes (Tokio, async-std, Embassy) in library code.
- Async examples are allowed only when placed behind the proper feature flags.

## Interface design
- All hardware access must go through the `Adxl372Interface` trait.
- `SpiInterface` implements the trait now.
- `I2cInterface` will implement the same trait in the future.
- The high-level driver (`Adxl372<IFACE>`) must not depend on SPI- or I2C-specific logic.

## Register access patterns
When generating register helpers:
- Prefer explicit bit masks over magic numbers.
- Group bitfields by register (e.g., `power_ctl`, `measure`, `timing`).
- Preserve unrelated bits when modifying registers.
- Use small helper functions for read/modify/write.

## Configuration logic
When generating or modifying `Config`:
- Always implement corresponding validation in `Config::validate`.
- Validation errors should use strongly-typed error variants.
- Enums must reflect actual datasheet constraints (ODR, BW, HPF, modes…).
- Ensure Nyquist rules: `BW <= ODR/2`.
- Encode datasheet restrictions at type level whenever possible.

## Buffer management (FIFO, no alloc)
FIFO access must be designed to work **without allocators**:

- The core FIFO APIs must operate on **caller-provided buffers**, not on `Vec` or heap-based containers.
- Typical signatures should look like:
  - `read_fifo_raw(&mut self, buf: &mut [u8]) -> Result<usize, Error<E>>`
  - `read_fifo_samples(&mut self, samples: &mut [Sample]) -> Result<usize, Error<E>>`
- These functions:
  - Fill the provided slice up to its capacity.
  - Return the number of bytes/samples actually written.
  - May be called repeatedly to fully drain the hardware FIFO.
- Internal stack buffers (e.g. small `[u8; N]`) may be used as scratch space, but must stay small and fixed-size.
- The core driver must NOT create or manage dynamic containers:
  - No `Vec`, `String` or similar in the main FIFO path.
  - No dynamic growth of internal buffers.

Optional convenience layers that use `heapless` are allowed **only behind feature flags** and must be thin wrappers around the slice-based APIs.

## FIFO logic
The driver is responsible for interpreting FIFO data according to the current configuration.

When generating FIFO-related code:
- The **primary** FIFO API should return a typed `Sample` struct, not raw bytes.
- `Sample` must reflect enabled axes and formats. Recommended layout:

  ```rust
  pub struct Sample {
      pub x: Option<i16>,
      pub y: Option<i16>,
      pub z: Option<i16>,
      pub is_peak: bool,
  }
  ```

- The driver must:
  - Derive expected layouts from `FifoFormat` and sensor configuration.
  - Decode raw FIFO bytes into typed `Sample` values.
  - Correctly handle sample alignment and the "series start" bit.
  - Hide all FIFO layout complexity from the user.

- Slice-based APIs must be preferred, for example:

  ```rust
  pub fn read_fifo_raw(
      &mut self,
      buf: &mut [u8],
  ) -> Result<usize, Error<E>>;

  pub fn read_fifo_samples(
      &mut self,
      samples: &mut [Sample],
  ) -> Result<usize, Error<E>>;
  ```

- These functions must not allocate; callers decide buffer size and memory placement.
- A **secondary** raw API (`read_fifo_raw`) is allowed for advanced users, but the `Sample`-based API is the recommended default.
- FIFO parsing must be side-effect-free and deterministic.
- If adding new FIFO formats, update layout tables/mappings.
- Implement errata workarounds in clearly-named internal functions (e.g. `read_fifo_samples_with_workaround`).

## Self-test logic
Self-test logic is intentionally isolated in `self_test.rs`.

Copilot must:
- Follow the steps and required delays of the official silicon errata.
- Not optimize out sample windows or timing requirements.
- Keep computations integer-based when possible.
- Restore previous configuration after the self-test.
- Provide a simple API in `device.rs` that delegates to `self_test.rs`.

## Logging and debugging
- The driver must NOT include mandatory logging.
- **Optional `defmt` support is allowed and encouraged behind a feature flag.**
- When generating logging code:
  - Use `#[cfg(feature = "defmt")]` around all debug/trace logs.
  - Prefer `defmt::trace!` or `defmt::debug!` for internal debugging (FIFO, self-test, register state).
  - Logging must NEVER affect control flow or timing.
  - Logging must NOT appear in public APIs or be required for normal operation.

Example:

```rust
#[cfg(feature = "defmt")]
defmt::trace!("fifo raw bytes: {:?}", &buf[..count]);
```

## API expectations
When Copilot generates new APIs:
- Prefer clear names like `read_status`, `set_timing`, `set_power_ctl`, `read_fifo_samples`.
- APIs should express *intent* rather than register-level operations.
- Public APIs must produce safe, typed data (e.g. `Sample`) that matches the sensor configuration.
- Avoid generating low-level register APIs unless absolutely required.

## What NOT to generate
Copilot should avoid:
- Async runtimes such as Tokio or async-std.
- Unnecessary trait abstractions or multi-layered generics.
- Global allocators or heap structures.
- `std` collections (`Vec`, `String`, `HashMap`, etc.) in the core driver.
- Overly complex macros or hidden state machines.
- Lifetime-heavy abstractions that reduce readability.

## Tests & Examples
Copilot should follow test expectations strictly.

Test guidelines:
- Validation tests must reflect the rules defined in `Config::validate`.
- FIFO tests must cover axis combinations (X, Y, Z, XY, XZ, YZ, XYZ, peak).
- Tests should check correct `Sample` decoding from raw FIFO bytes.
- Tests for FIFO must also verify that the slice-based APIs respect capacities and returned lengths.
- Prefer deterministic tests without randomness or unstable timing.

Examples:
- Show recommended usage through `Adxl372::new_spi`, `init`, `read_xyz_raw`, and FIFO iteration.
- Examples involving async or `defmt` must be guarded behind their respective feature flags.

###

In summary, this driver must be:
- clear, typed, deterministic and aligned with the ADXL372 datasheet,
- portable and `no_std`,
- sync-first but async-ready,
- logging-free by default but debuggable via optional `defmt`,
- and strictly non-allocating in its core APIs, especially for FIFO handling.

Copilot should always generate code that follows these constraints and preserves correctness.