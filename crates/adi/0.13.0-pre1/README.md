# Aldaron's Device Interface
The lightweight platform-agnostic device interface abstraction layer for
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
```toml
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

## TODO
Once all of the modules / features listed above exist in the crate, and no longer have TODO written next to them, version 1.0 will be released.

## Links
* [Website](https://free.plopgrizzly.com/adi)
* [Cargo](https://crates.io/crates/adi)
* [Documentation](https://docs.rs/adi)
* [Change Log](https://free.plopgrizzly.com/adi/changelog)
* [Contributing](https://plopgrizzly.com/contributing)
* [Code of Conduct](https://free.plopgrizzly.com/adi/codeofconduct)

---

[![Plop Grizzly](https://plopgrizzly.com/images/logo-bar.png)](https://plopgrizzly.com)