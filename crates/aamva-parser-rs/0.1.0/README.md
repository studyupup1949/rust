# AAMVA Parser for Rust

## Overview

`aamva-parser-rs` is a Rust library designed to parse RAW AAMVA data from PDF417 barcodes and convert it into JSON or YAML formats. This library can be particularly useful for applications dealing with driver licenses and identification cards.

## Features

- Parse AAMVA data from a string input.
- Convert parsed data to JSON or YAML formats.
- Handle various fields in the AAMVA specification.
- Easy command-line interface for quick usage.

## Installation

To use `aamva-parser-rs`, add the following line to your `Cargo.toml` file:

```toml
[dependencies]
aamva-parser-rs = "0.1.0"  # Replace with the latest version

