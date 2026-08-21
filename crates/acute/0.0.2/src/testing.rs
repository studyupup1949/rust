use acute::prelude::*;

fn main() {
    let (window, event_loop) = WinitWindow::new(WindowDescriptor::default());
    // let event_loop = WinitWindow::new_headless();
    let mut app = App::builder()
        .with_defaults(window)
        // .with_defaults_headless()
        .add_system(print_test())
        .add_system(print_time())
        .build();

    event_loop.run(move |event, event_loop, mut control_flow| {
        app.run(event, &mut control_flow);
    })
}

fn print_test() -> Box<dyn Schedulable> {
    SystemBuilder::new("TestPrintSystem").build(|_, _, _, _| println!("Test"))
}

fn print_time() -> Box<dyn Schedulable> {
    SystemBuilder::new("PrintTimeSystem")
        .read_resource::<Timer>()
        .build(move |_,  _, timer, _| {
            println!("{:?}", timer.delta_time())
        })
}