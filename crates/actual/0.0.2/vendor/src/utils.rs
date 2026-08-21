pub mod call_to;
pub mod func;
pub mod types;
pub mod r#use;
// pub mod use;

pub mod macros;
pub use r#use::read_input;
pub use r#use::read_input_into;
// pub use
/*
pub use utils::func::Identifier as FuncIdentifier;
pub use utils::x::Identifier as XIdentifier;
 */

pub fn input() -> r#use::Input {
    r#use::new()
}
