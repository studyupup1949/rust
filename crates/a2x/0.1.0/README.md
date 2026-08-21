ALFA to XACML (a2x)
-------------------
[![builds.sr.ht status](https://builds.sr.ht/~gheartsfield/a2x/commits/master.svg)](https://builds.sr.ht/~gheartsfield/a2x/commits/master?)

a2x converts policies written in the ALFA language to XACML 3.0 XML
policies.

Licensed under the [GNU General Public License v3.0 or
later](https://spdx.org/licenses/GPL-3.0-or-later.html).

### CHANGELOG

Please see the [CHANGELOG](CHANGELOG.md) for release history.

### Usage

Convert any files in the `src` directory ending in `.alfa` to XACML,
saving to the `xacml` directory.

```
$ a2x --input src --output xacml
```

The `--input` option can be repeated for as many files or directories
as desired.

Most entities from the XACML spec are predefined and available through
an implicitly imported namespace.  To see a listing, run:

```
$ a2x --show-builtins
```

If you prefer to not have these implicitly imported, they can be
disabled with the ```--disable-builtins``` flag.

The default prefix for `PolicySetId`, `PolicyId`, and `RuleId` can be
customized with the ```--namespace``` option.

### Building

a2x is written in Rust, so you will need to [install
Rust](https://www.rust-lang.org/) to compile it.  Rust version 1.85.0
(stable) or newer is supported.

To build a2x:

```
$ git clone https://git.sr.ht/~gheartsfield/a2x
$ cd a2x
$ cargo build --release --locked
$ ./target/release/a2x --version
a2x 0.1.0
```

### Running tests

a2x has unit tests and full end-to-end tests that ensure ALFA policies
are converted to exact matches of manually verified XACML policies.
To run all these tests, use:

```
$ cargo test
```

### References

* [XACML
  3.0](https://docs.oasis-open.org/xacml/3.0/xacml-3.0-core-spec-os-en.html) -
  the OASIS specification for eXtensible Access Control Markup
  Language.
* [ALFA 1.0](https://groups.oasis-open.org/higherlogic/ws/public/download/55228/alfa-for-xacml-v1.0-wd01.doc) - the OASIS draft specification for ALFA (Word format)
* [ALFA 2.0](https://www.ietf.org/archive/id/draft-brossard-alfa-authz-00.html) - future evolution of the ALFA language
