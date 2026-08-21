# Apple II Memory Map

![unit tests](https://github.com/dfgordon/a2-memory-map/actions/workflows/node.js.yml/badge.svg)
![unit tests](https://github.com/dfgordon/a2-memory-map/actions/workflows/rust.yml/badge.svg)

This is a database of Apple II special addresses intended to be used with language servers.  The central element is the file `map.json` which maps addresses to a set of descriptive strings.

There are bindings for TypeScript and Rust, see the packages on [npmjs.com](https://www.npmjs.com/package/a2-memory-map) and [crates.io](https://crates.io/crates/a2-memory-map).

## Background

The Apple II line of computers was often programmed by explicitly interacting with certain fixed addresses, especially in the I/O area, the zero page, and the ROM.

The "Apple IIe Reference Manual" has low level information.  The "Applesoft BASIC Programmer's Reference Manual" provides a chapter "Pokes, Peeks, and Calls" with information especially useful to BASIC programmers.

"All About Applesoft" from [Call-A.P.P.L.E.](https://www.callapple.org/) has a more in depth treatment.

Sometimes an ambiguity is best resolved by studying the relevant disassembly.