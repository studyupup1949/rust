use acute_ecs::prelude::*;
use acute_ecs::systems::schedule::Builder;

pub struct Scene {
    pub world: World,
    pub schedule: Schedule,
}

impl Scene {
    pub fn new(universe: &Universe, schedule_builder: Option<Builder>) -> Self {
        let world = universe.create_world();

        let schedule = schedule_builder.unwrap_or(Schedule::builder())
            .build();

        Self {
            world,
            schedule
        }
    }

    pub fn update(&mut self, resources: &mut Resources) {
        self.schedule.execute(&mut self.world, resources);
    }
}