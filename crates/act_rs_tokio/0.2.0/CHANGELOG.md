# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) (post version 0.2.0),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Version 0.2.0 (27/04/2026)

### Added

- Added some documentation.

- Added the impl_task_actor_catch_unwind macro.

- Added the impl_task_actor_build_state_and_catch_unwind macro.

- Added the spawn_build_state_and_catch_unwind method to the TaskActor implementation.

- Added the futures optional dependency.

- Added the spawn_catch_unwind and spawn_build_state_and_catch_unwind methods to the BlockingActor implementation.

- Added the spawn_catch_unwind and run_catch_unwind methods to the TaskActor implementation.

- Added the impl_task_actor_flexible and impl_task_actor_build_state_flexible macros.

- Added the task_actor_build_state_with_spawn, task_actor_build_state_with_spawn_flexible, task_actor_build_state_with_spawn_catch_unwind, task_actor_build_state_with_spawn_flexible, task_actor_build_state_and_catch_unwind, task_actor_build_state_with_spawn_catch_unwind, task_actor_catch_unwind_flexible, task_actor_build_state_and_catch_unwind_flexible and task_actor_build_state_with_spawn_catch_unwind_flexible test functions to the task_actor_macro_tests module.

- Added the impl_task_actor_build_state_with_spawn, impl_task_actor_build_state_with_spawn_flexible, impl_task_actor_build_state_with_spawn_catch_unwind,
impl_task_actor_catch_unwind_flexible and impl_task_actor_build_state_with_spawn_catch_unwind_flexible macros.

- Added pastey to the package dev-dependencies section of the cargo.toml file.

- Added the task_actor_macro_tests module with TestActorState, TestActorStateBuilder, TestActorFlowState, TestActorFlowStateBuilder, TestPaincHander structs, without_builder and with_builder functions and task_actor, task_actor_build_state, task_actor_flexible, task_actor_build_state_flexible and task_actor_catch_unwind test functions.

- Added the spawn_and_build_state method to the BlockingActor implementation.

- Added the spawn_and_build_state method to the TaskActor implementation.

- Updated various dependencies via the cargo update command.

- Changed “doc_auto_cfg” to “doc_cfg” in the docsrs package level cfg_attr decoration.



### Changed

- Updated the readme.

- The act_rs dependency version has been updated to "0.5.0".

- The tokio dependency version has been updated to "1.52.1".

- Renamed the mac_task_actors module to task_actor_macros.

- Updated some documentation.

- Moved the “proceed” bool declaration into the “if state.pre_run()” scope in the run method of the BlockingActor implementation.

- Moved the “proceed” bool meta-declarations into the “if state.pre_run_async().await” scopes in the run meta-methods of the existing actor-generating macros.

- Moved the “proceed” bool declaration into the “if state.pre_run_async().await” scope in the run method of the TaskActor implementation.

- Made the run static method public in the BlockingActor implementation.

- Renamed the impl_mac_task_actor macro to impl_task_actor.

- Updated documentation

- Renamed the spawn_and_build meta-method definition to spawn_and_build_state in the impl_task_actor_build_state macro.

- Made the run method of the TaskActor implementation public.

- Made async-trait an optional feature and updated the project to reflect this.

- Made the entering module private.

- Renamed the impl_mac_task_actor_built_state macro to impl_task_actor_build_state and updated its run function to deal with “state types” in same way the other actor oriented macros and structs do. The spawn_and_build_state meta-method was added and the spawn method was removed from this macro declaration.



### Removed

- Removed the runtime_enter, runtime_enter_param, runtime_enter_param_ref, runtime_enter_param_mut, handle_enter, handle_enter_param, handle_enter_param_ref and handle_enter_param_mut functions and some old code and comments from the entering module.



## Version 0.1.0 (08/08/2025)

- Initial release
