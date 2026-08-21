use crossterm::event;

// use crate::c_wrapper;

// prelude.rs (crate root)
// pub use crate::utils::prelude::*;/
// pub use crate::dsa::prelude::*;
// pub use crate::math::prelude
pub mod call {}
use crate::rust_book::*;
use crate::udemy::*;
pub fn call() -> std::io::Result<()> {
    section_1_9(); // manually doing this is bad jsut use the pointer to call to ALL funcitosn 1 time at start.
    chapter_three();
    section_1_10();
    crate::math::call::c_tables::t1();
    guessing_game();
    section_1_8();
    crate::math::c_ool_patterns();
    crate::dsa::collections::c_bubble_sort::bs_1();
    //     c_wrapper!(
    //         fn_name: a1,
    //         id: "madd.1",
    //         location: "math::add::c_add",
    //         target: math::add::add_two,
    //         desc: "Adds two integers and returns the result.",
    //         args: [(a: i32), (b: i32)],
    //         ret_ty: i32
    //     );
    // let mut vector = vec![vec![20, 30]];
    // vector.double_all();
    // let x = 42;
    // let y = 43;
    // let var1 = &x;
    // let mut var2 = &x;
    // var2 = &y;
    println!("\x1Bc");
    // println!("")
    // utility::func::ident::Js();
    call_to();
    // let (a, b) = call_to();
    // print!("{}, {:?}", a, b.name);
    // print!("{}, {:?}", a, b);
    // test();
    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| frame.render_widget("Hello World!", frame.area()))?;
            if event::read()?.is_key_press() {
                // if crate::event::read()?.is_key_press() {
                break Ok(());
            }
        }
    })
}

fn call_to() {
    // BEFORE we continue im again calling onto 1000 FUncs and not known and all output is in one thingy so i ahve to go through 2000lines;
    // before this mess gets bigger, we fix ;
    // println!("Start of guessing Game") // WAALHI I RMEMBER SMT AOBUT ISNG MOD {} TO CODE GUESSING AME WTF
    println!("\n\n\n START OF CALL_TO main.rs");
    // -> (i32, utility::func::ident::Identifier) {
    // let result = math::add::add_two(5, 3);
    // println!("Result: {}", result);
    // let (a, b) = utility::call_to::main();
    // (a, b)
    {
        println!("\n\n\n\n\n\n %%%###### Start of utility::call_to::main() ######%%%%");
        let id1 = crate::math::call::c_add::a1();
        // println!("Answer: {}", id1.args.0.val);
        println!("Identifier: {}\n", id1.name);
        println!(
            "Calling from main.rs::call_to(), to math::c_add::a1() function, which calls add::add_two() function."
        );
        // eprintln!("error fiel")
        println!("######%%% End of utility::call_to::main() ######%%%%\n\n\n\n\n\n ");
    }
    test();
    // bubble_sort();
    {
        println!("\n\n\n START OF bubble_sort main.rs");
        // let mut arr = vec![2.2, 2.1];

        // let (_, mut b) = dsa::collections::c_bubble_sort::bs_1();
        let mut b = crate::dsa::collections::c_bubble_sort::bs_1();
        // a =
        b.s_lock("Locked for testing".to_string());
        println!("END OF bubble_sort main.rs\n\n\n\n");
    }
    {
        println!("\n\n\n\n\nTesting INTO-iter vs ITER ");
        let ve = [vec![102030, 20, 21, 102, 20], vec![30, 30, 10]];
        let ve_iter = ve.iter();
        for (v_idx, v) in ve_iter.enumerate() {
            for (l_idx, l) in v.iter().enumerate() {
                println!("Vector Index: {v_idx}, Item Index: {l_idx}, Item Value: {l}");
            }
        }
        println!("\n\n\n\n\nEND OF Testing INTO-iter vs ITER\n\n ");
    }

    println!("\n\n\n END OF CALL_TO main.rs");
}
// mod test;
fn test() {
    println!("\n\n\n START OF test main.rs");
    let m = crate::test::m::test();
    println!("Identifier1 = {:?}", m);
    println!("END OF test main.rs\n\n\n\n");
    vec!["walter"];
}
// fn call_to() -> (i32, utility::func::ident::Identifier) {
// let result = math::add::add_two(5, 3);
// println!("Result: {}", result);
// let (a, b) = utility::call_to::main();
// (a, b)
// }
// fn test() {
//     let result = math::add::add_two(5, 3);
//     println!("Result: {}", result);
// }

// fn bubble_sort() {}
/*
mod vector_double {
    use num_traits::Num;

    // The trait: doubles all leaf numbers, returns nesting depth
    pub trait DoubleAll {
        fn double_all(&mut self) -> usize;
    }

    // Marker trait: "this type is a plain number, not a container"
    trait Leaf {}

    impl Leaf for i8 {}
    impl Leaf for i16 {}
    impl Leaf for i32 {}
    impl Leaf for i64 {}
    impl Leaf for i128 {}
    impl Leaf for u8 {}
    impl Leaf for u16 {}
    impl Leaf for u32 {}
    impl Leaf for u64 {}
    impl Leaf for u128 {}
    impl Leaf for f32 {}
    impl Leaf for f64 {}

    // Base case: a plain number is depth 0
    impl<T: Num + Copy + Leaf> DoubleAll for T {
        fn double_all(&mut self) -> usize {
            *self = *self * (T::one() + T::one());
            0
        }
    }

    // Recursive case: a Vec of anything that implements DoubleAll
    impl<T: DoubleAll> DoubleAll for Vec<T> {
        fn double_all(&mut self) -> usize {
            let mut depth = 0;
            for item in self.iter_mut() {
                depth = item.double_all();
            }
            depth + 1
        }
    }

    pub fn main() {
        // Depth 1
        let mut flat: Vec<i32> = vec![1, 2, 3];
        let d1 = flat.double_all();
        println!("{flat:?}, depth = {d1}");

        // Depth 2
        let mut nested: Vec<Vec<i32>> = vec![vec![1, 2], vec![3, 4]];
        let d2 = nested.double_all();
        println!("{nested:?}, depth = {d2}");

        // Depth 3
        let mut deep: Vec<Vec<Vec<i32>>> = vec![vec![vec![1, 2]], vec![vec![3]]];
        let d3 = deep.double_all();
        println!("{deep:?}, depth = {d3}");

        // Depth 4, with f64 instead of i32
        let mut very_deep: Vec<Vec<Vec<Vec<f64>>>> = vec![vec![vec![vec![1.5, 2.5]]]];
        let d4 = very_deep.double_all();
        println!("{very_deep:?}, depth = {d4}");
    }
}
 */
