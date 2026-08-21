use std::num::NonZero;

use std::sync::Arc;

use std::thread::{self, JoinHandle, Scope, ScopedJoinHandle, Thread, available_parallelism};

use std::io::Error;

use accessorise::impl_get_ref;

use inc_dec::IncDecExt;

use smol::Executor;

use smol::channel::{Sender, unbounded};

#[cfg(feature = "futures-lite")]
use futures_lite::future::block_on;

#[cfg(feature = "futures")]
use futures::executor::block_on;

use pastey::paste;

///
/// A basic thread-pool.
/// 
pub struct ThreadPool
{

    join_handles: Vec<JoinHandle<()>>,
    _signal: Sender<()>,
    executor: Arc<Executor<'static>>

}

impl ThreadPool
{

    pub fn new(arc_ex: Arc<Executor<'static>>) -> Result<Self, Error>
    {

        let avalible_parallelism_res = available_parallelism();

        match avalible_parallelism_res
        {

            Ok(val) =>
            {

                let thread_pool = Self::with_threads(arc_ex, val);

                Ok(thread_pool)

            }
            Err(err) =>
            {

                Err(err)

            }
            
        }

    }

    pub fn with_threads(arc_ex: Arc<Executor<'static>>, number: NonZero<usize>) -> Self
    {

        let (_signal, shutdown) = unbounded::<()>();

        let join_handles = {
            
            let mut number_of_threads: usize = number.into();

            let mut new_join_handles = Vec::with_capacity(number_of_threads);
            
            while number_of_threads > 0
            {

                let ex_moved = arc_ex.clone(); 

                let shutdown_moved = shutdown.clone();

                let jh = thread::spawn(move ||
                { 
                    
                    let _ = block_on(ex_moved.run(shutdown_moved.recv())); 
                
                });

                new_join_handles.push(jh);

                number_of_threads.mm();

            }

            new_join_handles

        };

        Self { join_handles, _signal, executor: arc_ex }

    }

    pub fn new_and_executor() -> Result<Self, Error>
    {

        let ex: Executor<'static> = Executor::new();

        let arc_ex = Arc::new(ex);

        let res = Self::new(arc_ex);

        res

    }

    pub fn with_threads_and_executor(number: NonZero<usize>) -> Self
    {

        let ex: Executor<'static> = Executor::new();

        let arc_ex = Arc::new(ex);

        let res = Self::with_threads(arc_ex, number);

        res

    }

    pub fn number_of_threads(&self) -> usize
    {

        self.join_handles.len()

    }

    impl_get_ref!(join_handles, Vec<JoinHandle<()>>);

    impl_get_ref!(executor, Arc<Executor<'static>>);

    /*
    pub fn join(mut self) -> Vec<Result<(), Box<dyn Any + Send>>>
    {
        
        let mut join_results = Vec::new();

        for join_handle in self.join_handles.drain(..)
        {

            join_results.push(join_handle.join());

        }

        join_results

    }
    */

    pub fn take_join_handles(self) -> Vec<JoinHandle<()>>
    {

        self.join_handles

    }

    pub fn block_on<F, T>(&self, func: F) -> T
        where F: AsyncFnOnce(&Self) -> T
    {

        block_on(func(self))

    }

}
