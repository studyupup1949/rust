//! Easy Address Comparison
//!
//! A set of macros to allow your types to be compared based on where they are stored in memory. This is useful when two instances of a type should not be considered equal unless they are literally the same instance.
//!
//! With this crate, you can derive `AddressEq`, `AddressOrd`, or `AddressHash` depending on your needs.
//!
//! ## Usage
//!
//! ```rust
//! use address_cmp::AddressEq;
//!
//! #[derive(AddressEq, Debug)]
//! struct A {
//!   pub a: u8,
//! }
//!
//! let a1 = A { a: 0 };
//! let a2 = A { a: 0 };
//!
//! assert_ne!(a1, a2);
//! assert_eq!(a1, a1);
//! ```

#[cfg(doc)]
use std::cmp::{Eq, Ord, PartialEq, PartialOrd};
#[cfg(doc)]
use std::hash::Hash;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derives [`PartialEq`] and [`Eq`] based on memory addresses.
///
/// By deriving [`AddressEq`], the [`PartialEq`] and [`Eq`] traits will automatically be
/// implemented for the given type. The implementations will use a given instance's address
/// in memory in when checking equality.
/// ```rust
/// use address_cmp::AddressEq;
///
/// #[derive(AddressEq, Debug)]
/// struct Person {
///     pub age: u8,
///     pub name: String,
/// }
///
/// let p1 = Person { age: 22, name: "Mercutio".into() };
/// let p2 = Person { age: 22, name: "Mercutio".into() };
///
/// assert_ne!(p1, p2);
/// assert_eq!(p1, p1);
/// ```
#[proc_macro_derive(AddressEq)]
pub fn address_eq(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let lifetime = &input.generics;
    (quote! {
        #[automatically_derived]
        impl #lifetime ::std::cmp::PartialEq for #name #lifetime {
            fn eq(&self, other: &Self) -> bool {
                ::std::ptr::eq(self as *const #name, other as *const #name)
            }
        }

        #[automatically_derived]
        impl #lifetime ::std::cmp::Eq for #name #lifetime {}
    })
    .into()
}

/// Derives [`Hash`] based on memory addresses.
///
/// By deriving [`AddressHash`], the [`Hash`] trait will automatically be
/// implemented for the given type. The implementation will use a given instance's address
/// in memory in when performing hashing.
///
/// Note that since implementations of Hash and Eq must agree, it is recommended to derive both
/// [`AddressHash`] and [`AddressEq`] together.
/// ```rust
/// use address_cmp::{AddressEq, AddressHash};
/// use std::collections::HashSet;
///
/// #[derive(AddressHash, AddressEq)]
/// enum Animal {
///     Bat,
///     Cat,
/// }
///
/// let mut set = HashSet::new();
/// let a1 = Animal::Bat;
/// let a2 = Animal::Cat;
/// let a3 = Animal::Bat;
///
/// set.insert(a1);
/// set.insert(a2);
/// set.insert(a3);
///
/// assert_eq!(set.len(), 3);
/// ```
#[proc_macro_derive(AddressHash)]
pub fn address_hash(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let lifetime = &input.generics;
    (quote! {
        #[automatically_derived]
        impl #lifetime ::std::hash::Hash for #name #lifetime {
            fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
                (self as *const #name).hash(state)
            }
        }
    })
    .into()
}

/// Derives [`PartialOrd`] and [`Ord`] based on memory addresses.
///
/// By deriving [`AddressOrd`], the [`PartialOrd`] and [`Ord`] traits will automatically be
/// implemented for the given type. The implementations will use a given instance's address
/// in memory in when performing comparison.
///
/// This form of comparison can be useful in certain cases, like implementing ordering based
/// on position in a slice, but can also produce unexpected results depending on how the compiler
/// chooses to place data in memory.
///
/// Since implementing [`PartialOrd`] on a type requires that the type also implements [`PartialEq`]
/// and implementations of Hash and Eq must agree, it is recommended to derive both
/// [`AddressOrd`] and [`AddressEq`] together.
/// ```rust
/// use address_cmp::{AddressEq, AddressOrd};
///
/// #[derive(AddressEq, AddressOrd, Debug)]
/// struct Person {
///     pub age: u8,
///     pub name: String,
/// }
///
/// let p1 = Person { age: 22, name: "Mercutio".into() };
/// let p2 = Person { age: 22, name: "Mercutio".into() };
/// let p3 = Person { age: 21, name: "Benvolio".into() };
///
/// let friends = vec![p1, p2, p3];
///
/// assert!(friends[0] < friends[1]);
/// assert!(friends[1] < friends[2]);
/// assert_eq!(friends[0], friends[0]);
/// ```
#[proc_macro_derive(AddressOrd)]
pub fn address_ord(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let lifetime = &input.generics;
    (quote! {
        #[automatically_derived]
        impl #lifetime ::std::cmp::Ord for #name #lifetime {
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                (self as *const #name).cmp(&(other as *const #name))
            }
        }

        #[automatically_derived]
        impl #lifetime ::std::cmp::PartialOrd for #name #lifetime {
            fn partial_cmp(&self, other: &Self) -> ::std::option::Option<::std::cmp::Ordering> {
                ::std::option::Option::Some(self.cmp(other))
            }
        }
    })
    .into()
}
