use std::env;

use coloredpp::Colorize;

#[derive(Clone, Debug)]
pub struct Opt<'a> {
    /// option name (--run)
    pub name: &'a str,
    /// option short name (-r)
    pub short: Option<&'a str>,
    /// (-l, --long)
    pub manual: &'a str,
    /// option description
    pub description: &'a str,
    /// option value
    pub value: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Arg<'a> {
    // argument name (run)
    pub name: &'a str,
    // argument name (run [options])
    pub manual: &'a str,
    // argument description
    pub description: &'a str,
    /// option values
    pub values: Vec<String>,
}

#[derive(Debug)]
pub struct CLI<'a> {
    pub name: Option<&'a str>,
    pub version: Option<&'a str>,
    pub description: Option<&'a str>,
    pub options: Vec<Opt<'a>>,
    pub args: Vec<Arg<'a>>,
}

impl<'a> CLI<'a> {
    pub fn new() -> Self {
        Self {
            name: None,
            version: None,
            description: None,
            options: vec![],
            args: vec![],
        }
    }

    /// set the name of the app
    pub fn name(&mut self, name: &'a str) -> &mut Self {
        self.name = Some(name);
        self
    }

    /// set the app version
    pub fn version(&mut self, version: &'a str) -> &mut Self {
        self.version = Some(version);
        self
    }

    /// set the app description
    pub fn description(&mut self, desc: &'a str) -> &mut Self {
        self.description = Some(desc);
        self
    }

    /// add new arg
    pub fn arg(&mut self, name: &'a str, manual: &'a str, description: &'a str) -> &mut Self {
        self.args.push(Arg {
            name,
            manual,
            description,
            values: vec![],
        });
        self
    }

    /// add new option
    pub fn option(&mut self, manual: &'a str, description: &'a str) -> &mut Self {
        let mut opt = Opt {
            name: "",
            description,
            short: None,
            manual,
            value: None,
        };
        if manual.contains(',') {
            let opts: Vec<&str> = manual.trim().split(',').collect();
            // handle short command
            if !opts
                .get(0)
                .expect("expected a short option")
                .trim()
                .starts_with("-")
            {
                panic!(
                    "short option should start with '-' ({})",
                    opts.get(0).unwrap()
                );
            }
            opt.short = opts.get(0).cloned();

            // handle long command
            if !opts
                .get(1)
                .expect("expected an option")
                .trim()
                .starts_with("--")
            {
                panic!("option should start with '--'({})", opts.get(1).unwrap());
            }
            opt.name = opts.get(1).unwrap().trim();
        } else {
            if manual.trim().starts_with("--") {
                opt.name = manual.trim();
            } else {
                panic!("option should start with '--'({})", manual);
            }
        }
        self.options.push(opt);
        self
    }

    /// Parse the CL arguments
    pub fn parse(&mut self) {
        let args: Vec<String> = env::args().collect();
        let mut iter = args.iter().peekable();
        iter.next();

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--help" | "-h" => {
                    self.print_help();
                }
                "--version" | "-v" => {
                    self.print_version();
                }
                other => {
                    if let Some(opt) = self
                        .options
                        .iter_mut()
                        .find(|o| o.name == other || o.short == Some(other))
                    {
                        opt.value = Some("".to_string());
                        if let Some(value) = iter.peek() {
                            if !value.starts_with('-') {
                                opt.value = Some(value.to_string());
                                iter.next();
                            }
                        }
                    } else if let Some(arg_def) = self.args.iter_mut().find(|a| a.name == other) {
                        while let Some(value) = iter.peek() {
                            if value.starts_with('-') {
                                break;
                            }
                            arg_def.values.push(value.to_string());
                            iter.next();
                        }
                        arg_def.values.push("".to_string());
                    }
                }
            }
        }
    }

    /// Get the value of a specific option by name
    pub fn get(&self, name: &str) -> Option<&[String]> {
        if let Some(opt) = self.options.iter().find(|opt| opt.name == name) {
            if let Some(ref v) = opt.value {
                Some(std::slice::from_ref(v))
            } else {
                None
            }
        } else if let Some(arg) = self.args.iter().find(|arg| arg.name == name) {
            if !arg.values.is_empty() {
                Some(&arg.values)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn print_help(&self) {
        println!(
            "\n{}, {}{}",
            self.name.unwrap_or("My Program").red().bold(),
            "v".cyan().bold(),
            self.version.unwrap_or("0.0.1").cyan().bold()
        );
        println!("{}", self.description.unwrap_or("").italic());
        println!("\n{}", "Options:".yellow().bold());
        println!(
            "\t{}, {} \t{}",
            "--help".blue(),
            "-h".blue(),
            "Print this message"
        );
        println!(
            "\t{}, {} \t{}",
            "--version".blue(),
            "-v".blue(),
            "Print the application version"
        );
        self.options.iter().for_each(|opt| {
            println!(
                "\t{}, {} \t{}",
                opt.name.blue(),
                opt.short.unwrap_or("").blue(),
                opt.description
            )
        });
        println!("");
    }

    fn print_version(&mut self) {
        println!(
            "{}{}",
            "v".blue().bold(),
            self.version.unwrap_or("0.0.1").blue().bold()
        );
    }
}
