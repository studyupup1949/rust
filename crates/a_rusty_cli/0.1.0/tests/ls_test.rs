
use a_rusty_cli::{Command, CommandError, CommandManager, Opt, OptValue};

/// Represents the `ls` command.
struct LsCommand;

impl Command for LsCommand {
    /// Returns the options available for the `ls` command.
    fn opts(&self) -> Vec<Opt> {
        vec![
            Opt::new(
                "a",
                "all",
                OptValue::BOOL(false),
                "do not ignore entries starting with .",
            ),
            Opt::new(
                "l",
                "long",
                OptValue::BOOL(false),
                "use a long listing format",
            ),
            Opt::new(
                "h",
                "human-readable",
                OptValue::BOOL(false),
                "with -l and -s, print sizes like 1K 234M 2G etc.",
            ),
        ]
    }

    /// Executes the `ls` command with the given arguments and options.
    fn run(
        &self,
        args: Vec<String>,
        opts: std::collections::HashMap<Opt, OptValue>,
    ) -> Result<(), CommandError> {
        println!(
            "Running ls command with args: {:?} and opts {:?}",
            args, opts
        );

        // Do ls command logic here

        let mut all = false;

        if let Some(opt) = a_rusty_cli::find_opt(opts, "--all".to_string()) {
            if let OptValue::BOOL(value) = opt {
                all = value;
            }
        }
        println!("all: {}", all);

        for entry in std::fs::read_dir(".").unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            let file_name = path.file_name().unwrap().to_str().unwrap().to_owned();

            if !all && file_name.starts_with(".") {
                continue;
            }

            println!("{}", file_name);
        }

        Ok(())
    }
}

#[test]
fn test_ls_command() {
    let mut manager = CommandManager::new();
    manager.add_command("ls".to_string(), Box::new(LsCommand));

    let args = vec!["ls".to_string(), "--all".to_string()];

    let res = manager.run(args);

    assert!(res.is_ok());

    let args = vec!["-h".to_string()];

    let res = manager.run(args);

    assert!(res.is_ok());
}

struct EchoCommand;


impl Command for EchoCommand {
    fn opts(&self) -> Vec<Opt> {
        vec![
            Opt::new(
                "n",
                "no-newline",
                OptValue::BOOL(false),
                "do not output the trailing newline",
            ),
        ]
    }

    fn run(
        &self,
        args: Vec<String>,
        opts: std::collections::HashMap<Opt, OptValue>,
    ) -> Result<(), CommandError> {
        let mut no_newline = false;

        if let Some(opt) = a_rusty_cli::find_opt(opts, "--no-newline".to_string()) {
            if let OptValue::BOOL(value) = opt {
                no_newline = value;
            }
        }

        let output = args.join(" ");

        if no_newline {
            print!("{}", output);
        } else {
            println!("{}", output);
        }

        Ok(())
    }
}

#[test]
fn test_echo_command() {
    let mut manager = CommandManager::new();

    manager.add_command("echo".to_string(), Box::new(EchoCommand));

    let args = vec!["echo".to_string(), "Hello".to_string(), "World".to_string()];

    let res = manager.run(args);

    assert!(res.is_ok());
}

struct CatCommand;

impl Command for CatCommand {
    fn run(
        &self,
        args: Vec<String>,
        _opts: std::collections::HashMap<Opt, OptValue>,
    ) -> Result<(), CommandError> {
        for arg in args.iter().skip(1) {
            let arg_clone = arg.clone();
            let contents = std::fs::read_to_string(arg_clone).map_err(|_| CommandError::new())?;
            println!("{}", contents);
        }

        Ok(())
    }
}

#[test]
fn test_cat_command() {
    let mut manager = CommandManager::new();
    manager.add_command("cat".to_string(), Box::new(CatCommand));

    let args = vec!["cat".to_string(), "Cargo.toml".to_string()];

    let res = manager.run(args);

    assert!(res.is_ok());

    let args = vec!["cat".to_string(), "non-existent-file".to_string()];
    let res = manager.run(args);
    
    assert!(res.is_err());
}

struct TestOneCommand;
struct TestTwoCommand;

impl Command for TestOneCommand {
    fn run(
        &self,
        _args: Vec<String>,
        _opts: std::collections::HashMap<Opt, OptValue>,
    ) -> Result<(), CommandError> {
        println!("TestOneCommand executed");
        Ok(())
    }
}

impl Command for TestTwoCommand {
    fn run(
        &self,
        _args: Vec<String>,
        _opts: std::collections::HashMap<Opt, OptValue>,
    ) -> Result<(), CommandError> {
        println!("TestTwoCommand executed");
        Ok(())
    }
}

#[test]
fn test_command_manager() {
    let mut manager = CommandManager::new();
    manager.add_command("test one".to_string(), Box::new(TestOneCommand));
    manager.add_command("test two".to_string(), Box::new(TestTwoCommand));

    let args = vec!["test".to_string(), "one".to_string()];
    let res = manager.run(args);
    assert!(res.is_ok());

    let args = vec!["test".to_string(), "two".to_string()];
    let res = manager.run(args);
    assert!(res.is_ok());

    let args = vec!["--help".to_string()];

    let res = manager.run(args);

    assert!(res.is_ok());
}
