# aam-rs for C#

C# bindings for the `aam-rs` AAM parser using the stable C FFI.

## Features

- **AAM Parsing**: Parse and query `.aam` configuration files
- **Cross-platform**: Supports Windows, macOS, and Linux (x64, ARM64)
- **Type-safe**: Strongly-typed C# API with proper resource management
- **Zero-copy**: Efficient bindings using P/Invoke
- **Schema Validation**: Built-in schema and type system support

## Installation

### Via NuGet

```bash
dotnet add package aam-csharp
```

Or via Package Manager:

```powershell
Install-Package aam-csharp
```

## Prerequisites

The managed package requires the native `aam_rs` library at runtime:

- Windows: `aam_rs.dll`
- macOS: `libaam_rs.dylib`
- Linux: `libaam_rs.so`

The NuGet package includes pre-built native libraries for:

- Windows (x64)
- macOS (x64, ARM64)
- Linux (x64)

### For local development

Build the native library from the repository:

```bash
cd aam-rs/
cargo build --release --features ffi
```

Then set the library path:

- **Linux/macOS**: `export LD_LIBRARY_PATH=/path/to/target/release:$LD_LIBRARY_PATH`
- **Windows**: Add the directory to your `PATH`

## Usage

### Basic Parsing

```csharp
using AamCsharp;

// Parse AAM content
using var doc = AamDocument.Parse("host = localhost\nport = 8080");

// Look up values
string? host = doc.Get("host");         // "localhost"
string? port = doc.Get("port");         // "8080"
```

### Find / ReverseSearch

```csharp
using var doc = AamDocument.Parse("host = localhost\nport = 8080\nalias = localhost");
var byKey = doc.Find("host");
var byValue = doc.Find("localhost");
var keys = doc.ReverseSearch("localhost");
```

### DeepSearch by key pattern

```csharp
using var doc = AamDocument.Parse(@"
root_path = srv
current_path = root_path
mode = active
");

var matches = doc.DeepSearch("path");
Console.WriteLine(matches["root_path"]); // srv
```

### Loading from Files

```csharp
using var doc = AamDocument.Load("config.aam");
```

## Building and Testing

### Solution Layout

- `aam-csharp.csproj` - class library (NuGet package)
- `tests/AamCsharp.Tests.csproj` - test project
- `examples/AamCsharp.Examples.csproj` - runnable examples
- `apps/AamCsharp.Console/AamCsharp.Console.csproj` - console app
- `AamCsharp.sln` - solution that includes all projects

### Build

```bash
dotnet build AamCsharp.sln
```

### Run Tests

```bash
dotnet test AamCsharp.sln
```

### Run Examples

```bash
dotnet run --project examples/AamCsharp.Examples.csproj -- basic
dotnet run --project examples/AamCsharp.Examples.csproj -- load
```

### Run Console App

```bash
dotnet run --project apps/AamCsharp.Console/AamCsharp.Console.csproj -- parse "host = localhost"
dotnet run --project apps/AamCsharp.Console/AamCsharp.Console.csproj -- load ./examples/config.aam
```

Note: Some tests require the native library to be available. On CI systems without native artifacts, tests gracefully
skip with `DllNotFoundException`.

## Supported Platforms

| Platform | Arch   | Status |
|----------|--------|--------|
| Windows  | x86_64 | ✓      |
| macOS    | x86_64 | ✓      |
| macOS    | ARM64  | ✓      |
| Linux    | x86_64 | ✓      |

## API Reference

### `AamDocument`

Main class for AAM document handling.

#### `Parse(string content): AamDocument`

Parses AAM content from a string.

#### `Load(string path): AamDocument`

Loads an AAM file from disk.

#### `Get(string key): string?`

Returns a value by key.

#### `Find(string query): Dictionary<string, string>`

Finds matching pairs by key, then by value fallback.

#### `DeepSearch(string pattern): Dictionary<string, string>`

Finds key-value pairs where keys contain a pattern.

#### `ReverseSearch(string value): string[]`

Finds keys for a value.

#### `Dispose(): void`

Releases native resources. Implement via `using` statement.

## Error Handling

The C# bindings use standard .NET exceptions:

```csharp
try
{
    using var doc = AamDocument.Parse("invalid content");
}
catch (InvalidOperationException ex)
{
    // Handle parse errors
    Console.WriteLine($"Parse error: {ex.Message}");
}
catch (DllNotFoundException ex)
{
    // Handle missing native library
    Console.WriteLine($"Native library not found: {ex.Message}");
}
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../LICENSE-MIT))

at your option.


