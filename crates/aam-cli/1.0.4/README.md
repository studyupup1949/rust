# Architecture-As-Memory (AAM) Rust Wrapper

This is the official Rust binary wrapper package for **Architecture-As-Memory (AAM)**. It provides a native command line wrapper to run AAM validation, health doctor diagnostics, and visualizer servers seamlessly.

## Installation

Ensure you have **Node.js** (version 18 or above) installed on your system.

```bash
cargo install aam-cli
```

## Quick Start

Initialize architecture cognition inside your project root:
```bash
aam init
```

Validate your current architecture schemas:
```bash
aam validate
```

Diagnose cognitive health and check for structural drifts:
```bash
aam doctor
```

Launch the local visual visualizer server:
```bash
aam dev
```
