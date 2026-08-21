// This example program may be used for any purpose proprietary or not.

extern crate adi;
extern crate barg;

use adi::screen::{App, Event, Runner};
use barg::*;

pub struct Ctx {
    // Time
    pub time: f32,
    // Frames
    pub frames: u32,
    // Barg Surface
    pub surface: SurfaceInfo,
    // Barg Font
    pub font: Font<'static>,
}

impl Default for Ctx {
    fn default() -> Ctx {
        Ctx {
            time: 0.0,
            frames: 0,
            surface: SurfaceInfo::new(Size(0, 0), None),
            font: Font::new(FONT).expect("Failed to load font!"),
        }
    }
}

fn mode(app: &mut App, ctx: &mut Ctx, _runner: &mut Runner<Ctx>, event: Event, dt: f32) {
    // Your code that runs every frame.
    match event {
        Event::Exit => app.stop(),
        Event::Timestep => {
            ctx.time += dt;
            ctx.frames += 1;
            if ctx.time >= 1.0 {
                if app.wh() != (ctx.surface.size.0, ctx.surface.size.1) {
                    let (w, h) = app.wh();
                    ctx.surface = SurfaceInfo::new(Size(w, h), Some(app.pitch()));
                }

                ctx.time -= 1.0;
                let s = format!(
                    "FPS Example\nVersion {}\n{} FPS",
                    env!("CARGO_PKG_VERSION"),
                    ctx.frames
                );

                app.draw(&mut |pixel_buffer| {
                    let mut surface = LinkSurface::new(&mut ctx.surface, pixel_buffer);

                    surface.clear();
                    surface.text(
                        [255, 255, 255, 255], // White
                        (5.0, 5.0, 36.0),     // Pos. & Size
                        &ctx.font,            // Font
                        &s,                   // String
                    );
                });
                ctx.frames = 0;
            }
        }
        _ => {}
    }
}

fn main() -> Result<(), String> {
    adi::screen::main(mode)
}
