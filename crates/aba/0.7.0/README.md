<!--
SPDX-FileCopyrightText: 2023 Gustavo Coutinho de Souza <dev@onemoresuza.mailer.me>

SPDX-License-Identifier: ISC
-->

# aba

[![built with nix](https://builtwithnix.org/badge.svg)](https://builtwithnix.org)
[![REUSE status][reuse-api-badge]][reuse-api-info]
[![Static Badge][shields-io-license]](LICENSES/ISC.txt)
[![dependency status][deps-rs-svg]][deps-rs-page]
[![builds.sr.ht status](https://builds.sr.ht/~onemoresuza/aba.svg)](https://builds.sr.ht/~onemoresuza/aba?)
[![Dynamic TOML Badge][version-badge]][refs]
[![Crates.io](https://img.shields.io/crates/v/aba?logo=rust)](https://crates.io/crates/aba/versions)


[reuse-api-badge]: <https://api.reuse.software/badge/git.fsfe.org/reuse/api>
[reuse-api-info]: <https://api.reuse.software/info/git.fsfe.org/reuse/api>
[shields-io-license]: <https://img.shields.io/badge/License-ISC-green?style=flat&logo=spdx&cacheSeconds=31536000>
[deps-rs-svg]: <https://deps.rs/repo/sourcehut/~onemoresuza/aba/status.svg>
[deps-rs-page]: <https://deps.rs/repo/sourcehut/~onemoresuza/aba>
[version-badge]: <https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fgit.sr.ht%2F~onemoresuza%2Faba%2Fblob%2Fmain%2FCargo.toml&query=%24.package.version&logo=sourcehut&label=version>

> **Address Book for [Aerc][aerc]**

# Contributing

This project has the default `Sourcehut` mailing lists:

1. [aba-discuss][aba-discuss];
1. [aba-devel][aba-devel];
1. [aba-announce][aba-announce].

And a [tracker][tracker] for **confirmed** bugs and
feature requests.

[aba-discuss]: <https://lists.sr.ht/~onemoresuza/aba-discuss>
[aba-devel]: <https://lists.sr.ht/~onemoresuza/aba-devel>
[aba-announce]: <https://lists.sr.ht/~onemoresuza/aba-announce>
[tracker]: <https://todo.sr.ht/~onemoresuza/aba>

# Changelog

The changelog may be found [here][changelog].

[changelog]: <https://git.sr.ht/~onemoresuza/aba/tree/main/item/CHANGELOG.md>

# Installing

## Nix User Repository - NUR

`aba` is packaged on [NUR][nur].

[nur]: <https://github.com/nix-community/nur-combined/tree/master/repos/onemoresuza/pkgs/aba/default.nix#L41>

## Pre-built Binary

Beginning on [0.4.0][040-version], `aba` provides a pre-built binary for
`x86_64-linux` and a checksum for it. They can be download [here][refs].

[040-version]: <https://git.sr.ht/~onemoresuza/aba/refs/0.4.0>

## Manual Install

Follow the [build instructions](#manual-build), then run:

```bash
sudo just install
```

To not install at the default location, `/usr/local`, just set the `PREFIX
variable`:

```bash
just --set PREFIX "${HOME}/.local" install # now there's no need for root
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
just build
```

[just]: <https://github.com/casey/just>
[scdoc]: <https://sr.ht/~sircmpwn/scdoc/>
[rust]: <https://www.rust-lang.org/tools/install>

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

[aercbook]: <https://sr.ht/~renerocksai/aercbook/>
[renerocksai]: <https://sr.ht/~renerocksai/>
[aerc]: <https://sr.ht/~rjarry/aerc/>
[sircmpwn]: <https://git.sr.ht/~sircmpwn/>
[rjarry]: <https://sr.ht/~rjarry/>
[refs]: <https://git.sr.ht/~onemoresuza/aba/refs>
