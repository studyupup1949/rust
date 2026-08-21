use act_rs::ActorFlow;

use smol::channel::{Receiver, Sender, unbounded};

use pastey::paste;

use std::num::NonZeroUsize;

use std::{any::Any, panic::AssertUnwindSafe};

use smol::Executor;

use crate::AutoDetachTask;

use std::sync::Arc;

#[cfg(feature="futures")]
use futures::FutureExt;

use crate::{impl_task_actor, impl_task_actor_build_state, impl_task_actor_build_state_and_catch_unwind, impl_task_actor_build_state_and_catch_unwind_flexible, impl_task_actor_build_state_flexible, impl_task_actor_build_state_with_spawn, impl_task_actor_build_state_with_spawn_catch_unwind, impl_task_actor_build_state_with_spawn_catch_unwind_flexible, impl_task_actor_build_state_with_spawn_flexible, impl_task_actor_catch_unwind, impl_task_actor_catch_unwind_flexible, impl_task_actor_flexible};

use crate::ThreadPool;

struct TestActorState
{

    sender: Sender<i32>

}

impl TestActorState
{

    pub fn new(sender: Sender<i32>) -> Self
    {

        Self
        {

            sender

        }

    }

    pub async fn pre_run_async(&mut self) -> bool
    {

        self.sender.send(1).await.unwrap();

        true

    }

    pub async fn run_async(&mut self) -> bool
    {

        self.sender.send(2).await.unwrap();

        false

    }

    pub async fn post_run_async(self)
    {

        self.sender.send(3).await.unwrap();

    }
    
}

struct TestActorStateBuilder
{

    sender: Sender<i32>

}

impl TestActorStateBuilder
{

    pub fn new(sender: Sender<i32>) -> Self
    {

        Self
        {
            
            sender
        
        }

    }

    pub async fn build_async(self) -> Option<TestActorState>
    {

        self.sender.send(0).await.unwrap();

        Some(TestActorState::new(self.sender))

    }

}

// ActorFlow

struct TestActorFlowState
{

    sender: Sender<i32>

}

impl TestActorFlowState
{

    pub fn new(sender: Sender<i32>) -> Self
    {

        Self
        {

            sender

        }

    }

    pub async fn pre_run_async(&mut self) -> ActorFlow
    {

        self.sender.send(1).await.unwrap();

        ActorFlow::Proceed

    }

    pub async fn run_async(&mut self) -> ActorFlow
    {

        self.sender.send(2).await.unwrap();

        ActorFlow::Exit

    }

    pub async fn post_run_async(self)
    {

        self.sender.send(3).await.unwrap();

    }
    
}

struct TestActorFlowStateBuilder
{

    sender: Sender<i32>

}

impl TestActorFlowStateBuilder
{

    pub fn new(sender: Sender<i32>) -> Self
    {

        Self
        {
            
            sender
        
        }

    }

    pub async fn build_async(self) -> Option<TestActorFlowState>
    {

        self.sender.send(0).await.unwrap();

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

async fn without_builder(receiver: Receiver<i32>)
{

    let res = receiver.recv().await;

    assert_eq!(res, Ok(1));

    let res = receiver.recv().await;

    assert_eq!(res, Ok(2));

    let res = receiver.recv().await;

    assert_eq!(res, Ok(3));

}

async fn with_builder(receiver: Receiver<i32>)
{

    let res = receiver.recv().await;

    assert_eq!(res, Ok(0));

    without_builder(receiver).await;

}

fn get_nonzero_2() -> NonZeroUsize
{

    NonZeroUsize::new(2).unwrap()

}

#[test]
fn task_actor()
{

    impl_task_actor!(TestActor);

    ThreadPool::with_threads_and_executor(get_nonzero_2()).block_on(async |this|
    {
        
        let (sender, receiver) = unbounded();

        let state = TestActorState::new(sender);

        TestActor::spawn(state, this.executor_ref());

        without_builder(receiver).await;

    });

}

#[test]
fn task_actor_build_state()
{

    impl_task_actor_build_state!(TestActor);

    ThreadPool::with_threads_and_executor(get_nonzero_2()).block_on(async |this|
    {

        let (sender, receiver) = unbounded();

        let state_builder = TestActorStateBuilder::new(sender);

        TestActor::spawn_and_build_state(state_builder, this.executor_ref());

        with_builder(receiver).await;

    });

}

#[test]
fn task_actor_build_state_with_spawn()
{

    impl_task_actor_build_state_with_spawn!(TestActor);

    ThreadPool::with_threads_and_executor(get_nonzero_2()).block_on(async |this|
    {

        let (sender, receiver) = unbounded();

        let state = TestActorState::new(sender);

        TestActor::spawn(state, this.executor_ref());

        without_builder(receiver).await;

        //

        let (sender, receiver) = unbounded();

        let state_builder = TestActorStateBuilder::new(sender);

        TestActor::spawn_and_build_state(state_builder, this.executor_ref());

        with_builder(receiver).await;

    });


}

#[test]
fn task_actor_flexible()
{

    impl_task_actor_flexible!(TestActorFlow);

    ThreadPool::with_threads_and_executor(get_nonzero_2()).block_on(async |this|
    {

        let (sender, receiver) = unbounded();

        let state = TestActorFlowState::new(sender);

        TestActorFlow::spawn(state, this.executor_ref());

        without_builder(receiver).await;

    });

}

#[test]
fn task_actor_build_state_flexible()
{

    impl_task_actor_build_state_flexible!(TestActorFlow);

    ThreadPool::with_threads_and_executor(get_nonzero_2()).block_on(async |this|
    {

        let (sender, receiver) = unbounded();

        let state_builder = TestActorFlowStateBuilder::new(sender);

        TestActorFlow::spawn_and_build_state(state_builder, this.executor_ref());

        with_builder(receiver).await;

    });

}

#[test]
fn task_actor_build_state_with_spawn_flexible()
{

    impl_task_actor_build_state_with_spawn_flexible!(TestActorFlow);

    ThreadPool::with_threads_and_executor(get_nonzero_2()).block_on(async |this|
    {

        let (sender, receiver) = unbounded();

        let state = TestActorFlowState::new(sender);

        TestActorFlow::spawn(state, this.executor_ref());

        without_builder(receiver).await;

        //

        let (sender, receiver) = unbounded();

        let state_builder = TestActorFlowStateBuilder::new(sender);

        TestActorFlow::spawn_and_build_state(state_builder, this.executor_ref());

        with_builder(receiver).await;

    });

}

//catch_unwind

#[cfg(feature="futures")]
#[test]
fn task_actor_catch_unwind()
{

    ThreadPool::with_threads_and_executor(get_nonzero_2()).block_on(async |this|
    {

        impl_task_actor_catch_unwind!(TestActor, TestPaincHander);

        let (sender, receiver) = unbounded();

        let state = TestActorState::new(sender);

        let panic_handler = Arc::new(TestPaincHander::new());

        TestActor::spawn_catch_unwind(state, &panic_handler, this.executor_ref());

        without_builder(receiver).await;

    });

}

#[cfg(feature="futures")]
#[test]
fn task_actor_build_state_and_catch_unwind()
{

    impl_task_actor_build_state_and_catch_unwind!(TestActor, TestPaincHander);

    ThreadPool::with_threads_and_executor(get_nonzero_2()).block_on(async |this|
    {

        let (sender, receiver) = unbounded();

        let state_builder = TestActorStateBuilder::new(sender);

        let panic_handler = Arc::new(TestPaincHander::new());

        TestActor::spawn_build_state_and_catch_unwind(state_builder, &panic_handler, this.executor_ref());

        with_builder(receiver).await;

    });

}

#[cfg(feature="futures")]
#[test]
fn task_actor_build_state_with_spawn_catch_unwind()
{

    impl_task_actor_build_state_with_spawn_catch_unwind!(TestActor, TestPaincHander);

    ThreadPool::with_threads_and_executor(get_nonzero_2()).block_on(async |this|
    {

        let panic_handler = Arc::new(TestPaincHander::new());

        //

        let (sender, receiver) = unbounded();

        let state = TestActorState::new(sender);

        TestActor::spawn_catch_unwind(state, &panic_handler, this.executor_ref());

        without_builder(receiver).await;

        //

        let (sender, receiver) = unbounded();

        let state_builder = TestActorStateBuilder::new(sender);

        TestActor::spawn_build_state_and_catch_unwind(state_builder, &panic_handler, this.executor_ref());

        with_builder(receiver).await;

    });

}

//flexible catch_unwind

#[cfg(feature="futures")]
#[test]
fn task_actor_catch_unwind_flexible()
{

    impl_task_actor_catch_unwind_flexible!(TestActorFlow, TestPaincHander);

    ThreadPool::with_threads_and_executor(get_nonzero_2()).block_on(async |this|
    {

        let (sender, receiver) = unbounded();

        let state = TestActorFlowState::new(sender);

        let panic_handler = Arc::new(TestPaincHander::new());

        TestActorFlow::spawn_catch_unwind(state, &panic_handler, this.executor_ref());

        without_builder(receiver).await;

    });

}

#[cfg(feature="futures")]
#[test]
fn task_actor_build_state_and_catch_unwind_flexible()
{

    impl_task_actor_build_state_and_catch_unwind_flexible!(TestActorFlow, TestPaincHander);

    ThreadPool::with_threads_and_executor(get_nonzero_2()).block_on(async |this|
    {

        let (sender, receiver) = unbounded();

        let state_builder = TestActorFlowStateBuilder::new(sender);

        let panic_handler = Arc::new(TestPaincHander::new());

        TestActorFlow::spawn_build_state_and_catch_unwind(state_builder, &panic_handler, this.executor_ref());

        with_builder(receiver).await;

    });

}

#[cfg(feature="futures")]
#[test]
fn task_actor_build_state_with_spawn_catch_unwind_flexible()
{

    impl_task_actor_build_state_with_spawn_catch_unwind_flexible!(TestActorFlow, TestPaincHander);

    ThreadPool::with_threads_and_executor(get_nonzero_2()).block_on(async |this|
    {

        let panic_handler = Arc::new(TestPaincHander::new());

        //

        let (sender, receiver) = unbounded();

        let state = TestActorFlowState::new(sender);

        TestActorFlow::spawn_catch_unwind(state, &panic_handler, this.executor_ref());

        without_builder(receiver).await;

        //

        let (sender, receiver) = unbounded();

        let state_builder = TestActorFlowStateBuilder::new(sender);

        let panic_handler = Arc::new(TestPaincHander::new());

        TestActorFlow::spawn_build_state_and_catch_unwind(state_builder, &panic_handler, this.executor_ref());

        with_builder(receiver).await;

    });

}

