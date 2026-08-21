add-syntax
==========

Attribute macros that prepend or append arbitrary syntax. Useful with
[`cfg_attr`].

This crate provides two attribute macros, [`prepend`] and [`append`], that
add the tokens passed to them to the start or end of the item to which the
attribute is applied, respectively. This is particularly useful with
[`cfg_attr`].

Example
-------

Conditionally applying `unsafe` when [`#[may_dangle]`][may_dangle] is used:

[may_dangle]: https://github.com/rust-lang/rust/issues/34761

```rust
#[cfg_attr(feature = "dropck_eyepatch", add_syntax::prepend(unsafe))]
impl<#[cfg_attr(feature = "dropck_eyepatch", may_dangle)] T> Drop
    for Foo<T>
{
    fn drop(&mut self) { /* ... */ }
}
```

If the hypothetical feature `dropck_eyepatch` is enabled, the code above
is equivalent to:

```rust
unsafe impl<#[may_dangle] T> Drop for Foo<T> {
    fn drop(&mut self) { /* ... */ }
}
```

Otherwise, if the feature is not enabled, the code is equivalent to:

```rust
impl<T> Drop for Foo<T> {
    fn drop(&mut self) { /* ... */ }
}
```

[`cfg_attr`]:
https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg_attr-attribute
[`prepend`]: https://docs.rs/add-syntax/0.1/add_syntax/attr.prepend.html
[`append`]: https://docs.rs/add-syntax/0.1/add_syntax/attr.append.html
