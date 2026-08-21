use adar::prelude::*;

#[StateEnum]
#[derive(Debug)]
enum ForkA {
    StateA(u32),
    EndState,
}

#[StateEnum]
#[derive(Debug)]
enum ForkB {
    StateB(u32),
    EndState,
}

#[StateEnum]
#[derive(Debug)]
enum MyState {
    StateAB {
        a: StateMachine<ForkA>,
        b: StateMachine<ForkB>,
    },
    EndState,
}

impl Machine for MyState {}
impl State for StateAB {
    fn on_update(
        &mut self,
        _args: Option<&mut Self::Args>,
        _context: &mut Self::Context,
    ) -> Option<Self::States> {
        self.a.update();
        self.b.update();

        // Check if both branches finished
        (self.a.is_finished() && self.b.is_finished()).then_some(EndState.into())
    }
}

impl Machine for ForkA {}
impl State for StateA {
    fn on_update(
        &mut self,
        _args: Option<&mut Self::Args>,
        _context: &mut Self::Context,
    ) -> Option<Self::States> {
        self.0 += 1;
        println!("StateA({})", self.0);

        (self.0 >= 6).then_some(EndState.into())
    }
}
impl Machine for ForkB {}
impl State for StateB {
    fn on_update(
        &mut self,
        _args: Option<&mut Self::Args>,
        _context: &mut Self::Context,
    ) -> Option<Self::States> {
        self.0 += 1;
        println!("StateB({})", self.0);

        (self.0 >= 3).then_some(EndState.into())
    }
}

fn main() {
    let mut sm = StateMachine::new(StateAB {
        a: StateMachine::new(StateA(0)),
        b: StateMachine::new(StateB(0)),
    });

    while !sm.is_finished() {
        sm.update();
    }
}
