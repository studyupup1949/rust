#![doc = include_str!("../README.md")]

#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "async-trait")]
mod task_actor;

#[cfg(feature = "async-trait")]
pub use task_actor::*;

mod blocking_actor;

pub use blocking_actor::*;

mod task_actor_macros;

//pub use task_actor_macros::*;

mod entering;

//pub use entering::*;

#[cfg(test)]
mod task_actor_macro_tests;

/*
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
*/
