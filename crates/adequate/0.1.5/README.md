# Adequate

[![status-badge](https://ci.grauwoelfchen.net/api/badges/4/status.svg)](
https://ci.grauwoelfchen.net/repos/4) [![doc::adequate](
https://docs.rs/adequate/badge.svg)](https://docs.rs/crate/adequate) [
![crate::adequate](
https://img.shields.io/crates/v/adequate?label=crates&style=flat)](
https://crates.io/crates/adequate)


A yet another validation library provides a macro inspired by [Accord](
https://github.com/ChrisBuchholz/accord).


## Repositories

This library is developed mainly on [Codeberg.org](
https://codeberg.org/grauwoelfchen/adequate), but the source code is hosted also
on [sourcehut](https://git.sr.ht/~grauwoelfchen/adequate).

Any patches, merge/pull requests or issues on those repositories are welcomed.

```zsh
# the main branch is "trunk"
% git clone git@codeberg.org:grauwoelfchen/adequate.git
% git --no-pager branch -v
* trunk xxxxxxx XXX
```

## Installation

```zsh
% cargo install adequate
```

## Usage

> TODO: Add more detailed examples

See `src/validation` directory for validators.

```rust
use adequate::validation::{contain, length};

// input
let fullname = Some("Albrecht Dürer".to_string());
let username = "albrecht".to_string();

let result = validate! {
    "fullname" => fullname => [
        length::max_if_present(3)
    ],
    "username" => username => [
        length::within(3..9),
        contain::contains_only_alphanumeric_chars(),
    ],
};
assert!(result.is_err());
```

### Validations

###### Contain

* contains
* contains_any
* contains_any_digits
* contains_any_lower_letters
* contains_any_upper_letters
* contains_only
* contains_only_alphanumeric_chars
* contains_if_present
* contains_if_given
* not_contain_if_given

###### Length

* max
* max_if_present
* min
* min_if_present
* within


## Development

See help as below:

```zsh
% make help
```

### Vet

```zsh
% make check
% make fmt
% make lint

# check code using all these vet-{check,format,lint} targets at once
% make vet
```

### Test

```zsh
% make test

# check the report by kcov
% make cov
```

### Build

```zsh
# debug build
% make build
```


## Release

All notable released changes of this package will be documented in CHANGELOG
file.

### Unreleased commits

[v0.1.5...trunk](
https://codeberg.org/grauwoelfchen/adequate/compare/v0.1.5...trunk)


## License

```text
Adequate
Copyright 2020-2025 Yasha

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

   http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```
