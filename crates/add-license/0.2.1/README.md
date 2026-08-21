# About

A Cargo subcommand that creates a file with a user-defined license in the project's directory.  

# Installation

Download from [crates.io](https://crates.io/crates/add-license):  
`cargo add add-license`  
or download [Source Code](https://gitlab.com/marquess-projects/add-license)  

Install the crate with:  
`cargo install add-license`  
or locally with `cargo install --path .`  

# Usage

`cargo add-license`  
Creates the LICENSE file based on your Cargo.toml file. If you have 'authors' and
'year' attributes there, it will replace the `<name>`, `<year>` fields inside the
LICENSE file.  
* The license attribute in Cargo.toml must be ONE of [AGPL-3.0-only, Apache-2.0, BSD-2-Clause, BSD-3-Clause, GPL-2.0, GPL-3.0, LGPL-2.1, MIT]  

`cargo add-license MIT`  
Replaces the 'license' attribute inside Cargo.toml and creates a LICENSE file. If you have 'authors' and
'year' attributes there, it will replace the `<name>`, `<year>` fields inside the
LICENSE file.  
* The license argument must be ONE of [AGPL3, APACHE2, BSD2, BSD3, GPL2, GPL3, LGPL2.1, MIT]

`cargo add-license -y:2024`  
ONLY replaces the 'year' attribute inside Cargo.toml.

`cargo add-license -a:Pablo_Marques,Carl_Menger`  
ONLY replaces the 'authors' attribute inside Cargo.toml.  
* Replace all spaces between names with '_', and place no space after the ',' symbol.  

All valid commands:  
`cargo add-license`  

`cargo add-license MIT`  
`cargo add-license -y:2024`  
`cargo add-license -a:Mary_Johnson,Carl_Menger`  

`cargo add-license MIT -y:2024`  
`cargo add-license MIT -a:Marques`  
`cargo add-license -y:2024 -a:Marques`  
`cargo add-license -a:Marques -y:2024`  

`cargo add-license MIT -y:2024 -a:Marques`  
`cargo add-license MIT -a:Marques -y:2024`  

**DO NOT FORGET**  
If you do not specify 'authors' and a 'year' in Cargo.toml OR in the command argument, your LICENSE will be created with the default `<year>` and `<authors>` fields.
