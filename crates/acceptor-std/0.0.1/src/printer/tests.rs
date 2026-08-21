use crate::support::block_on;
use accepts::{Accepts, AsyncAccepts};

use super::{DebugPrinter, DisplayPrinter};

#[test]
fn prints_display_values() {
    let printer = DisplayPrinter;
    printer.accept(10);
    printer.accept(20);
}

#[test]
fn prints_debug_values_to_stdout() {
    // Using Debug printer; just ensure it compiles and runs without panic.
    let printer = DebugPrinter;
    printer.accept(Some(1));
    printer.accept(None as Option<i32>);
}

#[test]
fn prints_asynchronously() {
    let printer = DebugPrinter;

    block_on(printer.accept_async(1));
    block_on(printer.accept_async(2));
}
