![Aldaron's Device Interface](/docs/adi.png)

# Aldaron's Device Interface 0.3.0
Aldaron's Device Interface, or ADI is a rust library for creating cross-platform
applications and video games.  It's goal is to be an powerful, easy-to-use, and
minimal SDL-esque library.  It is developed by Plop Grizzly.

# Documentation
The documentation for ADI can be found at http://plopgrizzly.tech/adi/.

# Supported Platforms
These are the platforms that ADI currently supports.

1. Vulkan + XCB + Linux
2. Vulkan + Windows

These are some planned platforms.

1. Vulkan + Wayland + Linux
2. Vulkan + Android
3. Vulkan + Direct to Display + Raspberry Pi
4. Metal + MacOS
5. Metal + IOS
6. Vulkan + Nintendo Switch
7. WebGL + Web Assembly
8. OpenGL + Windows
9. OpenGL + MacOS
10. No OS on Arm
11. No OS on X86
12. Arduino
13. Aldaros ( a planned OS written using No OS on Arm or X86 )
14. Vulkan + Direct to Display + Linux w/o Desktop Environment

If you'd like to help port the library to any of these platforms or others,
contact me at jeron.lau@plopgrizzly.tech.  I'll appreciate any help.

# Devices
These are the current supported devices for each supported platform

| Platform              | Clock     | Screen ( Input ) | Speakers | Network | Storage | USB    | Bluetooth | Microphone | Camera | Haptic |
|-----------------------|-----------|------------------|----------|---------|---------|--------|-----------|------------|--------|--------|
| Vulkan + XCB on Linux | Supported | Supported        | N.Y.I.   | N.Y.I.  | N.Y.I.  | N.Y.I. | N.Y.I     | N.Y.I      | N.Y.I  | N.Y.I  |
| Vulkan on Windows     | Supported | Supported        | N.Y.I.   | N.Y.I.  | N.Y.I.  | N.Y.I. | N.Y.I     | N.Y.I      | N.Y.I  | N.Y.I  |

## adi_clock 0.2.0
The CPU's clock and timer devices.

| Platform | Animate | Stopwatch | Sleep | Schedule Timer | Tell Time | Tell Date |
|----------|---------|-----------|-------|----------------|-----------|-----------|
| Linux    | Yes     | Yes       | Yes   | Yes            | No        | No        |
| Windows  | Yes     | Yes       | Yes   | Yes            | No        | No        |


## adi_screen 0.2.0
Video display, accessed through a window.  Because it's through a window, this
module takes care of input, too.

| Platform              | Window | Touch | Touchpad | Mouse | Keyboard | Joystick |
|-----------------------|--------|-------|----------|-------|----------|----------|
| Vulkan + XCB on Linux | Yes    | NYI   | Yes      | Yes   | Physical | Yes      |
| Vulkan on Windows     | Yes    | NYI   | NYI      | Yes   | Physical | NYI      |

## Speakers
Audio Output

## Network
Send and receive packets over Ethernet or Wifi.

## Storage
Hard drive, Solid state drive, USB drive, CD/DVD, and SD card

## USB
Send and recieve packets over USB between a Computer and an Arduino, Tablet, or
Phone.

## Bluetooth
Send and receive packets over bluetooth.

## Microphone
Record Audio from a Microphone.

## Camera
Record Video from a Webcam or other Camera.

## Haptic
Cause vibrations in a Joystick, Phone or Other controller.

# Contributing

If you'd like to help implement any of these unsupported devices for specific
platforms, contact me at jeron.lau@plopgrizzly.tech.  I'll appreciate any help.
