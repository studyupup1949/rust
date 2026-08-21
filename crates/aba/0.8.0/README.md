<!--
SPDX-FileCopyrightText: 2023 Gustavo Coutinho de Souza <dev@onemoresuza.mailer.me>

SPDX-License-Identifier: ISC
-->

# aba

[![built with nix](https://builtwithnix.org/badge.svg)](https://builtwithnix.org)

[![REUSE status][reuse-api-badge]][reuse-api-info]
[![Static Badge][shields-io-license]](LICENSES/ISC.txt)

[![Dynamic TOML Badge][version-badge]][refs]
[![builds.sr.ht status](https://builds.sr.ht/~onemoresuza/aba.svg)](https://builds.sr.ht/~onemoresuza/aba?)

[![Crates.io](https://img.shields.io/crates/v/aba?logo=rust)](https://crates.io/crates/aba/versions)
![Crates.io](https://img.shields.io/crates/d/aba?logo=Rust)
[![dependency status][deps-rs-svg]][deps-rs-page]


[reuse-api-badge]: <https://api.reuse.software/badge/git.fsfe.org/reuse/api>
[reuse-api-info]: <https://api.reuse.software/info/git.fsfe.org/reuse/api>
[shields-io-license]: <https://img.shields.io/badge/License-ISC-green?style=flat&logo=spdx&cacheSeconds=31536000>
[deps-rs-svg]: <https://deps.rs/repo/sourcehut/~onemoresuza/aba/status.svg>
[deps-rs-page]: <https://deps.rs/repo/sourcehut/~onemoresuza/aba>
[version-badge]: <https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fgit.sr.ht%2F~onemoresuza%2Faba%2Fblob%2Fmain%2FCargo.toml&query=%24.package.version&logo=sourcehut&label=version>

> **Address Book for [Aerc][aerc]**

# Contributing

This project has a [mailing list][mailing-list] and a [tracker][tracker], which
is for **confirmed** bugs and feature requests.

[mailing-list]: <https://lists.sr.ht/~onemoresuza/aba>
[tracker]: <https://todo.sr.ht/~onemoresuza/aba>

# Changelog

The changelog may be found [here][changelog].

[changelog]: <https://git.sr.ht/~onemoresuza/aba/tree/main/item/CHANGELOG.md>

# Installing

[![Packaging status](https://repology.org/badge/vertical-allrepos/aba.svg)](https://repology.org/project/aba/versions)

## Nix

`aba` is packaged on the [unstable channel][nixpkgs-unstable].

```bash
nix-env -iA nixos.aba # or nixpkgs.aba if on a non NixOS system
```

[nixpkgs-unstable]: <https://search.nixos.org/packages?channel=unstable&from=0&size=50&sort=relevance&type=packages&query=aba>

## crates.io

`aba` is published on [crates.io][crates-io].

```bash
cargo install aba
```

[crates-io]: <https://crates.io/crates/aba>

## Pre-built Binary

Beginning on [0.4.0][040-version], `aba` provides a pre-built binary for
`x86_64-linux` and a checksum for it. They can be download [here][refs].

[040-version]: <https://git.sr.ht/~onemoresuza/aba/refs/0.4.0>

## Manual Install from Manual Build

Follow the [build instructions](#manual-build), then run:

```bash
sudo just install
```

To not install at the default location, `/usr/local`, just set the `PREFIX
variable`:

```bash
just PREFIX="${HOME}/.local" install # now there's no need for root
```

# Building

## Nix

Run:

```bash
nix build .
```

## Manual Build

Install the following dependencies:

1. [just][just];
1. [scdoc][scdoc]; and
1. [Rust][rust].

Then run:

```bash
just
```


# Packaging

## Binary-based Distros

Pre-built binaries are available at every new [tag][refs].

## Source-based Distros

To build `aba` from source be sure to have the following dependencies available:

1. [just][just]
1. [Rust][rust]

## Manpages

To compile the manpages, [scdoc][scdoc] is needed. A [just][just] recipe is
available for both compiling ([doc][doc-recipe]) and Installing and compiling
([install-doc][install-doc-recipe]).

[doc-recipe]: <https://git.sr.ht/~onemoresuza/aba/tree/ad1ca73453231c94dad3b18d503b61441738cacb/item/justfile#L19>
[install-doc-recipe]: <https://git.sr.ht/~onemoresuza/aba/tree/ad1ca73453231c94dad3b18d503b61441738cacb/item/justfile#L29>

# aerc integration

## Address Completion

Add the following to your **aerc.conf**:

```ini
address-book-cmd=aba ls "%s"
```

## Parsing Addresses from an Email

Add the following to your **binds.conf**:

```ini
[view]
aa = :pipe -m aba parse --all<Enter>
```

The option `--all` may be changed depending on what one wants, that is, to
`--from`, `--cc` or `--to`.

# Related Projects

- [aercbook][aercbook], by [renerocksai][renerocksai]: a more minimalistic
address book for [aerc][aerc] written in `Zig` whose functionality
`aba` is **blatantly** based upon.

- [aerc], by [sircmpwn][sircmpwn], maintained by [rjarry][rjarry]: `aba` is an
address book for it, isn't? ;)

[aerc]: <https://sr.ht/~rjarry/aerc/>
[aercbook]: <https://sr.ht/~renerocksai/aercbook/>
[just]: <https://repology.org/project/just/information>
[refs]: <https://git.sr.ht/~onemoresuza/aba/refs>
[renerocksai]: <https://sr.ht/~renerocksai/>
[rjarry]: <https://sr.ht/~rjarry/>
[rust]: <https://repology.org/project/rust/information>
[scdoc]: <https://repology.org/project/scdoc/information>
[sircmpwn]: <https://git.sr.ht/~sircmpwn/>

