//! Entry Point to actual code.

pub mod acc_soft;
pub mod book;
mod database;
pub mod dsa;
pub mod math;
pub mod menu;
pub mod prelude;
pub mod reexports;
pub mod rust_book;
pub mod test;
pub mod traits;
pub mod tui;
pub mod ui;
pub mod utils;

pub fn ligma() {
    let m = crate::math::tables::new();
} // pub(crate) use utils::r#use::get_input;
pub use bigdecimal::BigDecimal as BD;
// fn main() {
//     let math = crate::math::new();
//     let tables = math.table();
//     let mut m = tables.clone();
//     m.auto_generate();
//     m.print();
//     m.reset();
//     m.init(BD::from(1), BD::from(0), BD::from(10), BD::from(1))
//         .generate()
//         .print();
// }
