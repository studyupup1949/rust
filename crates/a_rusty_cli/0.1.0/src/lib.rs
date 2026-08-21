use std::{collections::HashMap, fmt};
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OptValue {
    STRING(String),
    INT64(i64),
    UINT64(u64),
    BOOL(bool),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Opt {
    short: String,
    long: String,
    value: OptValue,
    description: String,
}

impl Opt {
    pub fn new(short: &str, long: &str, value: OptValue, description: &str) -> Opt {
        Opt {
            short: short.to_string(),
            long: long.to_string(),
            value: value,
            description: description.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandError {}

impl CommandError {
    pub fn new() -> CommandError {
        CommandError {}
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CommandError")
    }
}

pub trait Command {
    fn opts(&self) -> Vec<Opt> {
        vec![]
    }
    fn run(&self, args: Vec<String>, opts: HashMap<Opt, OptValue>) -> Result<(), CommandError>;
}

pub struct CommandManager {
    commands: HashMap<Vec<String>, Box<dyn Command + 'static>>,
}

pub fn find_opt(opts: HashMap<Opt, OptValue>, key: String) -> Option<OptValue> {
    for (opt, value) in opts.iter() {
        if key.eq(format!("--{}", opt.long).as_str()) {
            return Some(value.clone());
        }

        if key.eq(format!("-{}", opt.short).as_str()) {
            return Some(value.clone());
        }
    }
    None
}

impl CommandManager {
    pub fn new() -> CommandManager {
        CommandManager {
            commands: HashMap::new(),
        }
    }

    pub fn add_command(&mut self, name: String, command: Box<dyn Command>) {
        let name = name.split_whitespace().map(|s| s.to_string()).collect();
        self.commands.insert(name, command);
    }

    fn get_command(&self, args: Vec<String>) -> Option<&dyn Command> {
        // Collect all the keys in the hashmap and sort them by length
        let keys: Vec<&Vec<String>> = self.commands.keys().collect();

        for key in keys.iter().copied() {
            if key.len() > args.len() {
                println!("Key is longer than args");
                continue;
            }

            let mut matched = true;

            let args_without_options: Vec<String> = args
                .iter()
                .filter(|arg| !arg.starts_with("-"))
                .map(|arg| arg.clone())
                .collect();

            for i in 0..key.len() {
                if !key[i].eq(&args_without_options[i]) {
                    println!(
                        "Key does not match {:?} != {:?}",
                        key[i], args_without_options[i]
                    );
                    matched = false;
                    break;
                }
            }

            if matched {
                println!("Matched key: {:?}", key);
                return Some(self.commands.get(key).unwrap().as_ref());
            }
        }

        println!("No command matched");

        None
    }

    fn print_help(&self) {
        println!("Available commands:");
        for (key, command) in self.commands.iter() {
            println!("{}", key.join(" "));

            for opt in command.opts() {
                println!(
                    "  -{}, --{}: {}",
                    opt.short, opt.long, opt.description
                );
            }
        }
    }

    fn parse_opts(&self, args: Vec<String>, opts: Vec<Opt>) -> HashMap<Opt, OptValue> {
        let mut result = HashMap::new();

        for (pos, arg) in args.iter().enumerate() {
            if !arg.starts_with("-") {
                continue;
            }


            let opt_result = opts.iter().find(|opt| {
                arg.eq(format!("-{}", opt.short).as_str())
                    || arg.eq(format!("--{}", opt.long).as_str())
            });

            if opt_result.is_none() {
                println!("No opt found for {:?}", arg);
                continue;
            }

            let opt = opt_result.unwrap();

            match &opt.value {
                OptValue::INT64(default_val) => {
                    let value = args
                        .get(pos + 1)
                        .unwrap_or(&"".to_string())
                        .parse::<i64>()
                        .unwrap_or(*default_val);
                    result.insert(opt.clone(), OptValue::INT64(value));
                }
                OptValue::UINT64(default_value) => {
                    let value = args
                        .get(pos + 1)
                        .unwrap_or(&"".to_string())
                        .parse::<u64>()
                        .unwrap_or(*default_value);
                    result.insert(opt.clone(), OptValue::UINT64(value));
                }
                OptValue::STRING(default_value) => {
                    let value = args.get(pos + 1).unwrap_or(&default_value).clone();
                    result.insert(opt.clone(), OptValue::STRING(value.clone()));
                }
                OptValue::BOOL(_) => {
                    let value = args
                        .get(pos + 1)
                        .unwrap_or(&"".to_string())
                        .parse::<bool>()
                        .unwrap_or(true);
                    result.insert(opt.clone(), OptValue::BOOL(value));
                }
            }
        }

        result
    }

    pub fn run(&self, args: Vec<String>) -> Result<(), CommandError> {
        if args.len() == 0  {
            self.print_help();
            return Ok(());
        }

        if  args[0].eq("--help") || args[0].eq("-h") {
            self.print_help();
            return Ok(());
        }

        let command = self.get_command(args.clone()).ok_or(CommandError::new())?;

        let opts = self.parse_opts(args.clone(), command.opts());

        return command.run(args, opts);
    }
}
