use crate::utils::func::ident::{PID_TABLE, REGISTRY};
use std::io::{self /*BufRead*/};

pub fn run_tui() {
    loop {
        clear_screen();

        let table = PID_TABLE
            .table
            .lock()
            .unwrap();
        let mut pids: Vec<u64> = table
            .keys()
            .copied()
            .collect();
        pids.sort();

        if pids.is_empty() {
            println!("No functions registered!");
            return;
        }

        display_functions(&table, &pids);
        drop(table);

        println!("\nEnter function number (1-{}) or Q to quit: ", pids.len());

        let stdin = io::stdin();
        let mut input = String::new();
        stdin
            .read_line(&mut input)
            .unwrap();
        let input = input.trim();

        if input.eq_ignore_ascii_case("q") {
            break;
        } else if input.eq_ignore_ascii_case("") {
            clear_screen();
        }

        match input.parse::<usize>() {
            Ok(choice) if choice > 0 => {
                let table = PID_TABLE
                    .table
                    .lock()
                    .unwrap();
                let mut pids: Vec<u64> = table
                    .keys()
                    .copied()
                    .collect();
                pids.sort();

                if choice <= pids.len() {
                    let pid = pids[choice - 1];
                    if let Some(id) = table.get(&pid) {
                        println!("\nCalling: {}", id.name);
                    }
                    drop(table);

                    let func_result = {
                        let registry = REGISTRY
                            .lock()
                            .unwrap();
                        registry.call(pid)
                    };

                    match func_result {
                        Ok(f) => {
                            f();
                            println!("\n Function executed successfully");
                        }
                        Err(e) => {
                            println!("\n✗ Error: {}", e);
                        }
                    }
                    println!("\nPress ENTER to continue...");
                    let _ = stdin.read_line(&mut String::new());
                } else {
                    println!("Invalid selection!");
                }
            }
            _ => {
                println!("Invalid input!");
            }
        }
    }

    clear_screen();
    println!("Exiting TUI...");
}

fn display_functions(
    table: &std::collections::HashMap<u64, crate::utils::func::ident::Identifier>,
    pids: &[u64],
) {
    println!("╔════════════════════════════════════════╗");
    println!("║     Available Functions (TUI)          ║");
    println!("╠════════════════════════════════════════╣");

    for (idx, pid) in pids
        .iter()
        .enumerate()
    {
        if let Some(identifier) = table.get(pid) {
            let status_char = match &identifier
                .status
                .status_title
            {
                crate::utils::func::ident::Status_T::Working(_) => "⟳",
                crate::utils::func::ident::Status_T::Good(_) => "✓",
                crate::utils::func::ident::Status_T::Failed(_) => "✗",
                crate::utils::func::ident::Status_T::Paused(_) => "⏸",
            };

            println!(
                "║ ({}) {} {:<33}║",
                idx + 1,
                status_char,
                &identifier.name[..identifier
                    .name
                    .len()
                    .min(33)]
            );
        }
    }

    println!("╚════════════════════════════════════════╝");
}

fn clear_screen() {
    // print!("\x1B[2J\x1B[1;1H"); //idk what typa meth AI was doing
    println!("\x1Bc");
}
// fn main() {
// println!("wae")
// }
