# actix_handler_macro
![Rust](https://github.com/yamadapc/actix-handler-macro/workflows/Rust/badge.svg)
[![Docs](https://docs.rs/actix-handler-macro/badge.svg)](https://docs.rs/actix-handler-macro)
- - -

Helper macros for using Actix.

```rust
use actix::Message;
use actix_handler_macro::{actix_handler, Actor};

#[derive(Actor)]
struct Example;
type ExampleContext = actix::Context<Example>;

#[derive(Message)]
#[rtype(result = "String")]
struct Greeting {
    name: String,
}

#[actix_handler]
impl Example {
    fn greet(&self, message: Greeting, _ctx: &ExampleContext) -> String {
        format!("Hello {}", message.name).to_string()
    }
}
```
actor_derive;
expand_addr;
expand_impl_handlers;
expand_method_handlers;
options;
utils;

expand_impl_handlers::expand_item_impl;
options::{parse_options, Options};
proc_macro::TokenStream;
quote::quote;
syn::{parse_macro_input, AttributeArgs, DeriveInput, Item};
utils::compilation_error;

Allows writing Actix actors with impl blocks.

## Motivation
Actix can be quite verbose when declaring handlers:

```rust
use actix::Actor;
use actix::Context;
use actix::Handler;
use actix::Message;

struct Example;

impl Actor for Example {
    type Context = Context<Example>;
}

#[derive(Message)]
#[rtype(result = "String")]
struct Greeting { name: String }

impl Handler<Greeting> for Example {
    type Result = String;

    fn handle(&mut self, msg: Greeting, ctx: &mut Context<Self>) -> Self::Result {
        unimplemented!()
    }
}
```

`actix_derive_macro` reads an `impl` block and generates a bit of this code for each method.

## Usage
```rust
use actix_handler_macro::{Actor, actix_handler};
use actix::{Actor, Addr, Context, Message, System};


#[derive(Actor)]
struct Example;

type ExampleContext = Context<Example>;

#[derive(Clone, Message)]
#[rtype(result = "String")]
struct Greeting { name: String }

#[actix_handler]
impl Example {
    fn greet(&self, message: Greeting, _ctx: &ExampleContext) -> String {
        format!("Hello {}", message.name).to_string()
    }
}

fn example_usage() {
    let mut sys = System::new("actix-test-runtime");
    let addr: Addr<Example> = Example {}.start();

    sys.block_on(async move {
        let greeting = Greeting { name: "you".to_string() };
        // `Example` has its handler impl block
        let result = addr.send(greeting.clone()).await.ok().unwrap();
        assert_eq!(result, "Hello you");

        // An Addr trait is also expanded:
        let result = addr.greet(greeting.clone()).await.ok().unwrap();
        assert_eq!(result, "Hello you")
    });
}
```

This will expand a `Handler<Greeting>` impl for each method in Example.

## Actor `...Addr` trait
It'll also output a trait `GreetingAddr` and its implementation for `Addr<Example>` with
convenience methods:

```ignore
// Example output
trait GreetingAddr {
    fn greet(self: &Self, msg: Greeting) -> actix::prelude::Request<Example, Greeting>;
}
```

## RecipientRequest

Optionally, the trait can use a `actix::Recipient` and return a `actix::RecipientRequest`.

```rust
use actix::{Message};
use actix_handler_macro::{actix_handler, Actor};

#[derive(Actor)]
struct Example;
type ExampleContext = actix::Context<Example>;
#[derive(Message)]
#[rtype(result = "String")]
struct Greeting { name: String }

#[actix_handler(use_recipient)]
impl Example {
    fn greet(&self, message: Greeting, _ctx: &ExampleContext) -> String {
        format!("Hello {}", message.name).to_string()
    }
}
```

`#[actix_handler(use_recipient)]` will expand the `GreetingAddr` trait as:

```skip
// Example output
trait GreetingAddr {
    fn greet(self: &Self, msg: Greeting) -> actix::RecipientRequest<Greeting>;
}
```

A mock of the actor could be implemented as:
```skip
use actix::Message;
use actix_handler_macro::Actor;

#[derive(Actor)]
struct Example;

#[derive(Message)]
#[rtype(result = "String")]
struct Greeting;

#[derive(Actor)]
struct ExampleMock {
    mocker: actix::Addr<actix::actors::mocker::Mocker<Example>>,
}

impl GreetingAddr for ExampleMock {
    fn greet(self: &Self, msg: Greeting) -> actix::prelude::RecipientRequest<Greeting> {
        self.mocker.clone().recipient().send(msg)
    }
}
```
