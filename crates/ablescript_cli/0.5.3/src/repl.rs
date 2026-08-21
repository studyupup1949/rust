use ablescript::{interpret::ExecEnv, parser::parse};
use rustyline::DefaultEditor;

pub fn repl(ast_print: bool) {
    let mut rl = match DefaultEditor::new() {
        Ok(rl) => rl,
        Err(e) => {
            eprintln!("Failed to create editor: {e}");
            std::process::exit(-1);
        }
    };

    let mut env = ExecEnv::<ablescript::host_interface::Standard>::default();

    // If this is `Some`, the user has previously entered an
    // incomplete statement and is now completing it; otherwise, the
    // user is entering a completely new statement.
    let mut partial: Option<String> = None;
    loop {
        match rl.readline(if partial.is_some() { ">> " } else { ":: " }) {
            Ok(readline) => {
                let readline = readline.trim_end();
                let _ = rl.add_history_entry(readline);

                let partial_data = match partial {
                    Some(line) => line + readline,
                    None => readline.to_owned(),
                };

                partial = match parse(&partial_data).and_then(|ast| {
                    if ast_print {
                        eprintln!("{:#?}", &ast);
                    }
                    env.eval_stmts(&ast)
                }) {
                    Ok(_) => None,
                    Err(ablescript::error::Error {
                        // Treat "Unexpected EOF" errors as "we need
                        // more data".
                        kind: ablescript::error::ErrorKind::UnexpectedEoi,
                        ..
                    }) => Some(partial_data),
                    Err(e) => {
                        println!("{}", e);
                        println!(" | {}", partial_data);
                        println!(
                            "   {}{}",
                            " ".repeat(e.span.start),
                            "^".repeat((e.span.end - e.span.start).max(1))
                        );
                        None
                    }
                };
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("bye");
                break;
            }
            Err(rustyline::error::ReadlineError::Interrupted) => (),
            Err(e) => {
                println!("Error: {:?}", e);
                break;
            }
        }
    }
}
