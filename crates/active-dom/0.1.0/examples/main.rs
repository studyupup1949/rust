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
