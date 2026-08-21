//use async_trait::async_trait;
//use futures::Future;

//use futures::{executor::block_on, FutureExt};

use tokio::task::JoinHandle;

use std::{any::Any, marker::PhantomData, panic::{UnwindSafe, catch_unwind}, sync::Arc};

use act_rs::{ActorState, ActorStateBuilder};

///
/// A blocking thread based actor.
/// 
pub struct BlockingActor
{
}

impl BlockingActor
{

    ///
    /// Spawn a new actor with the provided state.
    /// 
    pub fn spawn<ST>(state: ST) -> JoinHandle<()>
        where ST: ActorState + Send + 'static
    {
        
        tokio::task::spawn_blocking(move ||
        {
    
            BlockingActor::run(state);

        })

    }

    ///
    /// Spawn a new actor and construct its state with the provided state builder in its thread.
    /// 
    pub fn spawn_and_build_state<ST, STB>(state_builder: STB) -> JoinHandle<()>
        where ST: ActorState + Send + 'static,
              STB: ActorStateBuilder<ST> + Send + 'static
    {
        
        tokio::task::spawn_blocking(move ||
        {

            if let Some(state) = state_builder.build()
            {

                BlockingActor::run(state);

            }      

        })

    }

    pub fn spawn_catch_unwind<ST, F>(state: ST, err_fn: &Arc<F>) -> JoinHandle<()>
        where ST: ActorState + Send + UnwindSafe + 'static,
              F: Fn(Box<dyn Any + Send>) + Send + Sync + 'static
    {
        
        let err_fn_clone = err_fn.clone();

        tokio::task::spawn_blocking(move ||
        {

            let result = catch_unwind(||
            {
                    
                BlockingActor::run(state);

            });

            if let Err(err) = result
            {

                err_fn_clone(err);

            }

        })

    }

    pub fn spawn_build_state_and_catch_unwind<ST, STB, F>(state_builder: STB, err_fn: &Arc<F>) -> JoinHandle<()>
        where ST: ActorState + Send + 'static,
              STB: ActorStateBuilder<ST> + Send + UnwindSafe + 'static,
              F: Fn(Box<dyn Any + Send>) + Send + Sync + 'static
    {

        let err_fn_clone = err_fn.clone();
          
        tokio::task::spawn_blocking(move ||
        {

            let result = catch_unwind(||
            {

                if let Some(state) = state_builder.build()
                {

                    BlockingActor::run(state);

                }

            });

            if let Err(err) = result
            {

                err_fn_clone(err);

            }

        })

    }

    pub fn run<ST>(mut state: ST)
        where ST: ActorState + Send + 'static
    {
        
        if state.pre_run()
        {

            let mut proceed = true; 

            while proceed
            {
                
                proceed = state.run();
    
            }

        }

        state.post_run();

    }  
    
}
