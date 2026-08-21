// This example program may be used for any purpose proprietary or not.

extern crate adi;

#[derive(Default)]
pub struct Ctx;

use adi::screen::{App, Event, Runner};

fn mode(app: &mut App, _ctx: &mut Ctx, _runner: &mut Runner<Ctx>, event: Event, _dt: f32) {
    // Your code that runs every frame.
    match event {
        Event::Exit => app.stop(),
        _ => {}
    }
}

fn main() -> Result<(), String> {
    adi::screen::main(mode)
}
