

use smol::{Executor, Task};

use act_rs::ActorStateAsync;

///
/// A task based actor.
/// 
pub struct TaskActor
{
}

impl TaskActor
{

    pub fn spawn<ST>(state: ST, ex: &Executor)
        where ST: ActorStateAsync + Send + 'static
    {
        
        let task = ex.spawn(async move {
    
            TaskActor::run(state).await;

        });

        task.detach();

    }

    pub fn spawn_attached<ST>(state: ST, ex: &Executor) -> Task<()>
        where ST: ActorStateAsync + Send + 'static
    {
        
        ex.spawn(async move {
    
            TaskActor::run(state).await;

        })

    }

    async fn run<ST>(mut state: ST)
        where ST: ActorStateAsync + Send + 'static
    {

        let mut proceed = true; 
        
        if state.pre_run_async().await
        {

            while proceed
            {
                
                proceed = state.run_async().await;
    
            }

        }
        
        state.post_run_async().await;

    }

}
