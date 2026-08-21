# Abler
A simple type to be used on place where different toggles/switches/flags/etc needed, especially in configurations.

# About
It supports (de)serialization via [serde](https://serde.rs/) when feature `serde` is enabled, but also core traits like [Display](https://doc.rust-lang.org/std/fmt/trait.Display.html) and [FromStr](https://doc.rust-lang.org/std/str/trait.FromStr.html) -- unconditionally.

Crate is `no-std` if not accounting it's [build.rs](/build.rs) stage.