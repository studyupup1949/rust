<div align="center">

# Act.rs smol

[![Crates.io](https://img.shields.io/crates/v/act_rs_smol)](https://crates.io/crates/act_rs_smol)
[![License](https://img.shields.io/badge/license-MIT%2FApache-blue)](#license)
[![Downloads](https://img.shields.io/crates/d/act_rs_smol)](https://crates.io/crates/act_rs_smol)
[![Docs](https://docs.rs/act_rs_smol/badge.svg)](https://docs.rs/act_rs_smol/latest/act_rs_smol)
[![Twitch Status](https://img.shields.io/twitch/status/coruscateor)](https://www.twitch.tv/coruscateor)

[X](https://twitter.com/Coruscateor) | 
[Twitch](https://www.twitch.tv/coruscateor) | 
[Youtube](https://www.youtube.com/@coruscateor) | 
[Mastodon](https://mastodon.social/@Coruscateor) | 
[GitHub](https://github.com/coruscateor) | 
[GitHub Sponsors](https://github.com/sponsors/coruscateor)

Act.rs smol is a minimal smol oriented actor framework.

</div>

<br />

## What Is An Actor?

An actor is an object that runs in its own thread or task. You would probably communicate with it via channels.

<br />

## Related Actor Crates

This crate uses types from [Act.rs](https://crates.io/crates/act_rs).

See also: [Act.rs tokio](https://crates.io/crates/act_rs_tokio)

<br />

## An Example

```rust

    //Adapted from the std TwoPlusTwoActorState ThreadActor test (in Act.rs (act_rs)). 

    use smol::channel::{Sender, Receiver, unbounded};

    use smol::Executor;

    use act_rs::ActorState;

    use act_rs_smol::AutoDetachTask;

    //Requires the thread_pool feature.

    use act_rs_smol::ThreadPool;

    use core::num::NonZeroUsize;

    use act_rs_smol::impl_task_actor;

    //Need for impl_task_actor:

    use act_rs::impl_pre_and_post_run_async;

    use pastey::paste;

    struct TwoPlusTwoActorState
    {

        number: u32,
        client_sender: Sender<String>

    }

    impl TwoPlusTwoActorState
    {

        pub fn new(client_sender: Sender<String>) -> Self
        {

            Self
            {

                number: 2,
                client_sender

            }

        }

        impl_pre_and_post_run_async!();

        async fn run_async(&mut self) -> bool
        {

            if self.number < 4
            {

                self.number += 2;

                let message = format!("two plus two is: {}", self.number);

                if let Err(_) = self.client_sender.send(message).await
                {

                    return false;

                }

                return true;

            }

            false
            
        }
        
    }

    impl_task_actor!(TwoPlusTwoActor);

    fn main()
    {

        let nonzero_val = NonZeroUsize::new(2).unwrap();

        ThreadPool::with_threads_and_executor(nonzero_val).block_on(async |this|
        {

            let (sender, receiver) = unbounded();

            TwoPlusTwoActor::spawn(TwoPlusTwoActorState::new(sender), this.executor_ref());

            let res = receiver.recv().await.expect("Error: Message not delivered");

            println!("{}", res);

        });

    }

```

<br />

## Features

| Feature      | Description                                                                                         |
| -----------  | --------------------------------------------------------------------------------------------------- |
| inc_dec      | Enable the IncDec dependency.                                                                       |
| futures-lite | Enable the futures-lite dependency.                                                                 |
| accessorise  | Enable the Accessorise dependency.                                                                  |
| pastey       | Enable the pastey dependency.                                                                       |
| futures      | Enable the futures dependency, also includes std.                                                   |
| async-trait  | Enable the TaskActor struct and the act_rs/async-trait  dependency.                                 |
| thread_pool  | Enable the ThreadPool struct and the inc_dec, accessorise, pastey dependencies. Also depends on std.|

<br />

**Note:** thread_pool requires the futures or the futures-lite features to be enabled.

<br />

## More Examples

- [Req It](https://github.com/coruscateor/req_it)

- [Escape It](https://github.com/coruscateor/escape_it)

- [Mapage](https://github.com/coruscateor/mapage)

- [Mapage Types Viewer](https://github.com/coruscateor/mapage_types_viewer)

<br />

## Todo

- Add more documentation
- Add more examples
- Add mmore tests
- Cleanup the code

## Maybe

- Enable the crate to be used in no_std environments.

<br />

## Coding Style

This project uses a coding style that emphasises the use of white space over keeping the line and column counts as low as possible.

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


