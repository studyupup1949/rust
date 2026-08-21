use std::marker::PhantomData;

pub trait StateTypes<P1 = (), P2 = (), P3 = (), P4 = (), P5 = (), P6 = (), P7 = (), P8 = ()> {
    type States;
    type Context;
    type Args;
}

pub trait State<P1 = (), P2 = (), P3 = (), P4 = (), P5 = (), P6 = (), P7 = (), P8 = ()>
where
    Self: StateTypes<P1, P2, P3, P4, P5, P6, P7, P8>,
{
    #[allow(unused_variables)]
    #[inline(always)]
    fn on_enter(&mut self, args: Option<&mut Self::Args>, context: &mut Self::Context) {}

    #[allow(unused_variables)]
    #[inline(always)]
    fn on_update(
        &mut self,
        args: Option<&mut Self::Args>,
        context: &mut Self::Context,
    ) -> Option<Self::States> {
        None
    }

    #[allow(unused_variables)]
    #[inline(always)]
    fn on_leave(&mut self, args: Option<&mut Self::Args>, context: &mut Self::Context) {}
}

pub trait Machine<P1 = (), P2 = (), P3 = (), P4 = (), P5 = (), P6 = (), P7 = (), P8 = ()>
where
    Self: StateTypes<P1, P2, P3, P4, P5, P6, P7, P8>,
{
    #[allow(unused_variables)]
    #[inline(always)]
    fn on_transition(&mut self, new_state: &Self::States, context: &mut Self::Context) {}
    #[allow(unused_variables)]
    #[inline(always)]
    fn on_update(&mut self, context: &mut Self::Context) {}
}

pub struct StateMachine<S, P1 = (), P2 = (), P3 = (), P4 = (), P5 = (), P6 = (), P7 = (), P8 = ()>
where
    S: State<P1, P2, P3, P4, P5, P6, P7, P8>
        + Machine<P1, P2, P3, P4, P5, P6, P7, P8>
        + StateTypes<P1, P2, P3, P4, P5, P6, P7, P8, States = S>,
{
    state: S::States,
    context: S::Context,
    phantom: PhantomData<(P1, P2, P3, P4, P5, P6, P7, P8)>,
}

pub trait UnitType {
    fn unit() -> Self;
}
impl UnitType for () {
    fn unit() -> Self {}
}

impl<S, P1, P2, P3, P4, P5, P6, P7, P8> StateMachine<S, P1, P2, P3, P4, P5, P6, P7, P8>
where
    S: State<P1, P2, P3, P4, P5, P6, P7, P8>
        + Machine<P1, P2, P3, P4, P5, P6, P7, P8>
        + StateTypes<P1, P2, P3, P4, P5, P6, P7, P8, States = S>,
{
    pub fn new_context<S2>(
        state: S2,
        mut context: S::Context,
    ) -> StateMachine<S2::States, P1, P2, P3, P4, P5, P6, P7, P8>
    where
        S2: StateTypes<P1, P2, P3, P4, P5, P6, P7, P8, States = S> + Into<S2::States>,
    {
        let mut state = state.into() as S2::States;
        state.on_enter(None, &mut context);
        StateMachine::<S2::States, P1, P2, P3, P4, P5, P6, P7, P8> {
            state,
            context,
            phantom: PhantomData,
        }
    }

    pub fn new<S2>(state: S2) -> Self
    where
        S2: StateTypes<P1, P2, P3, P4, P5, P6, P7, P8, States = S> + Into<S2::States>,
        S::Context: Default,
    {
        Self::new_context(state, S::Context::default())
    }

    pub fn run_args(&mut self, args: &mut S::Args) {
        while let Some(new_state) = State::on_update(&mut self.state, Some(args), &mut self.context)
        {
            self.transition(new_state);
        }
    }

    pub fn update_args(&mut self, args: &mut S::Args) {
        if let Some(new_state) = State::on_update(&mut self.state, Some(args), &mut self.context) {
            self.transition_args(new_state, Some(args));
        }
    }

    #[inline(always)]
    pub fn transition(&mut self, new_state: impl Into<S>) {
        self.transition_args(new_state, None);
    }

    pub fn transition_args(&mut self, new_state: impl Into<S>, mut args: Option<&mut S::Args>) {
        match args {
            Some(ref mut a) => {
                self.state.on_leave(Some(&mut **a), &mut self.context);
                let new_state = new_state.into();
                self.state.on_transition(&new_state, &mut self.context);
                self.state = new_state;
                self.state.on_enter(Some(a), &mut self.context);
            }
            None => {
                self.state.on_leave(None, &mut self.context);
                let new_state = new_state.into();
                self.state.on_transition(&new_state, &mut self.context);
                self.state = new_state;
                self.state.on_enter(None, &mut self.context);
            }
        }
    }

    pub fn context(&self) -> &S::Context {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut S::Context {
        &mut self.context
    }

    pub fn state(&self) -> &S::States {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut S::States {
        &mut self.state
    }
}

impl<S, P1, P2, P3, P4, P5, P6, P7, P8> StateMachine<S, P1, P2, P3, P4, P5, P6, P7, P8>
where
    S: State<P1, P2, P3, P4, P5, P6, P7, P8>
        + Machine<P1, P2, P3, P4, P5, P6, P7, P8>
        + StateTypes<P1, P2, P3, P4, P5, P6, P7, P8, States = S>,
    S::Args: UnitType,
{
    pub fn update(&mut self) {
        self.update_args(&mut S::Args::unit());
    }
    pub fn run(&mut self) {
        self.run_args(&mut S::Args::unit());
    }
}

impl<S, P1, P2, P3, P4, P5, P6, P7, P8> HasEndState
    for StateMachine<S, P1, P2, P3, P4, P5, P6, P7, P8>
where
    S: State<P1, P2, P3, P4, P5, P6, P7, P8>
        + Machine<P1, P2, P3, P4, P5, P6, P7, P8>
        + StateTypes<P1, P2, P3, P4, P5, P6, P7, P8, States = S>
        + HasEndState,
{
    fn is_finished(&self) -> bool {
        self.state.is_finished()
    }
}

impl<S, P1, P2, P3, P4, P5, P6, P7, P8> std::fmt::Debug
    for StateMachine<S, P1, P2, P3, P4, P5, P6, P7, P8>
where
    S: State<P1, P2, P3, P4, P5, P6, P7, P8>
        + Machine<P1, P2, P3, P4, P5, P6, P7, P8>
        + StateTypes<P1, P2, P3, P4, P5, P6, P7, P8, States = S>,
    S::States: std::fmt::Debug,
    S::Context: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("StateMachine")
            .field("state", &self.state)
            .field("context", &self.context)
            .finish()
    }
}

impl<S, P1, P2, P3, P4, P5, P6, P7, P8> Drop for StateMachine<S, P1, P2, P3, P4, P5, P6, P7, P8>
where
    S: State<P1, P2, P3, P4, P5, P6, P7, P8>
        + Machine<P1, P2, P3, P4, P5, P6, P7, P8>
        + StateTypes<P1, P2, P3, P4, P5, P6, P7, P8, States = S>,
{
    fn drop(&mut self) {
        self.state.on_leave(None, &mut self.context)
    }
}

#[derive(Debug)]
pub struct EndState;

impl StateTypes for EndState {
    type States = ();
    type Context = ();
    type Args = ();
}

impl State for EndState {}

pub trait HasEndState {
    fn is_finished(&self) -> bool;
}

#[cfg(test)]
mod test {
    use crate::{self as adar, prelude::*};
    use once_cell::sync::Lazy;
    use std::sync::{Arc, Mutex};

    #[derive(Eq, PartialEq, Debug)]
    enum MockState {
        A,
        B,
        C,
    }

    type MockContext = u32;
    type MockArgs = u16;
    #[derive(Eq, PartialEq, Debug)]
    enum MockCall {
        OnEnter((Option<MockArgs>, MockContext)),
        OnUpdate((Option<MockArgs>, MockContext)),
        OnLeave((Option<MockArgs>, MockContext)),
    }

    #[derive(Default, Clone)]
    struct Mock(Arc<Mutex<MockInner>>);

    static MOCK: Lazy<Mock> = Lazy::new(Mock::default);

    #[derive(Default)]
    struct MockInner {
        calls: Vec<(MockState, MockCall)>,
        b_transition: Option<Test>,
    }

    impl Mock {
        pub fn push(&self, state: MockState, call: MockCall) {
            self.0.lock().unwrap().calls.push((state, call));
        }

        pub fn take(&self) -> Vec<(MockState, MockCall)> {
            std::mem::take(&mut self.0.lock().unwrap().calls)
        }

        pub fn b_transition(&self, state: Test) {
            self.0.lock().unwrap().b_transition = Some(state);
        }
    }

    #[StateEnum(context=MockContext, args=MockArgs)]
    enum Test {
        A,
        B,
        C,
    }

    impl Machine for Test {}

    impl State for A {
        fn on_enter(&mut self, args: Option<&mut Self::Args>, context: &mut Self::Context) {
            MOCK.push(MockState::A, MockCall::OnEnter((args.cloned(), *context)));
        }

        fn on_update(
            &mut self,
            args: Option<&mut Self::Args>,
            context: &mut Self::Context,
        ) -> Option<Self::States> {
            MOCK.push(MockState::A, MockCall::OnUpdate((args.cloned(), *context)));
            None
        }

        fn on_leave(&mut self, args: Option<&mut Self::Args>, context: &mut Self::Context) {
            MOCK.push(MockState::A, MockCall::OnLeave((args.cloned(), *context)));
        }
    }
    impl State for B {
        fn on_enter(&mut self, args: Option<&mut Self::Args>, context: &mut Self::Context) {
            MOCK.push(MockState::B, MockCall::OnEnter((args.cloned(), *context)));
        }

        fn on_update(
            &mut self,
            args: Option<&mut Self::Args>,
            context: &mut Self::Context,
        ) -> Option<Self::States> {
            MOCK.push(MockState::B, MockCall::OnUpdate((args.cloned(), *context)));
            MOCK.0.lock().unwrap().b_transition.take()
        }

        fn on_leave(&mut self, args: Option<&mut Self::Args>, context: &mut Self::Context) {
            MOCK.push(MockState::B, MockCall::OnLeave((args.cloned(), *context)));
        }
    }
    impl State for C {
        fn on_enter(&mut self, args: Option<&mut Self::Args>, context: &mut Self::Context) {
            MOCK.push(MockState::C, MockCall::OnEnter((args.cloned(), *context)));
        }

        fn on_update(
            &mut self,
            args: Option<&mut Self::Args>,
            context: &mut Self::Context,
        ) -> Option<Self::States> {
            MOCK.push(MockState::C, MockCall::OnUpdate((args.cloned(), *context)));
            None
        }

        fn on_leave(&mut self, args: Option<&mut Self::Args>, context: &mut Self::Context) {
            MOCK.push(MockState::C, MockCall::OnLeave((args.cloned(), *context)));
        }
    }

    #[StateEnum]
    #[derive(Debug)]
    enum TestDerive {
        A2,
    }
    impl Machine for TestDerive {}
    impl State for A2 {}

    #[StateEnum(context = Arc<Mutex<MockInner>>)]
    enum TestWithComplexContext {
        A3,
    }
    impl Machine for TestWithComplexContext {}
    impl State for A3 {}

    #[StateEnum(context = for<T> Option<T> where T: std::fmt::Debug)]
    enum TestWithGenericWithContext {
        A4,
    }
    impl Machine for TestWithGenericWithContext {}
    impl<T> State<T> for A4 where T: std::fmt::Debug {}

    #[test]
    fn test_macro_edge_cases() {
        // Note: Just to make sure they can be constructed
        let sm = StateMachine::new(A2);
        println!("{:?}", sm);
        StateMachine::new_context(A3, Arc::new(Mutex::new(MockInner::default())));
        StateMachine::new_context(A4, Some(()));
    }

    #[test]
    fn test_external_transition_and_update() {
        let mut sm = StateMachine::new_context(A, 0);
        assert_eq!(
            MOCK.take(),
            vec![(MockState::A, MockCall::OnEnter((None, 0)))]
        );
        sm.update_args(&mut 0);
        assert_eq!(
            MOCK.take(),
            vec![(MockState::A, MockCall::OnUpdate((Some(0), 0)))]
        );
        sm.transition(B);
        assert_eq!(
            MOCK.take(),
            vec![
                (MockState::A, MockCall::OnLeave((None, 0))),
                (MockState::B, MockCall::OnEnter((None, 0)))
            ]
        );
        sm.update_args(&mut 0);
        assert_eq!(
            MOCK.take(),
            vec![(MockState::B, MockCall::OnUpdate((Some(0), 0)))]
        );
        sm.transition(C);
        assert_eq!(
            MOCK.take(),
            vec![
                (MockState::B, MockCall::OnLeave((None, 0))),
                (MockState::C, MockCall::OnEnter((None, 0)))
            ]
        );
        sm.update_args(&mut 0);
        assert_eq!(
            MOCK.take(),
            vec![(MockState::C, MockCall::OnUpdate((Some(0), 0)))]
        );
        sm.update_args(&mut 0);
        assert_eq!(
            MOCK.take(),
            vec![(MockState::C, MockCall::OnUpdate((Some(0), 0)))]
        );
        drop(sm);
        assert_eq!(
            MOCK.take(),
            vec![(MockState::C, MockCall::OnLeave((None, 0)))]
        );
    }

    #[test]
    fn test_internal_transition_and_update() {
        let mut sm = StateMachine::new_context(B, 0);
        assert_eq!(
            MOCK.take(),
            vec![(MockState::B, MockCall::OnEnter((None, 0)))]
        );
        sm.update_args(&mut 0);
        assert_eq!(
            MOCK.take(),
            vec![(MockState::B, MockCall::OnUpdate((Some(0), 0)))]
        );
        MOCK.b_transition(C.into());
        sm.update_args(&mut 0);
        assert_eq!(
            MOCK.take(),
            vec![
                (MockState::B, MockCall::OnUpdate((Some(0), 0))),
                (MockState::B, MockCall::OnLeave((Some(0), 0))),
                (MockState::C, MockCall::OnEnter((Some(0), 0)))
            ]
        );
        drop(sm);
        assert_eq!(
            MOCK.take(),
            vec![(MockState::C, MockCall::OnLeave((None, 0)))]
        );
    }
}
