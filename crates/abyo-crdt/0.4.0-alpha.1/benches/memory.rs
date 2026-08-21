//! Memory benchmark: peak heap allocation for typical workloads.
//!
//! Run with `cargo bench --bench memory`. Numbers are the *peak resident
//! heap* allocated through the global allocator during one iteration of
//! each scenario, plus the count of allocations.
#![allow(missing_docs)]

use abyo_crdt::{List, Text};
use peakmem_alloc::{PeakMemAlloc, PeakMemAllocTrait, INSTRUMENTED_SYSTEM};

#[global_allocator]
static GLOBAL: &PeakMemAlloc<std::alloc::System> = &INSTRUMENTED_SYSTEM;

fn measure<F: FnOnce()>(label: &str, f: F) {
    GLOBAL.reset_peak_memory();
    f();
    let peak = GLOBAL.get_peak_memory();
    println!("{label:>40}: peak {peak:>9} B");
}

fn main() {
    println!("Memory benchmarks (PeakAlloc, single-threaded):");
    println!("{:>40}  {:>11}  {:>14}", "scenario", "peak", "retained");

    measure("List<char> append 1000 chars", || {
        let mut list = List::<char>::new(1);
        for i in 0..1000 {
            list.insert(i, 'x');
        }
        std::hint::black_box(list);
    });

    measure("List<char> append 10000 chars", || {
        let mut list = List::<char>::new(1);
        for i in 0..10_000 {
            list.insert(i, 'x');
        }
        std::hint::black_box(list);
    });

    measure("List<char> 1000-then-delete-half", || {
        let mut list = List::<char>::new(1);
        for i in 0..1000 {
            list.insert(i, 'x');
        }
        for _ in 0..500 {
            list.delete(0);
        }
        std::hint::black_box(list);
    });

    measure("Text plain 1000 chars", || {
        let mut text = Text::new(1);
        text.insert_str(0, &"x".repeat(1000));
        std::hint::black_box(text);
    });

    measure("Text 1000 chars + 100 marks", || {
        let mut text = Text::new(1);
        text.insert_str(0, &"x".repeat(1000));
        for i in 0..100 {
            text.set_mark(i * 10..i * 10 + 5, "bold", true);
        }
        std::hint::black_box(text);
    });
}
