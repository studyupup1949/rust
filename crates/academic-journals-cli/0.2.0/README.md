# academic-journals-cli

Command-line interface for looking up academic journal abbreviations and full names.

## Installation

```bash
cargo install academic-journals-cli
```

## Usage

Look up the abbreviation for a journal full name:

```bash
academic-journals-cli string "Critical Care Medicine" --abbreviation
# Crit Care Med
```

Look up the full name for an abbreviation:

```bash
academic-journals-cli string "Crit Care Med"
# Critical Care Medicine
```

Process a file of journal names (one per line):

```bash
academic-journals-cli file journals.txt --abbreviation
```

## Data Source

Journal data is provided by the [JabRef](https://abbrv.jabref.org) project and is released under
CC0 1.0 Universal (CC0 1.0) Public Domain Dedication.

## License

Apache-2.0
