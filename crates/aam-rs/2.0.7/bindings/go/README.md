# aam-go

CGo bindings for the [aam-rs](https://github.com/INiNiDS/aam-rs) AAM parser.

## Prerequisites

Build the Rust library first (from the repository root):

```sh
cargo build --release --features ffi
```

This produces `target/release/libaam_rs.so` (Linux), `libaam_rs.dylib` (macOS),
or `aam_rs.dll` (Windows) as well as the static archive `libaam_rs.a`.

The CGo flags in `aam/aam.go` default to `../../target/release` (relative to
the package source), so building from the repository root works out of the box.
Override via environment variables if you install the library elsewhere:

```sh
export CGO_CFLAGS="-I/usr/local/include"
export CGO_LDFLAGS="-L/usr/local/lib -laam_rs -ldl -lpthread -lm"
```

## Installation

```sh
go get github.com/INiNiDS/aam-rs/go/aam
```

## Quick start

```go
package main

import (
    "fmt"
    "log"

    "github.com/INiNiDS/aam-rs/go/aam"
)

func main() {
    doc, err := aam.Parse("host = localhost\nport = 8080\n")
    if err != nil {
        log.Fatal(err)
    }
    defer doc.Close()

    if val, ok := doc.Get("host"); ok {
        fmt.Println("host:", val) // host: localhost
    }

    fmt.Println("reverse:", doc.ReverseSearch("8080"))
}
```

## More examples

```go
doc, err := aam.Parse("root_path = srv\nactive_path = root_path\nmode = active\n")
if err != nil {
    panic(err)
}
defer doc.Close()

fmt.Println(doc.DeepSearch("path"))
fmt.Println(doc.Find("active"))
```

## API

| Function / Method                                     | Description                             |
|-------------------------------------------------------|-----------------------------------------|
| `New() (*AAM, error)`                                 | Creates an empty AAM handle             |
| `Parse(content string) (*AAM, error)`                 | Parses AAM content from a string        |
| `Load(path string) (*AAM, error)`                     | Loads and parses a `.aam` file          |
| `(*AAM) Format(content string) (string, error)`       | Formats arbitrary AAM text              |
| `(*AAM) Get(key string) (string, bool)`               | Direct key lookup                       |
| `(*AAM) Find(query string) map[string]string`         | Key lookup with value fallback          |
| `(*AAM) DeepSearch(pattern string) map[string]string` | Pattern search by key                   |
| `(*AAM) ReverseSearch(value string) []string`         | Reverse lookup (value -> matching keys) |
| `(*AAM) SchemaNames() []string`                       | Returns schema names                    |
| `(*AAM) TypeNames() []string`                         | Returns type names                      |
| `(*AAM) LastError() string`                           | Returns last native error string        |
| `(*AAM) Close()`                                      | Frees the native handle (idempotent)    |

## Running tests

```sh
# From repository root
cargo build --release --features ffi

# Then run Go tests
cd go
go test -v ./...
```

The test suite includes parse/load, lookup behavior, deep-search by pattern, reverse search, and closed-handle
behavior.

