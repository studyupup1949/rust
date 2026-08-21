![Aldaron's Device Interface](/docs/adi.png)

# Aldaron's Device Interface 0.1.0
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

If you'd like to help port the library to any of these platforms or others,
contact me at jeron.lau@plopgrizzly.tech.  I'll appreciate any help.

# Devices
These are the current supported devices for each supported platform

| Platform              | Screen    | Input                    | Speakers | Network | Storage | USB    | Bluetooth | Microphone | Camera | Haptic |
|-----------------------|-----------|------------------|----------|---------|---------|--------|-----------|------------|--------|--------|
| Vulkan + XCB on Linux | Supported | Missing Joystick | N.Y.I.   | N.Y.I.  | N.Y.I.  | N.Y.I. | N.Y.I     | N.Y.I      | N.Y.I  | N.Y.I  |
| Vulkan on Windows     | Supported | Missing Joystick | N.Y.I.   | N.Y.I.  | N.Y.I.  | N.Y.I. | N.Y.I     | N.Y.I      | N.Y.I  | N.Y.I  |

## Screen
Video display, often though a window.

## Input
On screen or physical keyboard, Mouse and/or touch, and Virtual or physical
joystick

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
