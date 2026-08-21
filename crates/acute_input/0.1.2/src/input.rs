use crate::keyboard::Keyboard;
use crate::mouse::{Mouse};
use acute_window::event::{Event, WindowEvent, ElementState, MouseButton, MouseScrollDelta};

pub struct Input {
    pub keyboard: Keyboard,
    pub mouse: Mouse,
}

impl Input {
    pub fn update(&mut self, events: &Event<()>) {
        self.keyboard.clear();
        self.mouse.clear();

        match events {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(key) = input.virtual_keycode {
                            match input.state {
                                ElementState::Pressed => {
                                    self.keyboard.press(key)
                                }

                                ElementState::Released => {
                                    self.keyboard.release(key)
                                }
                            }
                        }
                    }

                    WindowEvent::MouseInput { button, .. } => {
                        self.mouse.toggle(*button);
                    }

                    WindowEvent::MouseWheel { delta, .. } => {
                        match delta {
                            MouseScrollDelta::LineDelta(x, y) => {
                                self.mouse.update_scroll((*x, *y));
                            }

                            // TODO: Do something with that?
                            MouseScrollDelta::PixelDelta(_) => {}
                        }
                    }

                    WindowEvent::CursorMoved { position, .. } => {
                        self.mouse.update_position(
                            (position.x as f32, position.x as f32)
                        );
                    }

                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn new() -> Self {
        Self {
            keyboard: Keyboard::new(),
            mouse: Mouse::new(),
        }
    }
}