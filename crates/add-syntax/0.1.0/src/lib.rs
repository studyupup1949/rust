/*
 * Copyright 2022 taylor.fish <contact@taylor.fish>
 *
 * This file is part of add-syntax.
 *
 * add-syntax is licensed under the Apache License, Version 2.0
 * (the "License"); you may not use add-syntax except in compliance
 * with the License. You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! Attribute macros that prepend or append arbitrary syntax. Useful with
//! [`cfg_attr`].
//!
//! This crate provides two attribute macros, [`prepend`] and [`append`], that
//! add the tokens passed to them to the start or end of the item to which the
//! attribute is applied, respectively. This is particularly useful with
//! [`cfg_attr`].
//!
//! Example
//! -------
//!
//! Conditionally applying `unsafe` when [`#[may_dangle]`][may_dangle] is used:
//!
//! [may_dangle]: https://github.com/rust-lang/rust/issues/34761
//!
//! ```rust
//! # #![cfg_attr(feature = "dropck_eyepatch", feature(dropck_eyepatch))]
//! # struct Foo<T>(T);
//! #[cfg_attr(feature = "dropck_eyepatch", add_syntax::prepend(unsafe))]
//! impl<#[cfg_attr(feature = "dropck_eyepatch", may_dangle)] T> Drop
//!     for Foo<T>
//! {
//!     fn drop(&mut self) { /* ... */ }
//! }
//! ```
//!
//! If the hypothetical feature `dropck_eyepatch` is enabled, the code above
//! is equivalent to:
//!
//! ```rust
//! # #![cfg_attr(feature = "dropck_eyepatch", feature(dropck_eyepatch))]
//! # struct Foo<T>(T);
//! # #[cfg(feature = "dropck_eyepatch")]
//! unsafe impl<#[may_dangle] T> Drop for Foo<T> {
//!     fn drop(&mut self) { /* ... */ }
//! }
//! ```
//!
//! Otherwise, if the feature is not enabled, the code is equivalent to:
//!
//! ```rust
//! # struct Foo<T>(T);
//! impl<T> Drop for Foo<T> {
//!     fn drop(&mut self) { /* ... */ }
//! }
//! ```
//!
//! [`cfg_attr`]:
#![doc = "https://doc.rust-lang.org/reference/conditional-compilation.html\
#the-cfg_attr-attribute"]
//! [`prepend`]: macro@prepend
//! [`append`]: macro@append

use proc_macro::{Delimiter, TokenStream, TokenTree};

fn split_attrs(item: TokenStream) -> (TokenStream, TokenStream) {
    let mut attrs = Vec::<TokenTree>::new();
    let mut iter = item.into_iter().fuse();
    loop {
        use Delimiter::*;
        use TokenTree::*;
        match [iter.next(), iter.next()] {
            [Some(Punct(p)), Some(Group(g))]
                if (p.as_char(), g.delimiter()) == ('#', Bracket) =>
            {
                attrs.extend([p.into(), g.into()]);
            }
            mut trees => {
                let trees = trees.iter_mut().flat_map(Option::take);
                return (
                    attrs.into_iter().collect(),
                    trees.chain(iter).collect(),
                );
            }
        };
    }
}

/// Adds the tokens provided to this attribute to the start of the item to
/// which this attribute is applied.
#[proc_macro_attribute]
pub fn prepend(attr: TokenStream, item: TokenStream) -> TokenStream {
    let (mut item_attrs, rest) = split_attrs(item);
    item_attrs.extend(attr.into_iter().chain(rest));
    item_attrs
}

/// Adds the tokens provided to this attribute to the end of the item to
/// which this attribute is applied.
#[proc_macro_attribute]
pub fn append(attr: TokenStream, mut item: TokenStream) -> TokenStream {
    item.extend(attr);
    item
}
