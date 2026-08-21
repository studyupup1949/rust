use std::env;

use coloredpp::Colorize;

#[derive(Clone, Debug)]
pub struct Opt<'a> {
    pub name: &'a str,
    pub short: Option<&'a str>,
    pub manual: &'a str,
    pub description: &'a str,
    pub value: Option<String>,
}

#[derive(Debug)]
pub struct CLI<'a> {
    pub name: Option<&'a str>,
    pub version: Option<&'a str>,
    pub description: Option<&'a str>,
    pub options: Vec<Opt<'a>>,
}

impl<'a> CLI<'a> {
    pub fn new() -> Self {
        Self {
            name: None,
            version: None,
            description: None,
            options: vec![],
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
        // skip trought the command name
        iter.next();
        // parse args
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--help" | "-h" => {
                    self.print_help();
                }
                "--version" | "-v" => {
                    // print version
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
                    }
                }
            }
        }
    }

    /// Get the value of a specific option by name
    pub fn get(&self, opt_name: &str) -> Option<&str> {
        self.options
            .iter()
            .find(|opt| opt.name == opt_name)
            .and_then(|opt| opt.value.as_deref())
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
