pub use rusty_timer::Timer;
use acute_ecs::systems::SystemBuilder;
use acute_ecs::systems::schedule::Schedulable;

pub fn update_timer() -> Box<dyn Schedulable> {
    SystemBuilder::new("UpdateTimer")
        .write_resource::<Timer>()
        .build(|_, _, timer, _ | {
            timer.update_delta_time();
            timer.update_fixed_time();
        })
}