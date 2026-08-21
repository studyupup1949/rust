# Advent-of-Utils

## Introduction
`Advent of Utils` helps you solving your [Advent of Code](https://adventofcode.com/) challenges. Not by implementing the solutions for you, but by handling the boilerplate work so you can focus on solving the puzzles.

## Table of Contents
- [Features](#features)
- [Setup](#setup)
- [Usage](#usage)
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

## Usage

### 1. Create Your Solution Structure

First, create a new Rust project for your solutions:

```bash
cargo new aoc-2023
cd aoc-2023

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

### 2. Set Up Your Project

In your project's `lib.rs`, use the `add_days!` macro to generate the boilerplate for your solutions:

```rust
use advent_of_utils::add_days;

// Generate modules for days 1 through 25
add_days!(1..=25);
```

### 3. Implement Solutions

For each day you want to solve, implement the `Solution` trait in the corresponding module. You need to create a file for all the days you added yet to your macro or the compiler will complain. Here's an example for day 1:

```rust
// src/day01.rs

use advent_of_utils::{Solution, AocOption};

pub struct Day01;

impl Solution for Day01 {
    fn part1(&self, input: String) -> AocOption {
        // Your solution for part 1
        1.into()
    }

    fn part2(&self, input: String) -> AocOption {
        // Your solution for part 2
        "part2 result".into()
    }
}
```

### 4. Run Solutions

Once your solutions are implemented and your code compiles you can run the your code through the `aou` CLI:

## CLI Reference

### Basic Commands

```bash
# Run a specific day's solution
aou run <YEAR> <DAY>

# Run all implemented solutions
aou run <YEAR>
```

For more informations on your options for the CLI run:

```bash
aou --help
```

## Disclaimer

This tool adheres to the automation guidelines outlined in the [Advent of Code FAQ on automation](https://www.reddit.com/r/adventofcode/wiki/faqs/automation/):
- The tool will only fetch the puzzle input for a day if it is not already cached locally or during controlled testing.
- Upon successfully fetching the input online, it will cache the input by default for future use.
- The User-Agent header in requests includes a reference to this repository. If there are any issues, please contact me (Itron_al_Lenn) via a platform linked in my [GitHub profile](https://github.com/Itron-al-Lenn).
