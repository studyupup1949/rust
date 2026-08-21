# aam-ruby

Ruby bindings for `aam-rs`, built with `magnus`.

## Features

- Parse AAM from string input.
- Direct lookup, key/value find, deep search, reverse search.
- Lightweight native extension with tiny Ruby API surface.

## Build extension

```bash
cargo build --manifest-path ruby/ext/aam_rb/Cargo.toml --release
```

## Build gem

```bash
cp ruby/ext/aam_rs/target/release/libaam_rs_ruby.so ruby/lib/aam_rs_ruby.so
gem build ruby/aam-ruby.gemspec
```

## Usage

```ruby
require_relative 'ruby/lib/aam_rb'

doc = AamRb::AAM.parse("host = localhost\nport = 8080")
value = doc.get('host')
puts value
```

## More examples

```ruby
require_relative 'ruby/lib/aam_rb'

doc = AamRb::AAM.parse("host = localhost\nalias = localhost")
puts doc.reverse_search('localhost').inspect
puts doc.find('host').inspect

# missing values return nil
missing = doc.get('missing')
puts missing.nil?
```

## Tests

```bash
cargo test --manifest-path ruby/ext/aam_rb/Cargo.toml
ruby ruby/tests/test_aam_rs.rb
```

Ruby tests now cover successful lookups, reverse search behavior, missing key handling, and parse errors.

