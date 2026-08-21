# Aldaron's Device Interface (adi 0.5.0)

[Aldaron's Device Interface (adi)](http://plopgrizzly.com/adi)
is a library developed by [Plop Grizzly](http://plopgrizzly.com) for
creating cross-platform applications and video games.  It's goal is to be a
powerful, easy-to-use, and minimal SDL-esque library.

[Cargo](https://crates.io/crates/adi) /
[Documentation](https://docs.rs/adi)

## Features
**adi**'s current features:
* Clock interface (missing time and date)
* Screen + Input interface (missing joystick & touchpad on Windows, missing touch)

**adi**'s planned features:
* Missing features listed above
* Speaker interface (Audio Output)
* Network interface (Send and receive packets over Ethernet or Wifi) 
* Storage interface (Hard drive, Solid state drive, USB drive, CD/DVD, and SD card)
* USB interface (Send and recieve packets over USB between a Computer and an Arduino, Tablet, or Phone)
* Bluetooth interface (Send and receive packets over bluetooth.)
* Microphone interface (Record Audio from a Microphone)
* Camera interface (Record Video from a Webcam or other Camera)
* Haptic interface (Cause vibrations in a Joystick, Phone or Other controller)

## Support
**adi**'s current support:
* Vulkan + XCB + Linux
* Vulkan + Windows

**adi**'s planned support:
* Vulkan + Wayland + Linux
* Vulkan + Android
* Vulkan + Direct to Display + Raspberry Pi
* Metal + MacOS
* Metal + IOS
* Vulkan + Nintendo Switch
* WebGL + Web Assembly
* OpenGL + Windows
* OpenGL + MacOS
* No OS on Arm
* No OS on X86
* Arduino
* Aldaros ( a planned OS written using No OS on Arm/X86 )
* Vulkan + Direct to Display + Linux w/o Desktop Environment

# Contributing

If you'd like to help implement functions for unsupported platforms, fix bugs,
improve the API or improve the Documentation, then contact me at
jeron.lau@plopgrizzly.com. I'll appreciate any help.
