# Active DOM

![Rust](https://img.shields.io/badge/Rust-DD3515?style=for-the-badge&logo=rust&logoColor=white)
![Wasm](https://img.shields.io/badge/Wasm-5B48D9?style=for-the-badge&logo=webassembly&logoColor=white)

Reactive wasm web framework in Rust.

## Usage

Add this crate to your `Cargo.toml` file:

```toml
[dependencies]
active_dom = "0.1.0"
```

```rs
// main.rs
use active_dom::{create_signal, mount, DOM};

fn main() {
    mount(|ctx| {
        let count = create_signal(ctx, 1);

        DOM::new("div")
            .child(
                &DOM::new("button")
                    .text("-")
                    .on("click", move |_| count.set(count.get() - 1))
            )
            .dyn_text(ctx, move || count.get().to_string())
            .child(
                &DOM::new("button")
                .text("+")
                .on("click", move |_| count.set(count.get() + 1))
            )
    });
}
```

```bash
trunk serve
```

## Contributing

Contributions are welcome! I would like you to contribute in this project.

## Roadmap

This project is in its early stages, and there are many missing features that need implementation. Check the [Issues](https://github.com/mdmahikaishar/active-dom/issues) section for a list of features, enhancements, and bug fixes that are planned.


## Inspired Of

Greg Johnston's youtube channel and git repo [simple-framework](https://github.com/gbj/simple-framework).


## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/mdmahikaishar/active-dom/LICENSE) file for details.
