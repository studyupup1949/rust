# abs-cli

Lightweight CLI parser in Rust.

## examples

```rust
use abs_cli::CLI;

fn main() {
    let mut program = CLI::new();
    program
        .name("My Program")
        .version("1.0.0")
        .description("My super cool cli program")
        .option("-l, --ls", "list the directory");
    program.parse();

    if let Some(ls_value) = program.get("--ls") {
        println!("Option --ls provided with value: {}", ls_value);
    }
}
```

If command `--ls` was passed, `Some` value will be returned. If there is a value after `--ls`, for example: `--ls hello`, value will be: `hello`.

Library has built-in help (`--help`, `-h`) and version (`--version`, `-v`) commands.
