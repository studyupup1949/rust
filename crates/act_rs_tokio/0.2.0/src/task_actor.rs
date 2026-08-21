//use async_trait::async_trait;
//use futures::Future;

use tokio::task::{self, JoinHandle, spawn_blocking, JoinError};
//use futures::{executor::block_on, FutureExt};
use std::{any::Any, marker::PhantomData, panic::{AssertUnwindSafe, UnwindSafe}, sync::Arc};

use act_rs::{ActorStateAsync, ActorStateBuilderAsync, ActorStateBuilderUnwindSafeAsync, ActorStateUnwindSafeAsync, AsyncPanicHandler};

#[cfg(feature="futures")]
use futures::FutureExt;

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
    pub fn spawn<ST>(state: ST) -> JoinHandle<()>
        where ST: ActorStateAsync + Send + 'static
    {
        
        tokio::spawn(async move
        {
    
            TaskActor::run(state).await;

        })

    }

    ///
    /// Spawn a new actor and construct its state with the provided state builder in its task.
    /// 
    pub fn spawn_and_build_state<ST, STB>(state_builder: STB) -> JoinHandle<()>
        where ST: ActorStateAsync + Send + 'static,
              STB: ActorStateBuilderAsync<ST> + Send + 'static
    {
        
        tokio::spawn(async move
        {

            if let Some(state) = state_builder.build_async().await
            {

                TaskActor::run(state).await;

            }      

        })

    }

    #[cfg(feature="futures")]
    pub fn spawn_catch_unwind<ST, PH>(state: ST, panic_handler: &Arc<PH>) -> JoinHandle<()>
        where ST: ActorStateAsync + UnwindSafe + Send + 'static, //+ UnwindSafe //ActorStateUnwindSafeAsync
              PH: AsyncPanicHandler + 'static
    {

        let panic_handler_clone = panic_handler.clone();
        
        tokio::spawn(async move
        {
    
            let future = TaskActor::run_catch_unwind(state);

            let result = future.catch_unwind().await; //future.catch_unwind().await;

            if let Err(err) = result
            {

                panic_handler_clone.handle_panic(err).await;

            }

        })

    }

    /*
    pub fn spawn_catch_unwind_async_fn<ST, F>(state: ST, err_fn: &Arc<F>) -> JoinHandle<()>
        where ST: ActorStateUnwindSafeAsync + Send + 'static, //+ UnwindSafe //ActorStateUnwindSafeAsync
              F: AsyncFn(Box<dyn Any + Send>) + Send + Sync + 'static
    {

        let err_fn_clone = err_fn.clone();
        
        tokio::spawn(async move
        {
    
            let future = TaskActor::run_catch_unwind(state);

            let result = future.catch_unwind().await; //future.catch_unwind().await;

            if let Err(err) = result
            {

                //Can't make AsyncFnOnce Send.

                err_fn_clone(err).await;

            }

        })

    }
    */

    #[cfg(feature="futures")]
    pub fn spawn_build_state_and_catch_unwind<ST, STB, PH>(state_builder: STB, panic_handler: &Arc<PH>) -> JoinHandle<()>
        where ST: ActorStateAsync + UnwindSafe + Send + 'static,
              STB: ActorStateBuilderAsync<ST> + UnwindSafe + Send + 'static,
              PH: AsyncPanicHandler + 'static
    {
        
        let panic_handler_clone = panic_handler.clone();

        tokio::spawn(async move
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

        })

    }

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
