# A8mini-camera-rs

This project is a Rust port of A8mini-gimbal-camera-control (C version), which was originally created by thiagolages. 
During the porting process, certain architectural changes were made to better fit the Rust programming language.

## Setup

- Default IP is `192.168.144.25`
- Default port is `37260`

### List of currently supported simple (hardcoded) commands:

- AutoCenter
- RotateUp
- RotateDown
- RotateRight
- RotateLeft
- StopRotation
- ZoomIn
- ZoomOut
- ZoomMax
- MaxZoomInformation
- FocusIn
- FocusOut
- TakePicture
- RecordVideo
- Rotate100100
- CameraInformation
- AutoFocus
- HardwareIDInformation
- FirmwareVersionInformation
- SetLockMode
- SetFollowMode
- SetFPVMode
- AttitudeInformation
- SetVideoOutputHDMI
- SetVideoOutputCVBS
- SetVideoOutputOff
- LaserRangefinderInformation
- RebootCamera
- RebootGimbal

### List of currently supported complex commands:

- SetYawPitchSpeed(i8, i8)
- SetYawPitchAngle(i16, i16)

**Note**: More commands might be supported by the camera but may not be included in the list of implemented commands.

**Disclamer**: SIYI does provide some sample code which was used to build this code.
