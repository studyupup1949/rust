# MO-Libary (WIP)

[![dependency status](https://deps.rs/repo/github/arma3modorganizer/MO-Libary/status.svg)](https://deps.rs/repo/github/arma3modorganizer/MO-Libary)
[![Travis CI](https://travis-ci.org/arma3modorganizer/MO-Libary.svg?branch=master)](https://travis-ci.org/arma3modorganizer/MO-Libary)
[![Build status](https://ci.appveyor.com/api/projects/status/d39clo2lta1qbv08?svg=true)](https://ci.appveyor.com/project/Scarjit/mo-libary)

This is the backend libary, that powers all Arma3 Mod Organizer projects.

Due to the WIP status of the project, anything can and will most likely change !

## Usage
 1. Clone the repository
 2. Use ```cargo build --all --all-targets``` to build it.
    1. If you are using rust, you can instead import it as a cargo crate:
    ```
    [dependencies]
    a3mo_lib = {path="path_to_lib_folder"}
    ```
 3. An example for FFI bindings will be uploaded, once the Libary is no longer WIP.
