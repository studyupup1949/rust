<div align="center">

# Accessorise

[![Crates.io](https://img.shields.io/crates/v/accessorise)](https://crates.io/crates/accessorise)
[![License](https://img.shields.io/badge/license-MIT%2FApache-blue)](#license)
[![Downloads](https://img.shields.io/crates/d/accessorise)](https://crates.io/crates/accessorise)
[![Docs](https://docs.rs/accessorise/badge.svg)](https://docs.rs/accessorise/latest/accessorise/)
[![Twitch Status](https://img.shields.io/twitch/status/coruscateor)](https://www.twitch.tv/coruscateor)

[X](https://twitter.com/Coruscateor) | 
[Twitch](https://www.twitch.tv/coruscateor) | 
[Youtube](https://www.youtube.com/@coruscateor) | 
[Mastodon](https://mastodon.social/@Coruscateor) | 
[GitHub](https://github.com/coruscateor) | 
[GitHub Sponsors](https://github.com/sponsors/coruscateor)

Add accessors to your objects.

</div>

```rust

use accessorise::*;

//use super::*;

use paste::paste;

trait TestTrait
{

    trait_get!(a_number, i32);

    trait_set!(a_number, i32);

    trait_get_ref!(a_string, String);

    trait_get_mut!(a_string, String);

    trait_get!(a_number_doc, i8, "This is a getter declaration, presumably for a number field.");

    trait_set!(a_number_doc, i8, "This is a setter declaration, presumably for a number field.");

    trait_get_ref!(a_string_doc, String, "This is a getter declaration, presumably for a String field.");

    trait_get_mut!(a_string_doc, String, "This is a setter declaration, presumably for a String field.");

}

#[derive(Default)]
struct TestStruct
{

    a_number: i32,
    a_string: String,
    a_number_doc: i8,
    a_string_doc: String,
    some_numbers: Vec<i32>

}

impl TestStruct
{

    pub fn new() -> Self
    {

        Self::default()

    }

    impl_get!(a_number, i32);

    impl_set!(a_number, i32);

    impl_get_clone!(a_string, String);

    impl_get_clone!(a_string_doc, String, "Returns a cloned String.");

    impl_get_ref!(some_numbers, Vec<i32>, "Returns some numbers by reference.");

    impl_get_mut!(some_numbers, Vec<i32>, "Returns some numbers by mutable reference.");

    impl_get_clone!(some_numbers, Vec<i32>);

    impl_set_clone!(some_numbers, Vec<i32>);

}

impl TestTrait for TestStruct
{

    impl_trait_get!(a_number, i32);

    impl_trait_set!(a_number, i32);

    impl_trait_get_ref!(a_string, String);

    impl_trait_get_mut!(a_string, String);

    impl_trait_get!(a_number_doc, i8, "This is a getter implementation for a number field.");

    impl_trait_set!(a_number_doc, i8, "This is a setter implementation for a number field.");

    impl_trait_get_ref!(a_string_doc, String, "This is a getter implementation for a String field.");

    impl_trait_get_mut!(a_string_doc, String, "This is a setter implementation for a String field.");
    
}

fn main()
{
    
    let mut test_struct = TestStruct::new();

    let _number = test_struct.a_number();

    test_struct.set_a_number(5);

}

```

<br/>

## Compiler:

Build with the latest stable compiler.

<br/>

## Todo:

- Add documentation
- Add more macros
- Add macro rules for things like aliases and multiple accessor declarations and definitions in one macro call.

<br/>

## Coding Style

This project uses a coding style the emphasises the use of white space over keeping the line and column counts as low as possible.

So this:

```rust
fn bar() {}

fn foo()
{

    bar();

}

```

Not this:

```rust
fn bar() {}

fn foo()
{
    bar();
}

```

<br/>

## License

Licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](./LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0 (see also: https://www.tldrlegal.com/license/apache-license-2-0-apache-2-0))
- MIT license ([LICENSE-MIT](./LICENSE-MIT) or http://opensource.org/licenses/MIT (see also: https://www.tldrlegal.com/license/mit-license))

at your discretion

<br/>

## Contributing

Please clone the repository and create an issue explaining what feature or features you'd like to add or bug or bugs you'd like to fix and perhaps how you intend to implement these additions or fixes. Try to include details though it doesn't need to be exhaustive and we'll take it from there (dependant on availability).

<br/>

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.




