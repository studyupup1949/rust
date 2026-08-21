/*

mayee use:
register_fn! {
    name: "guessing_game",
    id: "rb.gg",
    location: "rust_book::guessing_game::guessing_game()",
    description: "So this is the rust book's guessing game chapter.",
    called_by: ["main.rs::main()"],
}
fn guessing_game() -> Identifier { ... }
*/
// pub mod guessing_game {
//     // use std::io::Read;

//     pub fn guessing_game() {
//         println!("Enter Your guess");
//         let mut guess = String::new();
//         loop {
//             guess.clear();
//             match std::io::stdin().read_line(&mut guess) {
//                 Ok(num) => {
//                     println!("You entered {num}");
//                     match guess
//                         .trim()
//                         .parse::<i32>()
//                     {
//                         Ok(parsed) => {
//                             println!("You entered {parsed}");
//                             break;
//                         }
//                         Err(err) => {
//                             eprintln!("Failed to convert to integer, {err}");
//                         }
//                     }
//                 }
//                 Err(err) => {
//                     eprintln!("{err}")
//                 }
//             }
//         }
//         println!("guess = {guess}");
//     }
// }

/* beter version is:
use std::cmp::Ordering;
use std::io;

use rand::Rng;

fn main() {
    println!("Guess the number!");

    let secret_number = rand::thread_rng().gen_range(1..=100);

    loop {
        println!("Please input your guess.");

        let mut guess = String::new();

        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");

        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        println!("You guessed: {guess}");

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too big!"),
            Ordering::Equal => {
                println!("You win!");
                break;
            }
        }
    }
}
*/
//
// here it is
pub mod guessing_game {
    use crate::utils::{
        func::ident::{Identifier, Status},
        read_input,
    };
    use rand::prelude::*;

    // use rug::rand;
    pub fn guessing_game() -> Identifier {
        println!("\n\n\n\n\n\n ###### Start of rust_book::guessing_game::guessing_game() ######");
        let mut rng = rand::rng();
        let random: u32 = rng.random_range(0..=100);
        println!("random : {random}");
        'matchi: loop {
            let guess: u32 = read_input("Enter guess ");
            match guess.cmp(&random) {
                std::cmp::Ordering::Less => println!("Number is too smol"),
                std::cmp::Ordering::Greater => println!("Numbr lwk big"),
                std::cmp::Ordering::Equal => {
                    println!("equal gng");
                    break 'matchi;
                }
            }
        }
        //#####################################################################
        //#####################################################################
        //#####################################################################
        //#####################################################################
        let mut id1 = Identifier {
            name: "guessing_game".to_string(),
            id: "rb.gg".to_string(),
            pid: None,
            location: "rust_book::guessing_game::guessing_game()".to_string(),
            description: Some(
                "So this is the rust book's guessingame chapter. i will add more".to_string(),
            ),
            return_type: None,
            return_value: None,
            args_type: None,
            number_of_args: None,
            args: None,
            source: Some(
                "rust_book::guessing_game::guessing_game()::guessing_game(){}".to_string(),
            ),
            source_call: Some("call.rb.gg".to_string()),
            cid: None,
            called_by: Some(vec!["main.rs::main()".to_string()]),
            status: Status {
                status_title: crate::utils::func::ident::Status_T::Working(Some(
                    "rust_book".to_string(),
                )),
                status_code: 523,
            },
            validate: false,
            func_pointer: Some(guessing_game as fn() -> Identifier),
        };
        id1.print_s();
        id1.generate_pid()
            .validate();
        id1.print_s();
        let pid = id1
            .pid
            .expect("NO PID AHAH");
        println!("id1 is {}", id1.validate.clone());
        println!("PID = {}, {:?}", std::process::id(), pid);
        println!("Calling from rust_book::guessing_game::guessing_game()::guessing_game");
        println!("###### End of rust_book::guessing_game::guessing_game() ######\n\n\n\n\n");
        println!("{:?}", id1.clone());
        id1.s_status(Some("Free to be used".to_string()), Some(200));
        id1.print_s();

        id1
    }
    // fn
}
pub mod chapter_three {
    use crate::utils::func::ident::{Identifier, Status};
    pub fn chapter_three() -> Identifier {
        println!("\n\n\n\n\n\n ###### Start of rust_book::chapter_three::chapter_three() ######");

        let mut id1 = Identifier {
            name: "chapter_three".to_string(),
            id: "rb-2".to_string(),
            pid: None,
            location: "crate::rust_book::chapter_three::chapter_three()".to_string(),
            description: Some(
                "Chapter three of TRPB and also can be accessed in rust_book::chapter_three()"
                    .to_string(),
            ),
            return_type: None,
            return_value: None,
            args_type: None,
            number_of_args: None,
            args: None,
            source: Some(
                "crate::rust_book::chapter_three::chapter_three()::chapter_three(){}".to_string(),
            ),
            source_call: Some("call.rb-2".to_string()),
            cid: None,
            called_by: Some(vec!["main.rs::main()".to_string()]),
            status: Status {
                status_title: crate::utils::func::ident::Status_T::Working(Some(
                    "rust_book".to_string(),
                )),
                status_code: 523,
            },
            validate: false,
            func_pointer: Some(chapter_three as fn() -> Identifier),
        };
        id1.print_s();
        id1.generate_pid()
            .validate();
        id1.print_s();
        let pid = id1
            .pid
            .expect("NO PID AHAH");
        println!("id1 is {}", id1.validate.clone());
        println!("PID = {}, {:?}", std::process::id(), pid);
        println!("Calling from crate::rust_book::chapter_three::chapter_three()::chapter_three");
        println!("###### End of rust_book::chapter_three::chapter_three() ######\n\n\n\n\n");
        println!("{:?}", id1.clone());
        id1.s_status(Some("Free to be used".to_string()), Some(200));
        id1.print_s();

        id1
    }
}

pub use chapter_three::chapter_three;
pub use guessing_game::guessing_game;
