[![crates.io](https://img.shields.io/crates/v/access-mask)](https://crates.io/crates/access-mask)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue)](LICENSE-MIT)

# AccessMask
AccessMask is a command line tool that interprets handle access mask values (both generic and specific).

**It is directly taken from the awesome tools from the one and only (the GOAT) [Pavel Yosifovich](https://github.com/zodiacon/AccessMask).**

This project is more or less a one-to-one rust translation of his tools but rust powered.

My primary goal is to learn more about windows internals by reading ~~God~~ Pavel's  code, and then to make it available through `cargo install`.

## Usage
```text
Usage: access-mask.exe [OPTIONS] <VALUE>

Arguments:
<VALUE>  value is interpreted as hexadecimal, unless the -d switch is specified. Specific access mask bits will not be interpreted if type is not specified

Options:
-d, --decimal      interpret value as a decimal value
-t, --type <TYPE>  Specify a type for the access mask [possible values: process, thread, event, mutex, mutant, semaphore, timer, desktop, windows-station, winsta, key, token, job, file, directory, alpc, port, active-directory]
-h, --help         Print help
-V, --version      Print version
```
