# Absurd
A command line tool for managing stores in Surreal DB.

## Installation
Absurd is available on [crates.io](https://crates.io/crates/absurd), and requires that you also have [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed.

```sh
cargo install absurd
```

That's it!

## Usage
Absurd should be run within the root of your project.  
Running `absurd -h` will display the following output.

```sh
Command line tool for managing Surreal stores.

Usage: absurd <COMMAND>

Commands:
  create    Specify which type of component you want to create
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

A component is one of `Controller`, `Model`, or `Store`.  
You can think of these components as the following:  
* `Controller`    Controllers are a layer in between your models and stores
* `Model`         Models represents any record from the store
* `Store`         Stores handles the connection to a database

## Usage Examples
### Create a new store
Currently there is only support for the `Mem` and `RocksDB` engines.  
In the future, we plan to add support for all available SurrealDB engines including `Ws`.
```sh
# Create a store called General, using the Mem engine.
# This creates an in memory store, where no data is persisted.
absurd create store General
absurd create store General --engine mem

# Create a store called Setting, that stores the data in a file.
absurd create store Setting --engine rocks-db
```

# Create a new model
Models are a way to map your database records to a Rust struct.

There are two available table schemas available in SurrealDB: Schemaless and Schemafull.  
By default, Schemafull will be selected, unless the model name is passed like `Setting/Dark` without any `--fields`.  

A Schemafull table will be a single struct, where each field is a property of the struct.  
A Schemaless table will be a struct with a single field, where the value is a map of the record.

All models will be given the `id` with the type `Option<Thing>`.  
This is set to optional as it makes it easier to create new records without having to create a new struct.

```sh
# All fields will be given the data type of String. Support may be added to set
# the data type in the CLI.

# Creates a Schemafull model called User, with the fields name and email.
# User { id: Option<Thing>, name: String, email: String }
absurd create model User --fields name email
absurd create model User --fields name email --schemafull

# Create a Schemaless model called Setting, with the fields name and value.
# SettingDark { id: Option<Thing>, name: String, value: String }
absurd create model Setting/Dark
absurd create model Setting --fields dark --schemaless
```
