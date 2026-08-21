use adar::prelude::*;

#[StateEnum]
enum States {
    State1,
    State2,
}

impl Machine for States {}

impl State for State1 {
    fn on_update(
        &mut self,
        _args: Option<&mut Self::Args>,
        _context: &mut Self::Context,
    ) -> Option<Self::States> {
        println!("State1 update");
        Some(State2.into())
    }
}

impl State for State2 {
    fn on_update(
        &mut self,
        _args: Option<&mut Self::Args>,
        _context: &mut Self::Context,
    ) -> Option<Self::States> {
        println!("State2 update");
        Some(State1.into())
    }
}

fn main() {
    let mut sm = StateMachine::new(State1);
    for _ in 0..6 {
        sm.update();
    }
}
