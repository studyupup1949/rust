# Rusty CLI Framework

This is a simple framework for building CLI applications in Rust. It is designed to be simple to use and easy to extend.


## Usage

To use this framework, you need to add it to your `Cargo.toml` file:

```toml
[dependencies]
rusty_cli = "0.1.0"
```

Then you can use it in your code like this:

```rust
use rusty_cli::{Command, CommandError, CommandManager, Opt, OptValue};

struct EchoCommand;


impl Command for EchoCommand {
    fn run(
        &self,
        args: Vec<String>,
        opts: std::collections::HashMap<Opt, OptValue>,
    ) -> Result<(), CommandError> {
        let output = args.join(" ");
        Ok(())
    }
}


fn main() {
    let mut manager = CommandManager::new();
    
    manager.add_command("echo".to_string(), Box::new(EchoCommand));
    
    manager.run(std::env::args().skip(1).collect());
}
```

## Options

You can also add options to your commands. Options are specified using the `Opt` enum, and are passed to the `run` method as a `HashMap<Opt, OptValue>`. Here is an example of a command that takes a `--uppercase` option:

```rust
use rusty_cli::{Command, CommandError, CommandManager, Opt, OptValue};

struct LsCommand;

impl Command for LsCommand {
    /// Returns the options available for the `ls` command.
    fn opts(&self) -> Vec<Opt> {
        vec![
            Opt::new(
                "a",
                "all",
                OptValue::BOOL(false),
                "do not ignore entries starting with .",
            ),
            Opt::new(
                "l",
                "long",
                OptValue::BOOL(false),
                "use a long listing format",
            ),
            Opt::new(
                "h",
                "human-readable",
                OptValue::BOOL(false),
                "with -l and -s, print sizes like 1K 234M 2G etc.",
            ),
        ]
    }

    /// Executes the `ls` command with the given arguments and options.
    fn run(
        &self,
        args: Vec<String>,
        opts: std::collections::HashMap<Opt, OptValue>,
    ) -> Result<(), CommandError> {
        println!(
            "Running ls command with args: {:?} and opts {:?}",
            args, opts
        );

        // Do ls command logic here

        let mut all = false;

        if let Some(opt) = rusty_cli::find_opt(opts, "--all".to_string()) {
            if let OptValue::BOOL(value) = opt {
                all = value;
            }
        }
        println!("all: {}", all);

        for entry in std::fs::read_dir(".").unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            let file_name = path.file_name().unwrap().to_str().unwrap().to_owned();

            if !all && file_name.starts_with(".") {
                continue;
            }

            println!("{}", file_name);
        }

        Ok(())
    }
}

fn main() {
    let mut manager = CommandManager::new();

    manager.add_command("ls".to_string(), Box::new(LsCommand));

    manager.run(std::env::args().skip(1).collect());
}
```


The supported `OptValue` types are:

- `STRING(String)`: A string value.
- `BOOL(bool)`: A boolean value.
- `INT(i64)`: An integer value.
- `UINT(u64)`: An unsigned integer value.

