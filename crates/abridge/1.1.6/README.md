# abridge

Compress a sorted word list (often called a dictionary) or decompress a file compressed by `abridge` or GNU 
`word-list-compress`.

`abridge` is a port of GNU [word-list-compress](https://duckduckgo.com/?q=word-list-compress) to Rust. It doesn't have
anything to do with the [GNU Aspell](http://aspell.net/) project. `word-list-compress` is about 150 SLOC. `abridge` is
about 50 SLOC. They perform identically.

`abridge` is both a Rust library and CLI, so you can use `abridge` in your Rust project or run it by itself. The CLI
expects input from stdin and will output to stdout.

It only relies on [clap](https://clap.rs) and Rust's built-in `substring` crate.

## Install

You'll need the Rust compiler and Cargo. The easiest way to do that is to get [rustup](https://rustup.rs/).

```shell
cargo install abridge
```

## Usage

See `abridge --help`.

## Examples

```shell
abridge -c < words.txt # compress words.txt
```

```shell
abridge --decompress < words.tzip # decompress words.tzip
```

```shell
abridge --compress < words.txt > words.tzip # compress words.txt and save to words.tzip 
```

## Safety 

`abridge` can *compress* word lists where:

- Words are in alphabetical order
- Words are separated by newline
- Words contain ASCII characters 

Words may include uppercase characters.

`abridge` can *decompress* files compressed by `abridge` or `word-list-compress` alike.

## Testing

Run `cargo test`.

## License

`abridge` is licensed under [GNU General Public License](https://www.gnu.org/licenses/gpl-3.0.en.html). 

