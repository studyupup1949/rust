# fs_utils

A high-performance Rust library for file system operations with progress reporting and advanced search capabilities.

## Features

- **File Search**:
  - Recursive directory traversal
  - Glob pattern matching
  - Regular expression filtering
  - Case-sensitive/insensitive options

- **Directory Operations**:
  - Calculate total size (bytes or human-readable)
  - Recursive copy with progress reporting
  - File counting

- **File Operations**:
  - Copy with progress callbacks
  - Move with verification
  - Progress reporting with transfer speed

- **Performance**:
  - Asynchronous I/O with Tokio
  - Efficient memory usage
  - Parallel operations where beneficial

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
fs_utils = "0.1"
