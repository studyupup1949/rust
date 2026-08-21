# ac-library-rs expander for cargo-compete

## What's this

- ac-library-rs (<https://github.com/rust-lang-ja/ac-library-rs>)
- cargo compete (<https://github.com/qryxip/cargo-compete>)

A wrapper of ac-library-rs/expand.py, which is script to append required modules to your submition code.
For user of cargo-compete, great competitive programing helper, this expander were made.

## Install

Install from Cargo and build

```cmd
cargo install ac-library-rs-expander
```

If ac-library-rs is not in local, please clone it from GitHub.

```cmd
git clone https://github.com/rust-lang-ja/ac-library-rs.git
```

To determine path to ac-library-rs, please set an environment variable `$AC_LIBRARY_RS_HOME`

```cmd
export AC_LIBRARY_RS_HOME=path/to/ac-library-rs
```

## Usage

**In your cargo-compete contest workspace,**

```cmd
exl -t a modint
```

### Flags and Options

- *--all, -a* -> append all modules
- *-d, --doc-hidden* -> whether remove commet lines or empty lines
- *-h, --help*
- *-V, --version*
- *-t, --target-file-name* { i.e. "a" } -> id of problem

### Available args for Module name

These are **cAsE-InsEnsItIvE**

>Convolution,
>FenwickTree,
>Dsu,
>LazySegTree,
>Math,
>MaxFlow,
>MincostFlow,
>ModInt,
>Scc,
>SegFree,
>String,
>Twosat,
