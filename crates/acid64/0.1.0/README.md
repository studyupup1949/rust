# acid64

A simple Rust CLI tool to decode Base64 and Base64URL-encoded strings.
Supports input from command-line arguments, files, or stdin.

## Features

- Decode standard Base64 or Base64URL
- Accept input from:
    - Command-line argument
    - File (`-f`)
    - Stdin (default fallback)
- Optional flag to ignore padding (`-p`)
- Output decoded result to stdout
- Graceful error handling

## Usage


# Decode standard Base64 from CLI
```shell
acid64 "SGVsbG8gd29ybGQ="
```

# Decode Base64URL from CLI
```shell
acid64 -u "YWJjZGVmZ2hpamtsbW5vcHFyc3R1dnd4eXotLS0_"
```

# Decode from file
```shell
acid64 -f input.txt
```

# Decode from stdin
```shell
echo "SGVsbG8gd29ybGQ=" | acid64
```

# Decode Base64URL from file, ignoring padding
```shell
acid64 -p "SGVsbG8td29ybGQ"
```
