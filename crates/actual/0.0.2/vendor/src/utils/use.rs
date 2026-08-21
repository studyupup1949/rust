// fn get_input<T>() -> T

#[derive(Default)]
pub struct Input {
    buffer: Option<String>,
    prompt: Option<String>,
    file: Option<String>,
    mode: Option<WriteMethod>,
    convert_type: Option<Type>,
}

enum Type {
    BOOL,
    F64,
    I64,
    U64,
    I32,
    U32,
    F32,
    I16,
    U16,
    I8,
    U8,
}

enum WriteMethod {
    To,
    From,
}

// pub fn ::new()

// impl Default for Input {
//     fn default() -> Self {
//         Input {}
//     }
//}
/// A input struct that you can use to take input from user.
impl Input {
    pub fn new() -> Input {
        Input {
            buffer: None,
            prompt: None,
            file: None,
            mode: None,
            convert_type: None,
        }
    }
    pub fn remove_whitespace(&mut self) {
        // let buff : Option<String>;
        // let mut buff: String = "w".to_string();
        // match &self.buffer {
        // Some(v) => self.buffer.take() = v.trim().,
        // Some(v) => buff = Some(v.to_owned()),
        // Some(v) => {
        // buff = v.to_owned();
        // self.buffer = Some(v.trim().to_owned())
        // }
        // None => println!("Buffer is empty"),
        // }
        if let Some(v) = self.buffer.as_mut() {
            *v = v.trim().to_owned();
        } else {
            println!("Buffer is empty");
        }
    }
    // self.buffer = self.buffer.take().map(|v| v.trim().to_owned());
    // if let Some(v) = &self.buffer {
    // self.buffer = Some(v.trim().to_owned());}
    // let buff : Option<String>;
    // let mut buff : String = "w".to_string();
    // match &self.buffer {
    // Some(v) => self.buffer.take() = v.trim().,
    // Some(v) => buff = Some(v.to_owned()),
    // Some(v) => buff =v.to_owned(),
    //  None => println!("Buffer is empty"),
    // }
    // self.buffer = Some(buff.trim().to_owned())
    pub fn read(&mut self, prompt: String) {
        self.buffer = Some(read_input::<String>(&prompt));
    }
    // pub fn convert(&mut self) {
    //     match self.convert_type {
    //         None => println!("NO TYPE TO CONVERT TO"),
    //         Some(t) => match t {
    //             Type::BOOL => match t.parse::<bool>(),
    //         },
    //     }
    // }
}
pub fn new() -> Input {
    Input::new()
}
// TODO:
// laos inputshould have prompt and buffer
//I guess do something like:
// input.new ..
// input.reset (cauz input would have a couple fieds)
// and if we want to lets say read inputfrom a file we do:
// .read_from(..)
// and return to return a value
// drop to drop input
// reset to default state of input
// smt liek:
// enum :wrte
// onto
// from
// struc tinput
// write : write::Onto
// file locaiton: <Option<String>
// content = Option
// into means requres a mutable refce of a varbles u want to rwite input.contents onto.
// if uwant to mkae input do .read().
// if u want to mutabl ear u do .into(&mut x)
// if u want to modiyf file we just do .to(Option string)
// if to optin is given we overrwite locaiton
// if not we write to curr
// if it is alr none then we make a temp file
// adn if we want to use that file later , we can access, a pub mutex vec contianting all fiels.
// i guess make a file have somethign like x.l.m.  x being type, l being wtvr i geuss m
// so smt futureistic. i geuss
// what did we start and waht are we ending at lol.
// .return() to reutnr val.
// .from() to modify conents to new.
// .add_to_regex() if we want to add contnes into regis (lets ay we wnat ot .rseet )
// but in furute we want to access taht contnte again yk.
//
//
use std::io::Write;
pub fn read_input<T>(prompt: &str) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    loop {
        print!("{prompt} ");
        loop {
            match std::io::stdout().flush() {
                Ok(()) => break,
                Err(e) => {
                    eprintln!("Flush failed ({e}), retrying...");
                }
            }
        }
        let mut buf = String::new();
        if std::io::stdin()
            .read_line(&mut buf)
            .is_err()
        {
            println!("Failed to read input, try again.");
            continue;
        }

        match buf
            .trim()
            .parse::<T>()
        {
            Ok(value) => return value,
            Err(e) => println!("Invalid input ({e}), please re-enter."),
        }
    }
}
pub fn read_input_into<T>(var: &mut T, prompt: &str)
where
    T::Err: std::fmt::Display,
    T: std::str::FromStr,
{
    loop {
        println!("{prompt} ");
        loop {
            match std::io::stdout().flush() {
                Ok(()) => break,
                Err(e) => {
                    eprintln!("Flush failed ({e}), retrying...");
                }
            }
        }
        let mut buf = String::new();
        if std::io::stdin()
            .read_line(&mut buf)
            .is_err()
        {
            println!("Failed to read input, try again");
            continue;
        }
        match buf
            .trim()
            .parse::<T>()
        {
            Ok(value) => *var = value,
            Err(e) => println!("Invalid input ({e}), please re-enter . "),
        }
    }
}

pub fn read_input_to(file: Option<String>, prompt: &str) -> Result<String, String> {
    let v = "Pres".to_string();
    println!("{prompt}");
    match file {
        None => println!("NONE"),
        Some(v) => println!("{v}"),
    }
    Ok(v) // If file none then make file
}
// To print output to a file.
// pub fn read_input_from_into1()
// for reading file data
// fn read_from() // Forreading prompt from.

// pub fn parse_input<T>(input: &str) -> Result<T, T::Err>
// where
//     T: std::str::FromStr,
// {
//     input
//         .trim()
//         .parse::<T>()
// }
