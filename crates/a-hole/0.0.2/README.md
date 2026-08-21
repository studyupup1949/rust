```
   ___        _           _
  / _ \      | |         | |
 / /_\ \_____| |__   ___ | | ___
 |  _  |_____| '_ \ / _ \| |/ _ \
 | | | |     | | | | (_) | |  __/
 \_| |_/     |_| |_|\___/|_|\___|
```
**Attention-hole** — a declarative whitelist for attention

*The hole your attention actually goes through.*

[![Crates.io](https://img.shields.io/crates/v/a-hole.svg)](https://crates.io/crates/a-hole)[![License](https://img.shields.io/crates/l/a-hole.svg)](LICENSE)

## The idea

|  | Pi-hole | a-hole |
|--|---------|--------|
| blocks | ads | noise |
| mechanism | DNS blacklist | attention whitelist |
| list source | community maintained | you, by living |
| default | deny unknown | decay unknown |
| what gets through | everything not blocked | only what you keep returning to |

## What it is

The thing between you and the noise

## What it is not

pi-hole blocks what you don't want
a-hole surfaces only what you've proven you want

## How the whitelist works

You don't write it. You live it.

```
 the internet          your attention
      │                      │
   pi-hole               a-hole
  blacklist             whitelist
      │                      │
  your browser          what matters
```
