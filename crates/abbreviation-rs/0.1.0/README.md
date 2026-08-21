# Abbreviation - The de-abbreviating CLI!

**The idea is simple;** you enter an abbreviation, and it outputs the meaning(s) of it.

## Get Started

1. Install the CLI globally to your system with Cargo.
```sh
cargo install abbreviations-rs
```
2. Initialize the ``abbreviations.json`` file, like this:
```
abbreviation init
```
This command creates a file called abbreviations.json with all the data in the following directory:
- Windows: C:\Users\(your username)\AppData\Roaming\abbreviations_rs or C:/ProgramData
- Linux/MacOS: /home/(your username)/.abbreviations_rs

3. Search for the abbreviation you want!
```
abbreviation search idk
Meaning: I don't know
```

That's all you have to do to start using it!

## Contributing

There are many abbreviations that exist in the world, as a single developer I can't keep up with every single one that exists. If you want to add your own abbreviations, check out the src/abbreviations.json file and go ahead!