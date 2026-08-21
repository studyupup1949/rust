#![doc = include_str!("../README.md")]

//#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod task_actor;

pub use task_actor::*;

mod blocking_actor;

pub use blocking_actor::*;

mod mac_task_actors;

pub use mac_task_actors::*;

pub mod entering;

//pub use entering::*;



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
