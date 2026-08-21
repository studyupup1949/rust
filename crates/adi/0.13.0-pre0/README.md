[![Plop Grizzly](https://plopgrizzly.com/images/logo-bar.png)](https://plopgrizzly.com)

# [Aldaron's Device Interface](https://crates.io/crates/adi)
Lightweight platform-agnostic device interface abstraction layer for
creating apps and video games.

Aldaron's Device Interface aims to be the new [SDL](https://www.libsdl.org/)
in some aspects, but isn't specifically targeting video games.  This library
also aims to replace cross-platform GUI toolkits like
[GTK](https://www.gtk.org/).  Ultimately, though, it's cooler than both
combined, and way smaller!  Since those libraries are 3-letter acronyms,
this library must also be: [ADI](https://aldaron.tk/crates/)!

## Getting Started
Rather than having a bunch of separately maintained projects like SDL,
everything is together in ADI.  By default, everything is built (all of the
device interfaces).  This usually is not what you want, so this is how you
should specify ADI in your `Cargo.toml` if you only want the `screen` API:
```TOML
[dependencies.adi]
version = "0.13"
default-features = false
features = ["screen"]
path = "../adi"
```
Thanks to Cargo, that's a lot easier than finding all of the SDL libraries!
Conveniently, the features have the same name as the modules in this
documentation, to make it even easier!

## List of Modules / Features
| Module / Feature | More Info                                                 |
|------------------|-----------------------------------------------------------|
| `mic`            | Microphone support on Linux (Alsa).  TODO: Android, Windows, MacOS / iOS, Nintendo Switch, Web Assembly |
| `speaker`        | Speaker support on Linux (Alsa).  TODO: Android, Windows, MacOS / iOS, Nintendo Switch, Web Assembly |
| `screen`         | Screen support on Linux and Windows.  Can switch between OpenGL, OpenGLES or Vulkan depending on what's available.  TODO: Vulkan + Android, Vulkan + DirectFB on Raspberry Pi, Metal or MoltenVk on MacOS & iOS, Vulkan on the Nintendo Switch, WebGL on Web Assembly |
| `hid`            | Human interface device, which may also have haptic feedback (vibrate).  TODO: Separate from Screen, Implement missing joystick & touchpad support on Windows, missing touch |
| `net`            | Client / Server Wi-Fi & Ethernet stuff (even easier, simpler than `std::net`). |
| `drive`          | Application / Drive Storage (more secure than `std::fs`). TODO: Find storage locations for each platform, API for Reading / Writing CD & DVD |
| `usb`            | Low level USB interface.  Send and recieve packets over USB between a Computer and an Arduino, Tablet, or Phone. |
| `bluetooth`      | Send and receive packets over bluetooth. |
| `cam`            | Record Video from a Webcam or other Camera. |
| `gps`            | Geographic position locator through GPS, WiFi, Cell Towers, or a combination. |
| `sensor`         | Gyroscope, accelerometer, distance sensor, etc. |
| `gpio`           | GPIO (General Purpose Input/Output) for electronics.  Raspberry Pi and Arduino are good candidates for using this feature. |

## [Contributing](http://plopgrizzly.com/contributing/en#contributing)
This crate is made up of several modules, which are crates in themselves.  If
you'd like to contribute, look below at the roadmap, or the individual crate
roadmaps.  Check the link above for more details.

## Until The 1.0 Release
Once all the roadmaps have been completed, unless noted otherwise then 1.0 will
be released.

## Change Log
### 0.13
* Everything is one crate now.
* Split `audio` into `mic` and `speaker`.
* All APIs have been updated.
* Is now formatted using `cargo fmt` through `rustup component add rustfmt-preview`.

### 0.12
* Added `audio` module for supporting Audio Interface (Audio Input/Output - Speaker/Microphone).

### 0.11
* Update adi\_screen to 0.12
* Made `screen` feature for enabling/disabling `adi_screen` dependency / `screen` module.
