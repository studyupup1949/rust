use acute_ecs::prelude::*;
use acute_scenes::Scene;
use acute_window::event::{Event as WinitEvent, VirtualKeyCode};
use acute_window::event_loop::{ControlFlow};
use acute_ecs::systems::resource::Resource;
use super::builder::AppBuilder;
use crate::State;
use acute_render_backend::Renderer;
use acute_input::Input;
use acute_window::event::WindowEvent;

pub struct App {
    pub universe: Universe,
    pub resources: Resources,
    pub schedule: Schedule,
    pub render_schedule: Schedule,
    pub scene: Scene,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    pub fn run(&mut self, event: &WinitEvent<()>, control_flow: &mut ControlFlow) {
        if let Some(mut input) = self.resources.get_mut::<Input>() {
            input.update(event);
            if input.keyboard.pressed(VirtualKeyCode::LAlt) && input.keyboard.pressed(VirtualKeyCode::F4) {
                *control_flow = ControlFlow::Exit;
            }
        }
        match event {
            WinitEvent::WindowEvent {event, ..}  => {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        self.schedule.execute(&mut self.scene.world, &mut self.resources);
        self.scene.update(&mut self.resources);
        self.render_schedule.execute(&mut self.scene.world, &mut self.resources);
    }

    pub fn run_with_state<T: State>(&mut self, state: &mut T, event: &WinitEvent<()>, control_flow: &mut ControlFlow) {
        state.update( self);
        state.update_fixed(self);
        self.run(event, control_flow);
    }

    pub fn add_resource<T: Resource>(&mut self, resource: T) {
        self.resources.insert(resource);
    }
}

impl Default for App {
    fn default() -> Self {
        let universe = Universe::new();
        let scene = Scene::new(&universe, None);
        Self {
            universe,
            resources: Default::default(),
            schedule: Schedule::builder().build(),
            render_schedule: Schedule::builder().build(),
            scene,
        }
    }
}