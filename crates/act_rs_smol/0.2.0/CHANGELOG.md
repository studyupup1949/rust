# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) (post version 0.2.0),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Version 0.2.0 (10/06/2026)

### Added

- Added documentation

- Added the spawn_and_build_state, spawn_catch_unwind, spawn_build_state_and_catch_unwind and run_catch_unwind methods to the TaskActor struct.

- Added the futures dependency.

- Added the task_actor_macro_tests module with TestActorState, TestActorStateBuilder, TestActorFlowState, TestActorFlowStateBuilder, TestPaincHander structs, without_builder and with_builder functions and task_actor, task_actor_build_state, task_actor_build_state_with_spawn, task_actor_flexible, task_actor_build_state_flexible, task_actor_build_state_with_spawn_flexible, task_actor_catch_unwind, task_actor_build_state_and_catch_unwind, task_actor_build_state_with_spawn_catch_unwind, task_actor_catch_unwind_flexible, task_actor_build_state_and_catch_unwind_flexible and task_actor_build_state_with_spawn_catch_unwind_flexible test functions.

- Added the impl_task_actor_build_state_with_spawn_catch_unwind, impl_task_actor_catch_unwind_flexible, impl_task_actor_build_state_and_catch_unwind_flexible and impl_task_actor_build_state_with_spawn_catch_unwind_flexible macros.

- Added the impl_task_actor_catch_unwind and impl_task_actor_build_state_and_catch_unwind macros.

- Added “/.vscode” and “/old” strings to the .gitignore file.

- Added the impl_task_actor_build_state_flexible and impl_task_actor_build_state_with_spawn_flexible macros.

- Added the accessorise and pastey optional dependencies and made them be included when you specify the thread_pool feature.

- Added the impl_task_actor_build_state_with_spawn and impl_task_actor_flexible macros.

- Added an inc_dec optional dependency.
    
- Added an futures-lite optional dependency.
    
- Added a thread_pool feature.
    
- Added a ThreadPool struct.

- Added the async-trait feature.

- Added the AutoDetachTask struct.

- Added a features field with values to the package.metadata.docs.rs section in the cargo.toml file.



### Changed

- Updated various dependences via “cargo update”.
    
- Updated the reademe.

- Other minor changes.

- Updated the inc_dec dependency to version 0.2.0.

- Uncommented package.metadata.docs.rs section in the Cargo.toml file.

- Uncommented and changed “doc_auto_cfg” to “doc_cfg” in the docsrs package level cfg_attr statement.

- Renamed the mac_task_actors module to task_actor_macros.

- Updated the act_rs dependency to version 0.5.0.

- Renamed the impl_mac_task_actor macro to impl_task_actor and updated its spawn meta-method to return an AutoDetachTask instance.

- Renamed impl_mac_task_actor_built_state to impl_task_actor_build_state and rearranged it to work like the impl_task_actor macro. Its spawn meta-method was renamed to spawn_and_build_state and it now returns an AutoDetachTask instance.

- The spawn method of the TaskActor implementation now returns an AutoDetachTask instance.

- Made made the presence of the TaskActor struct dependant on the newly added async-trait feature.



### Removed

- Removed the .vscode directory.

- Removed the spawn_attached meta-method of the impl_task_actor_build_state macro.

- The spawn_attached method of the TaskActor implementation has been removed.

- Removed the all-features field from the package.metadata.docs.rs section in the cargo.toml file.




## Version 0.1.0 (08/08/2025)

- Initial release
