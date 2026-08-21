use adar::prelude::*;
use std::time::Duration;

#[StateEnum(context = i32)]
enum States {
    State1,
    State2,
}

impl Machine for States {}

impl State for State1 {
    fn on_update(
        &mut self,
        _args: Option<&mut Self::Args>,
        context: &mut Self::Context,
    ) -> Option<Self::States> {
        *context += 1;
        println!("State1 update #{}", context);
        if *context < 3 {
            std::thread::sleep(Duration::from_millis(500));
            Some(State2.into())
        } else {
            None // run() returns after on_update() returns None
        }
    }
}

impl State for State2 {
    fn on_update(
        &mut self,
        _args: Option<&mut Self::Args>,
        _context: &mut Self::Context,
    ) -> Option<Self::States> {
        println!("State2 update");
        std::thread::sleep(Duration::from_millis(500));
        Some(State1.into())
    }
}

fn main() {
    let mut sm = StateMachine::new_context(State1, 0);
    sm.run();
}
