<div align="center">

# Act.rs Tokio

[![Crates.io](https://img.shields.io/crates/v/act_rs_tokio)](https://crates.io/crates/act_rs_tokio)
[![License](https://img.shields.io/badge/license-MIT%2FApache-blue)](#license)
[![Downloads](https://img.shields.io/crates/d/act_rs_tokio)](https://crates.io/crates/act_rs_tokio)
[![Docs](https://docs.rs/act_rs_tokio/badge.svg)](https://docs.rs/act_rs_tokio/latest/act_rs_tokio/)
[![Twitch Status](https://img.shields.io/twitch/status/coruscateor)](https://www.twitch.tv/coruscateor)

[X](https://twitter.com/Coruscateor) | 
[Twitch](https://www.twitch.tv/coruscateor) | 
[Youtube](https://www.youtube.com/@coruscateor) | 
[Mastodon](https://mastodon.social/@Coruscateor) | 
[GitHub](https://github.com/coruscateor) | 
[GitHub Sponsors](https://github.com/sponsors/coruscateor)

Act.rs Tokio is a minimal Tokio oriented actor framework.

</div>

<br />

## What Is An Actor?

An actor is an object that runs in its own thread or task. You would usually communicate with it via channels.

<br />

## Related Actor Crates

You can also create task based actors:

This crate uses types from [Act.rs](https://crates.io/crates/act_rs).

See also: [Act.rs smol](https://crates.io/crates/act_rs_smol)

<br />

## An Example

Here's the first example in the Act.rs readme adapted to implement a macro generated actor:

<br />

```rust

    //Adapted from the std TwoPlusTwoActorState ThreadActor test (in Act.rs (act_rs)). 

    use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

    use act_rs_tokio::impl_task_actor;

    //Need for impl_task_actor:

    use act_rs::impl_pre_and_post_run_async;

    use pastey::paste;

    use tokio::task::JoinHandle;

    struct TwoPlusTwoActorState
    {

        number: u32,
        client_sender: UnboundedSender<String>

    }

    impl TwoPlusTwoActorState
    {

        pub fn new(client_sender: UnboundedSender<String>) -> Self
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

                if let Err(_) = self.client_sender.send(message)
                {

                    return false;

                }

                return true;

            }

            false
            
        }
        
    }

    impl_task_actor!(TwoPlusTwoActor);

    #[tokio::main(flavor = "multi_thread", worker_threads = 2)]
    async fn main()
    {

        let (sender, mut receiver) = unbounded_channel();

        TwoPlusTwoActor::spawn(TwoPlusTwoActorState::new(sender));

        let res = receiver.recv().await.expect("Error: Message not delivered");

        println!("{}", res);

    }

```

<br />

## Examples

- [Req It](https://github.com/coruscateor/req_it)

- [Escape It](https://github.com/coruscateor/escape_it)

- [Mapage](https://github.com/coruscateor/mapage)

- [Mapage Types Viewer](https://github.com/coruscateor/mapage_types_viewer)

<br />

## Todo

- Add more documentation
- Add examples
- Add more tests
- Cleanup the code

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


