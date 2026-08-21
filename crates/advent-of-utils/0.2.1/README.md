# Advent-of-Utils

## Introduction
`Advent of Utils` helps you solving your [Advent of Code](https://adventofcode.com/) challenges. Not by implementing the solutions for you, but by handling the boilerplate work so you can focus on solving the puzzles.

## Table of Contents
- [Features](#features)
- [Setup and Usage](#setup-and-usage)
- [Implementation Guide](#implementation-guide)
- [CLI Reference](#cli-reference)
- [Disclaimer](#disclaimer)

## Features
- 🚀 Automatic input fetching with local caching
- 📊 Built-in performance benchmarking
- 🔧 Simple macro-based setup
- 💾 Session token management

## Setup

### Prerequisites
1. An Advent of Code account and session token
2. Rust toolchain installed

### Installation
Add this to your `Cargo.toml`:
```toml
[dependencies]
advent-of-utils = "0.2.0"

[lib]
crate-type = ["cdylib"]
```

Install the CLI using:
```bash
cargo install advent-of-utils-cli
```

### Environment Setup

Set your Advent of Code session token:

```bash
# Linux/MacOS
export AOC_SESSION=your_session_token

# Windows (PowerShell)
$env:AOC_SESSION="your_session_token"
```

You will find your session token in the cookies on the [Advent of Code](https://adventofcode.com/) page when you are logged in.

## CLI Reference

### Available Commands

```bash
# Run a specific day's solution
aou run <YEAR> <DAY>

# Run all implemented solutions
aou run <YEAR>

# Show help
aou --help
```
## Disclaimer

This tool adheres to the automation guidelines outlined in the [Advent of Code FAQ on automation](https://www.reddit.com/r/adventofcode/wiki/faqs/automation/):
- The tool will only fetch the puzzle input for a day if it is not already cached locally or during controlled testing.
- Upon successfully fetching the input online, it will cache the input by default for future use.
- The User-Agent header in requests includes a reference to this repository. If there are any issues, please contact me (Itron_al_Lenn) via a platform linked in my [GitHub profile](https://github.com/Itron-al-Lenn).
