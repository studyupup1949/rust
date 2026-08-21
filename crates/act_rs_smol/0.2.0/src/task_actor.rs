#[cfg(feature="futures")]
use std::panic::AssertUnwindSafe;

#[cfg(feature="futures")]
use std::{panic::UnwindSafe, sync::Arc};

#[cfg(feature="futures")]
use act_rs::AsyncPanicHandler;

use smol::Executor;

use act_rs::{ActorStateAsync, ActorStateBuilderAsync};

#[cfg(feature="futures")]
use futures::FutureExt;

use crate::AutoDetachTask;

///
/// A task based actor.
/// 
pub struct TaskActor
{
}

impl TaskActor
{

    ///
    /// Spawn a new actor with the provided state.
    /// 
    pub fn spawn<ST>(state: ST, ex: &Executor) -> AutoDetachTask<()>
        where ST: ActorStateAsync + Send + 'static
    {
        
        let task = ex.spawn(async move
        {
    
            TaskActor::run(state).await;

        });

        AutoDetachTask::new(task)

    }

    ///
    /// Spawn a new actor and construct its state with the provided state builder in its task.
    /// 
    pub fn spawn_and_build_state<ST, STB>(state_builder: STB, ex: &Executor) -> AutoDetachTask<()>
        where ST: ActorStateAsync + Send + 'static,
              STB: ActorStateBuilderAsync<ST> + Send + 'static
    {
        
        let task = ex.spawn(async move
        {

            if let Some(state) = state_builder.build_async().await
            {

                TaskActor::run(state).await;

            }      

        });

        AutoDetachTask::new(task)

    }

    ///
    /// Spawn a new actor with the provided state catching any unwinding panics.
    /// 
    #[cfg(feature="futures")]
    pub fn spawn_catch_unwind<ST, PH>(state: ST, panic_handler: &Arc<PH>, ex: &Executor) -> AutoDetachTask<()>
        where ST: ActorStateAsync + UnwindSafe + Send + 'static,
              PH: AsyncPanicHandler + 'static
    {

        let panic_handler_clone = panic_handler.clone();
        
        let task = ex.spawn(async move
        {
    
            let future = TaskActor::run_catch_unwind(state);

            let result = future.catch_unwind().await; //future.catch_unwind().await;

            if let Err(err) = result
            {

                panic_handler_clone.handle_panic(err).await;

            }

        });

        AutoDetachTask::new(task)

    }

    ///
    /// Spawn a new actor and construct its state with the provided state builder in its task, catching any unwinding panics.
    /// 
    #[cfg(feature="futures")]
    pub fn spawn_build_state_and_catch_unwind<ST, STB, PH>(state_builder: STB, panic_handler: &Arc<PH>, ex: &Executor) -> AutoDetachTask<()>
        where ST: ActorStateAsync + UnwindSafe + Send + 'static,
              STB: ActorStateBuilderAsync<ST> + UnwindSafe + Send + 'static,
              PH: AsyncPanicHandler + 'static
    {
        
        let panic_handler_clone = panic_handler.clone();

        let task = ex.spawn(async move
        {

            match AssertUnwindSafe(state_builder.build_async()).catch_unwind().await
            {

                Ok(opt_state) =>
                {

                    if let Some(state) = opt_state
                    {

                        if let Err(err) = TaskActor::run_catch_unwind(state).catch_unwind().await
                        {

                            panic_handler_clone.handle_panic(err).await;

                        }

                    }

                }
                Err(err) =>
                {

                    panic_handler_clone.handle_panic(err).await;

                }
                
            }

        });

        AutoDetachTask::new(task)

    }

    ///
    /// Call ActorStateAsync methods on the provided state, intended to be used to run an actor.
    /// 
    pub async fn run<ST>(mut state: ST)
        where ST: ActorStateAsync + Send + 'static
    {
        
        if state.pre_run_async().await
        {

            let mut proceed = true; 

            while proceed
            {
                
                proceed = state.run_async().await;
    
            }

        }
        
        state.post_run_async().await;

    }

    ///
    /// Call ActorStateAsync methods on the provided state catching any unwinding panics. Intended to be used to run an actor.
    /// 
    #[cfg(feature="futures")]
    pub async fn run_catch_unwind<ST>(mut state: ST)
        where ST: ActorStateAsync + Send + UnwindSafe + 'static //ActorStateUnwindSafeAsync
    {
        
        if AssertUnwindSafe(state.pre_run_async()).await
        {

            let mut proceed = true; 

            while proceed
            {
                
                proceed = AssertUnwindSafe(state.run_async()).await;
    
            }

        }
        
        AssertUnwindSafe(state.post_run_async()).await;

    }

}
