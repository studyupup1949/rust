//! # actix_handler_macro
//! Helper macros for using Actix.
//!
//! ```rust
//! use actix::Message;
//! use actix_handler_macro::{actix_handler, Actor};
//!
//! #[derive(Actor)]
//! struct Example;
//! type ExampleContext = actix::Context<Example>;
//!
//! #[derive(Message)]
//! #[rtype(result = "String")]
//! struct Greeting {
//!     name: String,
//! }
//!
//! #[actix_handler]
//! impl Example {
//!     fn greet(&self, message: Greeting, _ctx: &ExampleContext) -> String {
//!         format!("Hello {}", message.name).to_string()
//!     }
//! }
//! ```
mod actor_derive;
mod expand_addr;
mod expand_impl_handlers;
mod expand_method_handlers;
mod options;
mod utils;

use expand_impl_handlers::expand_item_impl;
use options::{parse_options, Options};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, DeriveInput, Item};
use utils::compilation_error;

/// Allows writing Actix actors with impl blocks.
///
/// ## Motivation
/// Actix can be quite verbose when declaring handlers:
///
/// ```rust
/// use actix::Actor;
/// use actix::Context;
/// use actix::Handler;
/// use actix::Message;
///
/// struct Example;
///
/// impl Actor for Example {
///     type Context = Context<Example>;
/// }
///
/// #[derive(Message)]
/// #[rtype(result = "String")]
/// struct Greeting { name: String }
///
/// impl Handler<Greeting> for Example {
///     type Result = String;
///
///     fn handle(&mut self, msg: Greeting, ctx: &mut Context<Self>) -> Self::Result {
///         unimplemented!()
///     }
/// }
/// ```
///
/// `actix_derive_macro` reads an `impl` block and generates a bit of this code for each method.
///
/// ## Usage
/// ```rust
/// use actix_handler_macro::{Actor, actix_handler};
/// use actix::{Actor, Addr, Context, Message, System};
///
///
/// #[derive(Actor)]
/// struct Example;
///
/// type ExampleContext = Context<Example>;
///
/// #[derive(Clone, Message)]
/// #[rtype(result = "String")]
/// struct Greeting { name: String }
///
/// #[actix_handler]
/// impl Example {
///     fn greet(&self, message: Greeting, _ctx: &ExampleContext) -> String {
///         format!("Hello {}", message.name).to_string()
///     }
/// }
///
/// fn example_usage() {
///     let mut sys = System::new("actix-test-runtime");
///     let addr: Addr<Example> = Example {}.start();
///
///     sys.block_on(async move {
///         let greeting = Greeting { name: "you".to_string() };
///         // `Example` has its handler impl block
///         let result = addr.send(greeting.clone()).await.ok().unwrap();
///         assert_eq!(result, "Hello you");
///
///         // An Addr trait is also expanded:
///         let result = addr.greet(greeting.clone()).await.ok().unwrap();
///         assert_eq!(result, "Hello you")
///     });
/// }
/// ```
///
/// This will expand a `Handler<Greeting>` impl for each method in Example.
///
/// ## Actor `...Addr` trait
/// It'll also output a trait `GreetingAddr` and its implementation for `Addr<Example>` with
/// convenience methods:
///
/// ```ignore
/// // Example output
/// trait GreetingAddr {
///     fn greet(self: &Self, msg: Greeting) -> actix::prelude::Request<Example, Greeting>;
/// }
/// ```
///
/// ## RecipientRequest
///
/// Optionally, the trait can use a `actix::Recipient` and return a `actix::RecipientRequest`.
///
/// ```rust
/// use actix::{Message};
/// use actix_handler_macro::{actix_handler, Actor};
///
/// #[derive(Actor)]
/// struct Example;
/// type ExampleContext = actix::Context<Example>;
/// #[derive(Message)]
/// #[rtype(result = "String")]
/// struct Greeting { name: String }
///
/// #[actix_handler(use_recipient)]
/// impl Example {
///     fn greet(&self, message: Greeting, _ctx: &ExampleContext) -> String {
///         format!("Hello {}", message.name).to_string()
///     }
/// }
/// ```
///
/// `#[actix_handler(use_recipient)]` will expand the `GreetingAddr` trait as:
///
/// ```skip
/// // Example output
/// trait GreetingAddr {
///     fn greet(self: &Self, msg: Greeting) -> actix::RecipientRequest<Greeting>;
/// }
/// ```
///
/// A mock of the actor could be implemented as:
/// ```skip
/// use actix::Message;
/// use actix_handler_macro::Actor;
///
/// #[derive(Actor)]
/// struct Example;
///
/// #[derive(Message)]
/// #[rtype(result = "String")]
/// struct Greeting;
///
/// #[derive(Actor)]
/// struct ExampleMock {
///     mocker: actix::Addr<actix::actors::mocker::Mocker<Example>>,
/// }
///
/// impl GreetingAddr for ExampleMock {
///     fn greet(self: &Self, msg: Greeting) -> actix::prelude::RecipientRequest<Greeting> {
///         self.mocker.clone().recipient().send(msg)
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn actix_handler(args: TokenStream, input: TokenStream) -> TokenStream {
    let parsed_args = parse_macro_input!(args as AttributeArgs);
    let options = parse_options(parsed_args);

    let mut parsed_input = parse_macro_input!(input as Item);
    let expanded = expand_actix_handler(options, &mut parsed_input);

    let mut output = TokenStream::from(quote!(#parsed_input));
    output.extend(expanded);
    output
}

fn expand_actix_handler(options: Options, input: &mut Item) -> TokenStream {
    match input {
        Item::Impl(item_impl) => expand_item_impl(options, item_impl),
        _ => compilation_error("#[actix_handler] will only work on 'impl' blocks"),
    }
}

#[proc_macro_derive(Actor, attributes(actor))]
pub fn actor_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    actor_derive::actor::expand(&ast).into()
}

#[proc_macro_derive(Supervised)]
pub fn supervised_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    actor_derive::supervised::expand(&ast).into()
}

#[proc_macro_derive(ArbiterService)]
pub fn arbiter_service_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    actor_derive::arbiter_service::expand(&ast).into()
}
