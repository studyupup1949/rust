use act_rs::ActorFlow;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use pastey::paste;

use tokio::task::JoinHandle;

use std::{any::Any, panic::AssertUnwindSafe};

use std::sync::Arc;

#[cfg(feature="futures")]
use futures::FutureExt;

use crate::{impl_task_actor, impl_task_actor_build_state, impl_task_actor_build_state_and_catch_unwind, impl_task_actor_build_state_and_catch_unwind_flexible, impl_task_actor_build_state_flexible, impl_task_actor_build_state_with_spawn, impl_task_actor_build_state_with_spawn_catch_unwind, impl_task_actor_build_state_with_spawn_catch_unwind_flexible, impl_task_actor_build_state_with_spawn_flexible, impl_task_actor_catch_unwind, impl_task_actor_catch_unwind_flexible, impl_task_actor_flexible};

struct TestActorState
{

    sender: UnboundedSender<i32>

}

impl TestActorState
{

    pub fn new(sender: UnboundedSender<i32>) -> Self
    {

        Self
        {

            sender

        }

    }

    pub async fn pre_run_async(&mut self) -> bool
    {

        self.sender.send(1).expect("pre_run_async error");

        true

    }

    pub async fn run_async(&mut self) -> bool
    {

        self.sender.send(2).expect("run_async error");

        false

    }

    pub async fn post_run_async(self)
    {

        self.sender.send(3).expect("post_run_async error");

    }
    
}

struct TestActorStateBuilder
{

    sender: UnboundedSender<i32>

}

impl TestActorStateBuilder
{

    pub fn new(sender: UnboundedSender<i32>) -> Self
    {

        Self
        {
            
            sender
        
        }

    }

    pub async fn build_async(self) -> Option<TestActorState>
    {

        self.sender.send(0).expect("build_async error");

        Some(TestActorState::new(self.sender))

    }

}

// ActorFlow

struct TestActorFlowState
{

    sender: UnboundedSender<i32>

}

impl TestActorFlowState
{

    pub fn new(sender: UnboundedSender<i32>) -> Self
    {

        Self
        {

            sender

        }

    }

    pub async fn pre_run_async(&mut self) -> ActorFlow
    {

        self.sender.send(1).expect("pre_run_async error");

        ActorFlow::Proceed

    }

    pub async fn run_async(&mut self) -> ActorFlow
    {

        self.sender.send(2).expect("run_async error");

        ActorFlow::Exit

    }

    pub async fn post_run_async(self)
    {

        self.sender.send(3).expect("post_run_async error");

    }
    
}

struct TestActorFlowStateBuilder
{

    sender: UnboundedSender<i32>

}

impl TestActorFlowStateBuilder
{

    pub fn new(sender: UnboundedSender<i32>) -> Self
    {

        Self
        {
            
            sender
        
        }

    }

    pub async fn build_async(self) -> Option<TestActorFlowState>
    {

        self.sender.send(0).expect("build_async error");

        Some(TestActorFlowState::new(self.sender))

    }

}

struct TestPaincHander();

impl TestPaincHander
{

    pub fn new() -> Self
    {

        Self()

    }

    pub async fn handle_panic(&self, _boxed_panic: Box<dyn Any + Send>)
    {

        println!("oops!");

    }

}

//

async fn without_builder(mut receiver: UnboundedReceiver<i32>)
{

    let res = receiver.recv().await;

    assert_eq!(res, Some(1));

    let res = receiver.recv().await;

    assert_eq!(res, Some(2));

    let res = receiver.recv().await;

    assert_eq!(res, Some(3));

}

async fn with_builder(mut receiver: UnboundedReceiver<i32>)
{

    let res = receiver.recv().await;

    assert_eq!(res, Some(0));

    without_builder(receiver).await;

}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn task_actor()
{

    impl_task_actor!(TestActor);

    let (sender, receiver) = unbounded_channel();

    let state = TestActorState::new(sender);

    TestActor::spawn(state);

    without_builder(receiver).await;

}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn task_actor_build_state()
{

    impl_task_actor_build_state!(TestActor);

    let (sender, receiver) = unbounded_channel();

    let state_builder = TestActorStateBuilder::new(sender);

    TestActor::spawn_and_build_state(state_builder);

    with_builder(receiver).await;

}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn task_actor_build_state_with_spawn()
{

    impl_task_actor_build_state_with_spawn!(TestActor);

    //

    let (sender, receiver) = unbounded_channel();

    let state = TestActorState::new(sender);

    TestActor::spawn(state);

    without_builder(receiver).await;

    //

    let (sender, receiver) = unbounded_channel();

    let state_builder = TestActorStateBuilder::new(sender);

    TestActor::spawn_and_build_state(state_builder);

    with_builder(receiver).await;

}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn task_actor_flexible()
{

    impl_task_actor_flexible!(TestActorFlow);

    let (sender, receiver) = unbounded_channel();

    let state = TestActorFlowState::new(sender);

    TestActorFlow::spawn(state);

    without_builder(receiver).await;

}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn task_actor_build_state_flexible()
{

    impl_task_actor_build_state_flexible!(TestActorFlow);

    let (sender, receiver) = unbounded_channel();

    let state_builder = TestActorFlowStateBuilder::new(sender);

    TestActorFlow::spawn_and_build_state(state_builder);

    with_builder(receiver).await;

}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn task_actor_build_state_with_spawn_flexible()
{

    impl_task_actor_build_state_with_spawn_flexible!(TestActorFlow);

    //

    let (sender, receiver) = unbounded_channel();

    let state = TestActorFlowState::new(sender);

    TestActorFlow::spawn(state);

    without_builder(receiver).await;

    //

    let (sender, receiver) = unbounded_channel();

    let state_builder = TestActorFlowStateBuilder::new(sender);

    TestActorFlow::spawn_and_build_state(state_builder);

    with_builder(receiver).await;

}

//catch_unwind

#[cfg(feature="futures")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn task_actor_catch_unwind()
{

    impl_task_actor_catch_unwind!(TestActor, TestPaincHander);

    let (sender, receiver) = unbounded_channel();

    let state = TestActorState::new(sender);

    let panic_handler = Arc::new(TestPaincHander::new());

    TestActor::spawn_catch_unwind(state, &panic_handler);

    without_builder(receiver).await;

}

#[cfg(feature="futures")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn task_actor_build_state_and_catch_unwind()
{

    impl_task_actor_build_state_and_catch_unwind!(TestActor, TestPaincHander);

    let (sender, receiver) = unbounded_channel();

    let state_builder = TestActorStateBuilder::new(sender);

    let panic_handler = Arc::new(TestPaincHander::new());

    TestActor::spawn_build_state_and_catch_unwind(state_builder, &panic_handler);

    with_builder(receiver).await;

}

#[cfg(feature="futures")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn task_actor_build_state_with_spawn_catch_unwind()
{

    impl_task_actor_build_state_with_spawn_catch_unwind!(TestActor, TestPaincHander);

    let panic_handler = Arc::new(TestPaincHander::new());

    //

    let (sender, receiver) = unbounded_channel();

    let state = TestActorState::new(sender);

    TestActor::spawn_catch_unwind(state, &panic_handler);

    without_builder(receiver).await;

    //

    let (sender, receiver) = unbounded_channel();

    let state_builder = TestActorStateBuilder::new(sender);

    TestActor::spawn_build_state_and_catch_unwind(state_builder, &panic_handler);

    with_builder(receiver).await;

}

//flexible catch_unwind

#[cfg(feature="futures")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn task_actor_catch_unwind_flexible()
{

    impl_task_actor_catch_unwind_flexible!(TestActorFlow, TestPaincHander);

    let (sender, receiver) = unbounded_channel();

    let state = TestActorFlowState::new(sender);

    let panic_handler = Arc::new(TestPaincHander::new());

    TestActorFlow::spawn_catch_unwind(state, &panic_handler);

    without_builder(receiver).await;

}

#[cfg(feature="futures")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn task_actor_build_state_and_catch_unwind_flexible()
{

    impl_task_actor_build_state_and_catch_unwind_flexible!(TestActorFlow, TestPaincHander);

    let (sender, receiver) = unbounded_channel();

    let state_builder = TestActorFlowStateBuilder::new(sender);

    let panic_handler = Arc::new(TestPaincHander::new());

    TestActorFlow::spawn_build_state_and_catch_unwind(state_builder, &panic_handler);

    with_builder(receiver).await;

}

#[cfg(feature="futures")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn task_actor_build_state_with_spawn_catch_unwind_flexible()
{

    impl_task_actor_build_state_with_spawn_catch_unwind_flexible!(TestActorFlow, TestPaincHander);

    let panic_handler = Arc::new(TestPaincHander::new());

    //

    let (sender, receiver) = unbounded_channel();

    let state = TestActorFlowState::new(sender);

    TestActorFlow::spawn_catch_unwind(state, &panic_handler);

    without_builder(receiver).await;

    //

    let (sender, receiver) = unbounded_channel();

    let state_builder = TestActorFlowStateBuilder::new(sender);

    let panic_handler = Arc::new(TestPaincHander::new());

    TestActorFlow::spawn_build_state_and_catch_unwind(state_builder, &panic_handler);

    with_builder(receiver).await;

}

