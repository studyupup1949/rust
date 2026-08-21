# adb-wireless

Generate a QR code in terminal for adb wireless pair

### Prerequisites

* Ensure you have `adb` installed and available in your PATH.
* Ensure your Android device has USB debugging enabled.
* Ensure your Android device is connected to the same network as your computer.

### Installation

```
cargo install adb-wireless
```

### Usage

```
adb-wireless pair
```

### Commands

```
>>> adb-wireless -h
CLI tool to generate QR code for adb wireless connection

Usage: adb-wireless <COMMAND>

Commands:
  pair     Generate QR code for adb wireless connection
  reverse  Map TCP ports from device to host
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

```
>>> adb-wireless pair -h
Generate QR code for adb wireless connection

Usage: adb-wireless pair

Options:
  -h, --help  Print help
```

```
>>> adb-wireless reverse -h
Map TCP ports from device to host

Usage: adb-wireless reverse <PORT:PORT>...

Arguments:
  <PORT:PORT>...  Port mappings in the format <device_port>:<host_port>

Options:
  -h, --help  Print help
```