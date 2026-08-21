# Adequate

[![crate::adequate](
https://img.shields.io/crates/v/adequate?label=crates&style=flat)](
https://crates.io/crates/adequate) [![doc::adequate](
https://docs.rs/adequate/badge.svg)](https://docs.rs/crate/adequate)

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

// inputs
let fullname = "Albrecht Dürer";
let username = "albrecht";

let result = validate! {
    "fullname" => fullname => [length::max(3)],
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


## Build

Check `make help`

```zsh
# debug build
% make build:debug
```

## Development

### vet

```zsh
# check code using all vet:xxx targets
% make vet:all
```

### Test

```zsh
% make test

# or check the report by kcov
% make coverage
```

### CI

> TODO: Use Woodpecker CI

Run CI jobs on local docker conatiner (Gentoo Linux) using gitlab-runner.  
See `.gitlab-ci.yml`.


```zsh
# prepare environment variables for CI via .env.ci
% cp .env.ci.sample .env

# e.g. test (see .gitlab-ci.yml)
% make runner-test
```


## Release

All notable released changes of this package will be documented in CHANGELOG
file.

### Unreleased commits

[v0.1.3...trunk](
https://codeberg.org/grauwoelfchen/adequate/compare/v0.1.3...trunk)


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
