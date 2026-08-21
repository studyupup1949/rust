# Rust Advanced String Generator

This project provides a powerful and flexible string generator based on regex-like patterns, with support for features such as character classes, custom repeats, array-based string selection, incremental values, and more. The application can be compiled and used both as a native Rust library and in a WebAssembly (WASM) context.

## Demo

Demo for this project below
https://igarashi.net/rust-advanced-string-generator/

## Features

- **Character Classes**: Supports `\d`, `\w`, `\s`, `\D`, `\W`, `\S`, etc.
- **Custom Repeats**: Generate strings with patterns like `\d{2,4}`.
- **Character Ranges**: Use ranges like `[a-z]`, `[A-Z]`, `[0-9]` to specify sets of characters.
- **Negation in Ranges**: Specify characters not to be included with patterns like `[^a-z]`.
- **Incremental Values**: Automatically increment values using patterns like `\i+` for ascending and `\i-` for descending.
- **Array-Based Selection**: Choose from an array of strings using patterns like `\a`, `\a+`, and `\a-`.
- **Group Capturing and Backreferences**: Capture groups of characters and reference them later in the pattern.
- **WASM Support**: Compile the project to WebAssembly and use it in a web environment.


## Installation

1. **Clone the repository**:
    ```sh
    git clone https://github.com/your-repo/regex_generator.git
    ```
2. **Navigate to the project directory**:
    ```sh
    cd regex_generator
    ```
3. **Build the project**:
    ```sh
    cargo build --release
    ```

### For WebAssembly

1. Ensure you have `wasm-pack` installed:

```sh
cargo install wasm-pack
```

2. Build the project targeting WebAssembly:

```sh
wasm-pack build --target web --features wasm
```

3. Use the generated WebAssembly module in your web project.


## Usage

```sh
regex_generator [OPTIONS] PATTERN [INCREMENT] [ARRAY]
```


### Examples

1. **Generate a string with an incrementing value with leading zeros**:
    ```sh
    ./target/release/regex_generator -p '\\i{:10}' -i 43
    ```

2. **Generate a random string from an array**:
    ```sh
    ./target/release/regex_generator -p '[A-Za-z]{5}' -a 'apple,banana,grape'
    ```

3. **Use a combination of patterns**:
    ```sh
    ./target/release/regex_generator -p '\\w{3}-\\d{2:5}-\\i{:6}' -i 100 -a 'cat,dog,mouse'
    ```


### WASM Example

```javascript
import init, { WasmRegexGenerator } from './pkg/regex_generator_wasm.js';

async function run() {
    await init();

    const generator = new WasmRegexGenerator("\\i+\\d\\d", "1299", ["apple", "banana", "cherry"]);
    console.log(generator.generate()); // Outputs: 1300
    console.log(generator.generate()); // Outputs: 1301
}

run();
```

### Options

| Option               | Description                                                |
|----------------------|------------------------------------------------------------|
| `-h`, `--help`       | Prints help information                                    |
| `-v`, `--version`    | Prints version information                                 |
| `-p`, `--pattern`    | Specifies the pattern to use                               |
| `-i`, `--increment`  | Initial value for the increment (optional)                 |
| `-a`, `--array`      | Array of strings (comma-separated) for `/a` pattern (optional) |

## Supported Patterns

| Pattern  | Description                                                                                   | Example Input     | Example Output         |
|----------|-----------------------------------------------------------------------------------------------|-------------------|------------------------|
| `\d`     | Any digit from `0` to `9`.                                                                     | `\d\d`            | `42`, `07`             |
| `\w`     | Any "word" character: letters, digits, and underscores.                                       | `\w\w\w`          | `abc`, `1X_`           |
| `\s`     | Any whitespace character (space, tab, newline).                                                | `\s\s`            | `  `, `\t `            |
| `\D`     | Any non-digit character.                                                                       | `\D\D`            | `AB`, `--`             |
| `\W`     | Any non-word character.                                                                        | `\W\W`            | `**`, `@#`             |
| `\S`     | Any non-whitespace character.                                                                  | `\S\S\S`          | `abc`, `a1b`           |
| `{n,m}`  | Insert between `n` and `m` times.                                                               | `\d{2,4}`         | `12`, `4321`           |
| `[abc]`  | Insert any one of the characters `a`, `b`, or `c`.                                              | `[abc]{3}`        | `abc`, `cab`           |
| `[^abc]` | Insert any character except `a`, `b`, or `c`.                                                   | `[^abc]{3}`       | `xyz`, `123`           |
| `[a-z]`  | Insert any character in the range from `a` to `z`.                                              | `[a-z]{3}`        | `abc`, `xyz`           |
| `[0-9]{n:z}`  | Insert any number in the range from `0` to `9`, `n` times with `z` of leading zero.        | `[0-9]{3:5}`      | `00827`, `00281`           |
| `\i` or `\i+`    | Insert an incrementing value, starting from the specified value and increasing with each use.  | `\i+\d\d`         | `1300`, `1301`         |
| `\i-`    | Insert a decrementing value, starting from the specified value and decreasing with each use.   | `\i-\d\d`         | `1299`, `1298`         |
| `\i{:z}`    | Insert a incrementing value, starting from the specified value and leading zero.   | `\i{:6}`         | `001299`, `001298`         |
| `\a`     | Insert a random string from an array of values.                                                | `\a`              | `apple`, `banana`      |
| `\a+`    | Insert a string from an array in ascending order.                                              | `\a+`             | `apple`, `banana`      |
| `\a-`    | Insert a string from an array in descending order.                                             | `\a-`             | `cherry`, `banana`     |
| `()`     | Group characters.                                                                              | `(\d\d)`          | `42`                   |
| `\1`     | Backreference to the first captured group.                                                     | `(\d\d)\1`        | `4242`                 |
| `|`      | Alternation; insert either the expression before or the expression after.                       | `a|b`             | `a`, `b`               |

## Testing

### Native Rust

To run the tests in a native Rust environment, execute:

```sh
cargo test
```

### WebAssembly

To build and test the WebAssembly version, follow these steps:

1. Build the WASM module:

   ```sh
   wasm-pack build --target web --features wasm
   ```

2. Create an HTML file to load and test the WASM module.

3. Serve the HTML file locally using a server like Python's `http.server`:

   ```sh
   python3 -m http.server
   ```


## Help

To get help information, run:

```sh
./target/release/regex_generator -h
```

## Version

To check the version of the tool, run:

```sh
./target/release/regex_generator -v
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please submit a pull request or create an issue to report bugs or suggest features.

---
This `README.md` file provides comprehensive documentation for the `RegexGenerator` project, including installation instructions, usage examples, a detailed list of supported patterns, and testing instructions.

