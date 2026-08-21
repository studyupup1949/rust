/**

    Generates an async oriented actor to be instantiated within an async runtime context.

    The state type that is provided to the produced spawn method must implement:

    ActorStateAsync

    or

    async fn pre_run_async(&mut sel) -> bool;

    async fn run_async(&mut self) -> bool;

    async fn post_run_async(self);

    directly

    Also tokio::task::JoinHandle and paste::paste must be in module scope.

    The latter is a macro from the [paste crate](https://crates.io/crates/paste).

    Works with version 1.0.15 and above.



    The name of the state type is generated from the provided $actor_type.

    As part of the macro output process "State" is appended to the provided $actor_type macro identity parameter, this type is required by the generated spawn method.
    


    The returned bool values from the pre_run_async and run_async method implementations indicate whether or not the actors execution should proceed.

    The post_run_async method is called regardless.

*/
#[macro_export]
macro_rules! impl_task_actor
{

    ($actor_type:ident) =>
    {

        paste!
        {

            pub struct $actor_type
            {
            }

            impl $actor_type
            {

                pub fn spawn(state: [<$actor_type State>]) -> JoinHandle<()>
                {
                    
                    tokio::spawn(async move
                    {

                        $actor_type::run(state).await;

                    })

                }

                async fn run(mut state: [<$actor_type State>])
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

            }
            
        }

    }

}

/**
 * Similar to impl_task_actor, but it also generates a spawn_and_build_state method for building the actor state in another task.
 * 
 * Requires everything that impl_task_actor does, but also that an actor state builder type with a method "build_async" that returns an optional actor state object share the scope of the macro call.
 * 
 * The actor state builder type name consists of the provided actor type name with "StateBuilder" appended.
 * 
 * Note that if build_async returns a None value then none of the run methods are called (including post_run_async).
*/
#[macro_export]
macro_rules! impl_task_actor_build_state
{

    ($actor_type:ident) =>
    {

        paste!
        {

            
            pub struct $actor_type
            {
            }

            impl $actor_type
            {

                pub fn spawn_and_build_state(state_builder: [<$actor_type StateBuilder>]) -> JoinHandle<()>
                {
                    
                    tokio::spawn(async move
                    {

                        let opt_state = state_builder.build_async().await;

                        if let Some(state) = opt_state
                        {

                            $actor_type::run(state).await;

                        }

                    })

                }

                async fn run(mut state: [<$actor_type State>])
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

            }
            
        }

    }

}

#[macro_export]
macro_rules! impl_task_actor_build_state_with_spawn
{

    ($actor_type:ident) =>
    {

        paste!
        {

            
            pub struct $actor_type
            {
            }
            
            impl $actor_type
            {

                pub fn spawn(state: [<$actor_type State>]) -> JoinHandle<()>
                {
                    
                    tokio::spawn(async move
                    {

                        $actor_type::run(state).await;

                    })

                }

                pub fn spawn_and_build_state(state_builder: [<$actor_type StateBuilder>]) -> JoinHandle<()>
                {
                    
                    tokio::spawn(async move
                    {

                        let opt_state = state_builder.build_async().await;

                        if let Some(state) = opt_state
                        {

                            $actor_type::run(state).await;

                        }

                    })

                }

                async fn run(mut state: [<$actor_type State>])
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

            }
            
        }

    }

}

//ActorFlow Compatible

#[macro_export]
macro_rules! impl_task_actor_flexible
{

    ($actor_type:ident) =>
    {

        paste!
        {

            pub struct $actor_type
            {
            }

            impl $actor_type
            {

                pub fn spawn(state: [<$actor_type State>]) -> JoinHandle<()>
                {
                    
                    tokio::spawn(async move
                    {

                        $actor_type::run(state).await;

                    })

                }

                async fn run(mut state: [<$actor_type State>])
                {
                    
                    if state.pre_run_async().await.into()
                    {

                        let mut proceed = true; 

                        while proceed
                        {
                            
                            proceed = state.run_async().await.into()
                
                        }

                    }

                    state.post_run_async().await;

                }

            }
            
        }

    }

}



#[macro_export]
macro_rules! impl_task_actor_build_state_flexible
{

    ($actor_type:ident) =>
    {

        paste!
        {

            pub struct $actor_type
            {
            }

            impl $actor_type
            {

                pub fn spawn_and_build_state(state_builder: [<$actor_type StateBuilder>]) -> JoinHandle<()>
                {
                    
                    tokio::spawn(async move
                    {

                        let opt_state = state_builder.build_async().await;

                        if let Some(state) = opt_state
                        {

                            $actor_type::run(state).await;

                        }

                    })

                }

                async fn run(mut state: [<$actor_type State>])
                {
                    
                    if state.pre_run_async().await.into()
                    {

                        let mut proceed = true; 

                        while proceed
                        {
                            
                            proceed = state.run_async().await.into();
                
                        }

                    }

                    state.post_run_async().await;

                }

            }
            
        }

    }

}

#[macro_export]
macro_rules! impl_task_actor_build_state_with_spawn_flexible
{

    ($actor_type:ident) =>
    {

        paste!
        {

            pub struct $actor_type
            {
            }

            impl $actor_type
            {

                pub fn spawn(state: [<$actor_type State>]) -> JoinHandle<()>
                {
                    
                    tokio::spawn(async move
                    {

                        $actor_type::run(state).await;

                    })

                }

                pub fn spawn_and_build_state(state_builder: [<$actor_type StateBuilder>]) -> JoinHandle<()>
                {
                    
                    tokio::spawn(async move
                    {

                        let opt_state = state_builder.build_async().await;

                        if let Some(state) = opt_state
                        {

                            $actor_type::run(state).await;

                        }

                    })

                }

                async fn run(mut state: [<$actor_type State>])
                {
                    
                    if state.pre_run_async().await.into()
                    {

                        let mut proceed = true; 

                        while proceed
                        {
                            
                            proceed = state.run_async().await.into();
                
                        }

                    }

                    state.post_run_async().await;

                }

            }
            
        }

    }

}

//catch_unwind




#[macro_export]
macro_rules! impl_task_actor_catch_unwind
{

    ($actor_type:ident, $panic_handler_type:ty) =>
    {

        paste!
        {

            pub struct $actor_type
            {
            }

            impl $actor_type
            {

                pub fn spawn_catch_unwind(state: [<$actor_type State>], panic_handler: &Arc<$panic_handler_type>) -> JoinHandle<()>
                {
                    
                    let panic_handler_clone = panic_handler.clone();
                    
                    tokio::spawn(async move
                    {

                        if let Err(err) = $actor_type::run_catch_unwind(state).catch_unwind().await
                        {

                            panic_handler_clone.handle_panic(err).await;

                        }

                    })

                }

                async fn run_catch_unwind(mut state: [<$actor_type State>])
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
            
        }

    }

}



#[macro_export]
macro_rules! impl_task_actor_build_state_and_catch_unwind
{

    ($actor_type:ident, $panic_handler_type:ty) =>
    {

        paste!
        {

            pub struct $actor_type
            {
            }

            impl $actor_type
            {

                pub fn spawn_build_state_and_catch_unwind(state_builder: [<$actor_type StateBuilder>], panic_handler: &Arc<$panic_handler_type>) -> JoinHandle<()>
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

                                    if let Err(err) = $actor_type::run_catch_unwind(state).catch_unwind().await
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

                async fn run_catch_unwind(mut state: [<$actor_type State>])
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
            
        }

    }

}

#[macro_export]
macro_rules! impl_task_actor_build_state_with_spawn_catch_unwind
{

    ($actor_type:ident, $panic_handler_type:ty) =>
    {

        paste!
        {

            pub struct $actor_type
            {
            }

            impl $actor_type
            {

                pub fn spawn_catch_unwind(state: [<$actor_type State>], panic_handler: &Arc<$panic_handler_type>) -> JoinHandle<()>
                {

                    let panic_handler_clone = panic_handler.clone();
                    
                    tokio::spawn(async move
                    {

                        if let Err(err) = $actor_type::run_catch_unwind(state).catch_unwind().await
                        {

                            panic_handler_clone.handle_panic(err).await;

                        }

                    })

                }

                pub fn spawn_build_state_and_catch_unwind(state_builder: [<$actor_type StateBuilder>], panic_handler: &Arc<$panic_handler_type>) -> JoinHandle<()>
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

                                    if let Err(err) = $actor_type::run_catch_unwind(state).catch_unwind().await
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

                async fn run_catch_unwind(mut state: [<$actor_type State>])
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
            
        }

    }

}

//flexible catch_unwind



#[macro_export]
macro_rules! impl_task_actor_catch_unwind_flexible
{

    ($actor_type:ident, $panic_handler_type:ty) =>
    {

        paste!
        {

            pub struct $actor_type
            {
            }

            impl $actor_type
            {

                pub fn spawn_catch_unwind(state: [<$actor_type State>], panic_handler: &Arc<$panic_handler_type>) -> JoinHandle<()>
                {
                    
                    let panic_handler_clone = panic_handler.clone();
                    
                    tokio::spawn(async move
                    {

                        if let Err(err) = $actor_type::run_catch_unwind(state).catch_unwind().await
                        {

                            panic_handler_clone.handle_panic(err).await;

                        }

                    })

                }

                async fn run_catch_unwind(mut state: [<$actor_type State>])
                {
                    
                    if AssertUnwindSafe(state.pre_run_async()).await.into()
                    {

                        let mut proceed = true; 

                        while proceed
                        {
                            
                            proceed = AssertUnwindSafe(state.run_async()).await.into();
                
                        }

                    }
                    
                    AssertUnwindSafe(state.post_run_async()).await;

                }

            }
            
        }

    }

}



#[macro_export]
macro_rules! impl_task_actor_build_state_and_catch_unwind_flexible
{

    ($actor_type:ident, $panic_handler_type:ty) =>
    {

        paste!
        {

            pub struct $actor_type
            {
            }

            impl $actor_type
            {

                pub fn spawn_build_state_and_catch_unwind(state_builder: [<$actor_type StateBuilder>], panic_handler: &Arc<$panic_handler_type>) -> JoinHandle<()>
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

                                    if let Err(err) = $actor_type::run_catch_unwind(state).catch_unwind().await
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

                async fn run_catch_unwind(mut state: [<$actor_type State>])
                {
                    
                    if AssertUnwindSafe(state.pre_run_async()).await.into()
                    {

                        let mut proceed = true; 

                        while proceed
                        {
                            
                            proceed = AssertUnwindSafe(state.run_async()).await.into();
                
                        }

                    }
                    
                    AssertUnwindSafe(state.post_run_async()).await;

                }

            }
            
        }

    }

}



#[macro_export]
macro_rules! impl_task_actor_build_state_with_spawn_catch_unwind_flexible
{

    ($actor_type:ident, $panic_handler_type:ty) =>
    {

        paste!
        {

            pub struct $actor_type
            {
            }

            impl $actor_type
            {

                pub fn spawn_catch_unwind(state: [<$actor_type State>], panic_handler: &Arc<$panic_handler_type>) -> JoinHandle<()>
                {

                    let panic_handler_clone = panic_handler.clone();
                    
                    tokio::spawn(async move
                    {

                        if let Err(err) = $actor_type::run_catch_unwind(state).catch_unwind().await
                        {

                            panic_handler_clone.handle_panic(err).await;

                        }

                    })

                }

                pub fn spawn_build_state_and_catch_unwind(state_builder: [<$actor_type StateBuilder>], panic_handler: &Arc<$panic_handler_type>) -> JoinHandle<()>
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

                                    if let Err(err) = $actor_type::run_catch_unwind(state).catch_unwind().await
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

                async fn run_catch_unwind(mut state: [<$actor_type State>])
                {
                    
                    if AssertUnwindSafe(state.pre_run_async()).await.into()
                    {

                        let mut proceed = true; 

                        while proceed
                        {
                            
                            proceed = AssertUnwindSafe(state.run_async()).await.into();
                
                        }

                    }
                    
                    AssertUnwindSafe(state.post_run_async()).await;

                }

            }
            
        }

    }

}