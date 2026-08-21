================================================================================
                         A608 Embedded Fingerprint Sensor
                            no_std Rust Library
================================================================================

A no_std compatible Rust library for interfacing with fingerprint sensors
(R503, R307, ZFM-20, etc.) on embedded systems using UART communication.

================================================================================
FEATURES
================================================================================

- Full no_std support for embedded systems
- Zero heap allocation - uses fixed-size buffers
- Generic over any UART implementation using embedded-io traits
- Complete fingerprint sensor protocol implementation
- Support for R503 LED control

================================================================================
SUPPORTED SENSORS
================================================================================

- R503 (with LED control)
- R307
- ZFM-20
- AS608
- Other compatible sensors using the same UART protocol

================================================================================
DEPENDENCIES
================================================================================

Add to your Cargo.toml:

    [dependencies]
    a608_embedded = "0.1.0"
    embedded-hal = "1.0.0"
    embedded-io = "0.7.1"

================================================================================
QUICK START
================================================================================

1. Create a UART instance for your platform
2. Create a FingerprintSensor with the UART and password
3. Call init() to verify connection and read system parameters
4. Use the sensor methods to capture and match fingerprints

Basic usage pattern:

    use a608_embedded::{FingerprintSensor, OK, NOFINGER};

    // Create sensor with default password [0, 0, 0, 0]
    let mut sensor = FingerprintSensor::new(uart, [0, 0, 0, 0]);

    // Initialize sensor
    sensor.init().unwrap();

    // Capture fingerprint
    loop {
        if sensor.get_image().unwrap() == OK {
            break;
        }
    }

    // Convert to template
    sensor.image_2_tz(1).unwrap();

    // Search in database
    if sensor.finger_fast_search().unwrap() == OK {
        println!("Found finger ID: {}", sensor.finger_id);
    }

================================================================================
API REFERENCE
================================================================================

INITIALIZATION
--------------
new(uart, password)           Create sensor with default address
new_with_address(uart, addr, password)  Create sensor with custom address
init()                        Initialize and verify connection
release()                     Release UART ownership

SYSTEM COMMANDS
---------------
check_module()                Check if sensor is responding
verify_password()             Verify password with sensor
read_sysparam()               Read system parameters
set_sysparam(param, value)    Set system parameter
soft_reset()                  Perform soft reset

FINGERPRINT CAPTURE
-------------------
get_image()                   Capture fingerprint image
image_2_tz(slot)              Convert image to template (slot 1 or 2)

TEMPLATE MANAGEMENT
-------------------
create_model()                Create model from templates in slots 1 & 2
store_model(location, slot)   Store template to flash memory
load_model(location, slot)    Load template from flash to slot
delete_model(location)        Delete template from flash
empty_library()               Delete all templates
count_templates()             Count stored templates
read_template_page(page, bitmap)  Read template bitmap for page

FINGERPRINT MATCHING
--------------------
finger_search()               Standard fingerprint search
finger_fast_search()          High-speed fingerprint search
compare_templates()           Compare templates in slots 1 & 2

DATA TRANSFER
-------------
get_fpdata(buffer, out)       Download fingerprint data from sensor
send_fpdata(data, buffer)     Upload fingerprint data to sensor

LED CONTROL (R503 only)
-----------------------
set_led(color, mode, speed, cycles)   Control LED state

PROPERTIES
----------
finger_id                     Last matched finger ID
confidence                    Last match confidence score
template_count                Number of stored templates
library_size()                Maximum library capacity
data_packet_size()            Current data packet size setting
system_parameters()           Full system parameters

================================================================================
ERROR CODES
================================================================================

OK (0x00)                     Success
PACKETRECIEVEERR (0x01)       Packet receive error
NOFINGER (0x02)               No finger detected
IMAGEFAIL (0x03)              Failed to capture image
IMAGEMESS (0x06)              Image too messy
FEATUREFAIL (0x07)            Failed to generate features
NOMATCH (0x08)                Fingerprints don't match
NOTFOUND (0x09)               No matching fingerprint found
ENROLLMISMATCH (0x0A)         Enrollment images don't match
BADLOCATION (0x0B)            Invalid storage location
PASSFAIL (0x13)               Password verification failed

================================================================================
EXAMPLES
================================================================================

See the examples/ directory for complete examples:

- examples/esp32_example.rs   - ESP32 using esp-hal
- examples/stm32_example.rs   - STM32 using stm32-hal2

================================================================================
WIRING
================================================================================

Sensor Pin      MCU Pin
----------      -------
VCC (Red)       3.3V or 5V (check sensor specs)
GND (Black)     GND
TX (Yellow)     UART RX
RX (Green)      UART TX
IRQ (Blue)      Optional GPIO input (touch detection)
3.3V (White)    3.3V (for R503 LED power, optional)

Default UART settings:
- Baud rate: 57600
- Data bits: 8
- Stop bits: 1
- Parity: None

================================================================================
LICENSE
================================================================================

MIT License

Copyright (c) 2024

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

================================================================================
