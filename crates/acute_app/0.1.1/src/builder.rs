use acute_ecs::prelude::*;
use super::app::App;
use acute_ecs::systems::resource::Resource;
use acute_core::Timer;
use acute_ecs::systems::schedule::Builder;

pub struct AppBuilder {
    pub app: App,
    pub system_builder: Builder,
    pub render_system_builder: Builder,
}

impl AppBuilder {
    pub fn build(mut self) -> App {
        self.app.schedule = self.system_builder.build();
        self.app.render_schedule = self.render_system_builder.build();

        self.app
    }

    pub fn add_resource<T: Resource>(mut self, resource: T) -> Self {
        self.app.resources.insert(resource);
        self
    }

    pub fn add_system(mut self, system: Box<dyn Schedulable>) -> Self {
        self.system_builder = self.system_builder.add_system(system);
        self
    }

    pub fn add_render_system(mut self, render_system: Box<dyn Schedulable>) -> Self {
        self.render_system_builder = self.render_system_builder.add_system(render_system);
        self
    }
}



impl Default for AppBuilder {
    fn default() -> Self {
        Self {
            app: Default::default(),
            system_builder: Default::default(),
            render_system_builder: Default::default()
        }
    }
}