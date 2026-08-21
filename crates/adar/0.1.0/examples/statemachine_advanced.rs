use adar::prelude::*;
use std::{
    fmt::Debug,
    time::{Duration, Instant},
};

#[derive(Debug)]
struct Context<T>
where
    T: Debug,
{
    // Contents of the context can be accessed from all states.
    transitions: u32,
    // Anything that implements Debug, will be printed at each callback
    _demo: T,
}

// Adding generic parameters to the context
#[StateEnum(context=for<T> Context<T> where T: Debug)]
// Derive always have to come after the #[StateEnum] macro
#[derive(Debug)]
enum States {
    // You can store variables inside the states
    CountState(u32),
    ContinueCountState(u32),
    // You can even name them...
    DurationState { start: Instant },
    ExitState,
}

impl<T> Machine<T> for States
where
    T: Debug,
{
    fn on_transition(&mut self, new_state: &Self::States, _context: &mut Self::Context) {
        println!("State transition: {:?} -> {:?}", self, new_state);
    }
}

// You can transition the inner states like this
impl From<&mut CountState> for ContinueCountState {
    fn from(value: &mut CountState) -> Self {
        Self(value.0 + 10)
    }
}

// Generic arguments for the context need to be specified this way...
impl<T> State<T> for CountState
where
    T: Debug,
{
    fn on_enter(&mut self, _args: Option<&mut Self::Args>, context: &mut Self::Context) {
        context.transitions += 1;
        println!("CountState::on_enter({:?})", context);
    }
    fn on_update(
        &mut self,
        _args: Option<&mut Self::Args>,
        context: &mut Self::Context,
    ) -> Option<Self::States> {
        self.0 += 1;
        println!("CountState::on_update({:?}) count={}", context, self.0);
        // You can make decisions based on the context or state variables
        if self.0 >= 5 {
            Some(ContinueCountState::from(self).into())
        } else {
            None
        }
    }
    fn on_leave(&mut self, _args: Option<&mut Self::Args>, context: &mut Self::Context) {
        println!("CountState::on_leave({:?})", context);
    }
}

impl<T> State<T> for ContinueCountState
where
    T: Debug,
{
    fn on_enter(&mut self, _args: Option<&mut Self::Args>, context: &mut Self::Context) {
        context.transitions += 1;
        println!("ContinueCountState::on_enter({:?})", context);
    }
    fn on_update(
        &mut self,
        _args: Option<&mut Self::Args>,
        context: &mut Self::Context,
    ) -> Option<Self::States> {
        self.0 += 1;
        println!(
            "ContinueCountState::on_update({:?}) count={}",
            context, self.0
        );
        if self.0 >= 20 {
            Some(
                DurationState {
                    start: Instant::now(),
                }
                .into(),
            )
        } else {
            None
        }
    }
    fn on_leave(&mut self, _args: Option<&mut Self::Args>, context: &mut Self::Context) {
        println!("ContinueCountState::on_leave({:?})", context);
    }
}

impl<T> State<T> for DurationState
where
    T: Debug,
{
    fn on_enter(&mut self, _args: Option<&mut Self::Args>, context: &mut Self::Context) {
        context.transitions += 1;
        println!("DurationState::on_enter({:?})", context);
    }
    fn on_update(
        &mut self,
        _args: Option<&mut Self::Args>,
        context: &mut Self::Context,
    ) -> Option<Self::States> {
        let now = Instant::now();
        let elapsed = now - self.start;
        println!(
            "DurationState::on_update({:?}) elapsed={}",
            context,
            elapsed.as_secs_f32()
        );
        if elapsed >= Duration::from_secs(3) {
            Some(ExitState.into())
        } else {
            None
        }
    }
    fn on_leave(&mut self, _args: Option<&mut Self::Args>, context: &mut Self::Context) {
        println!("DurationState::on_leave({:?})", context);
    }
}

impl<T> State<T> for ExitState
where
    T: Debug,
{
    fn on_enter(&mut self, _args: Option<&mut Self::Args>, context: &mut Self::Context) {
        context.transitions += 1;
        println!("ExitState::on_enter({:?})", context);
    }
    fn on_update(
        &mut self,
        _args: Option<&mut Self::Args>,
        context: &mut Self::Context,
    ) -> Option<Self::States> {
        println!("ExitState::on_update({:?})", context);
        None
    }
    fn on_leave(&mut self, _args: Option<&mut Self::Args>, context: &mut Self::Context) {
        println!("ExitState::on_leave({:?})", context);
    }
}

fn main() {
    let mut sm = StateMachine::new_context(
        CountState(0),
        Context {
            transitions: 0,
            _demo: "Demo",
        },
    );

    while !matches!(sm.state(), States::ExitState(ExitState)) {
        sm.update();
        std::thread::sleep(Duration::from_millis(100));
    }
}
