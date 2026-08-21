use winit::dpi::LogicalSize;
use winit::window::{Fullscreen, Window, WindowBuilder};
use winit::event_loop::EventLoop;


pub struct WindowDescriptor {
    title: String,
    size: LogicalSize<u32>,
    fullscreen: Option<Fullscreen>,
    resizable: bool,
}

impl Default for WindowDescriptor {
    fn default() -> Self {
        Self {
            title: "Acute".to_string(),
            size: LogicalSize { width: 1280, height: 720 },
            fullscreen: None,
            resizable: false
        }
    }
}

pub struct WinitWindow;

impl WinitWindow {
    pub fn new(window_desc: WindowDescriptor) -> (Window, EventLoop<()>) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(window_desc.title)
            .with_inner_size(window_desc.size)
            .with_fullscreen(window_desc.fullscreen)
            .with_resizable(window_desc.resizable)
            .build(&event_loop).unwrap();

        (window, event_loop)
    }

    pub fn new_headless() -> EventLoop<()> {
        let event_loop = EventLoop::new();
        event_loop
    }
}