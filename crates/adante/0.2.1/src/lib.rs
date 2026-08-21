//! # adante
//!
//! `adante` is a simple library that handles the logic of user defined types
//! in order to provide efficiency in parsing command line arguments.
//! At its core, `adante` simply provides an interface of which command line
//! arguments can be transformed into a collection of three types.
//!
//! - Flags, which consist of:
//!     - A flag key
//!     - An optional `String` value
//! - Actions
//! - Errors
//!
//! This is achieved by implementing a simple, but widely versatile
//! set of tools laid out by the library. Here are the steps to making
//! your first parser!
//!
//! ## 1. Define an enum consisting of all the errors your program might run into.
//!
//! This can be seperate from your application's general Error enum
//! if you have one, or it can be the same.
//!
//! ```
//! #[derive(Debug, Clone, Copy)] // Highly advised
//! pub enum ErrorType {
//!     Syntax,
//!     InvalidAction,
//!     InvalidFlag,
//!     NoFlagVal,
//! }
//! ```
//!
//! ## 2. Imply the `Error` trait from the library.
//!
//! ```
//! use adante::Error;
//! #[derive(Debug, Clone, Copy)] // Highly advised
//! pub enum ErrorType {
//!     Syntax,
//!     InvalidAction,
//!     InvalidFlag,
//!     NoFlagVal,
//! }
//! impl Error for ErrorType {
//!     fn handle(&self) {
//!         println!("{}", self.as_str());
//!         std::process::exit(1);
//!     }
//!     fn as_str(&self) -> &str {
//!         match self {
//!             Self::Syntax => "Improper syntax usage.",
//!             Self::InvalidAction => "One or more of your actions entered is invalid.",
//!             Self::InvalidFlag => "One or more of your flags entered is invalid",
//!             Self::NoFlagVal => "One or more of your flags is missing a field",
//!         }
//!     }
//! }
//! ```
//!
//! ## 3. Define an enum consisting of all of your flag and action keys.
//!
//! ```
//! enum FlagType {
//!     Help,
//!     Verbose,
//!     Print,
//! }
//!
//! enum ActionType {
//!     Add,
//!     Remove,
//!     Edit,
//! }
//! ```
//!
//! ## 4. Imply the `ArgumentType` trait from the library.
//!
//! ```
//! use adante::ArgumentType;
//! enum FlagType {
//!     Help,
//!     Verbose,
//!     Print,
//! }
//! impl ArgumentType for FlagType {
//!     fn from_str<ErrorType>(key: &str, error: ErrorType)
//!         -> Result<Self, ErrorType> {
//!         match key {
//!             "-h" | "--help" => Ok(Self::Help),
//!             "-v" | "--verbose" => Ok(Self::Verbose),
//!             "-p" | "--print" => Ok(Self::Print),
//!             _ => Err(error),
//!         }
//!     }
//! }
//!
//! enum ActionType {
//!     Add,
//!     Remove,
//!     Edit,
//! }
//! impl ArgumentType for ActionType {
//!     fn from_str<ErrorType>(key: &str, error: ErrorType)
//!         -> Result<Self, ErrorType> {
//!         match key {
//!             "a" | "add" => Ok(Self::Add),
//!             "r" | "remove" => Ok(Self::Remove),
//!             "e" | "edit" => Ok(Self::Edit),
//!             _ => Err(error),
//!         }
//!     }
//! }
//! ```
//!
//! ## And voila!
//!
//! Now your parser is complete! By plugging `std::env::args::collect()` into
//! `Arguments::parse()`, you will get a working `Arguments` object!
//!

#[cfg(test)]
mod tests;

/// A trait describing the shared methods of both Flags and Arguments

pub trait ArgumentType {
    /// A user implemented function that takes a string as input and returns an
    /// argument type.
    ///
    /// # Examples
    /// ```
    /// use adante::{ArgumentType, Error};
    ///
    /// #[derive(Debug, Clone, Copy)]
    /// enum ErrorType {
    ///     Syntax, // EXTREMELY simple example
    ///             // More complex examples are shown in
    ///             // the documentation for Error
    /// }
    /// impl Error for ErrorType {
    ///     fn handle(&self) {
    ///         ()
    ///     }
    ///     fn as_str(&self) -> &str {
    ///         "Syntax Error"
    ///     }
    /// }
    ///
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq)]
    /// enum FlagType {
    ///     Help,
    ///     Verbose,
    ///     Print,
    ///     TestFail, // NOTE: For testing only
    ///               // Use Error
    /// }
    /// impl ArgumentType for FlagType {
    ///     fn from_str<ErrorType>(key: &str, error: ErrorType)
    ///                                -> Result<Self, ErrorType> {
    ///         match key {
    ///             "-h" | "--help" => Ok(Self::Help),
    ///             "-v" | "--verbose" => Ok(Self::Verbose),
    ///             "-p" | "--print" => Ok(Self::Print),
    ///             _ => Err(error),
    ///         }
    ///     }
    /// }
    /// let result = match FlagType::from_str("-v", ErrorType::Syntax) {
    ///     Ok(t) => t,
    ///     Err(_) => FlagType::TestFail // In actual usecase this would pipe into
    ///                                  // An Error's handle function
    /// };
    /// assert_eq!(result, FlagType::Verbose)
    ///
    /// ```
    fn from_str<E: Error>(key: &str, error: E) -> Result<Self, E>
    where
        Self: std::marker::Sized;
}

/// A trait that describes the functions an error must implement to be valid

pub trait Error {
    /// A user implemented function that performs a task then exits
    /// depending on the type of error it is called on.
    ///
    /// In proper usage `std::process:exit(1)` would be used; however, this
    /// example just uses `assert_eq!(2 + 2, 4)` to validate the test.
    ///
    /// # Examples
    ///
    /// ```
    /// use adante::Error;
    ///
    /// #[derive(Debug, Clone, Copy)]
    /// enum ErrorType {
    ///     Syntax,
    ///     InvalidAction,
    ///     InvalidFlag,
    ///     NoFlagVal,
    /// }
    /// impl Error for ErrorType {
    ///     fn handle(&self) {
    ///         // Handle code goes here:
    ///         match self {
    ///             Self::Syntax => assert_eq!(2 + 2, 4),           // Only branch that should work
    ///             Self::InvalidAction => assert_eq!(2 + 2, 5),
    ///             Self::InvalidFlag => assert_eq!(2 + 2, 5),
    ///             Self::NoFlagVal => assert_eq!(2 + 2, 5),
    ///         }
    ///     }
    ///     fn as_str(&self) -> &str {" "}
    /// }
    ///
    /// let test_error = ErrorType::Syntax;
    /// test_error.handle();
    /// ```
    fn handle(&self);
    /// A user implemented function that returns a &str (usually an error message)
    /// depending ont he type of error it is called on.
    ///
    /// # Examples
    ///
    /// ```
    /// use adante::Error;
    ///
    /// #[derive(Debug, Clone, Copy)]
    /// enum ErrorType {
    ///     Syntax,
    ///     InvalidAction,
    ///     InvalidFlag,
    ///     NoFlagVal,
    /// }
    /// impl Error for ErrorType {
    ///     fn handle(&self) {  }
    ///     fn as_str(&self) -> &str {
    ///         match self {
    ///             Self::Syntax => "Good!",
    ///             Self::InvalidAction => "Bad!",
    ///             Self::InvalidFlag => "Bad!",
    ///             Self::NoFlagVal => "Bad!",
    ///         }
    ///     }
    /// }
    ///
    /// let test_error = ErrorType::Syntax;
    /// assert_eq!(test_error.as_str(), "Good!");
    /// ```
    fn as_str(&self) -> &str;
}

/// A subset struct of the `Arguments` struct that describes a Flag object

#[derive(Debug)]
pub struct Flag<T: ArgumentType> {
    pub key: T,
    // NOTE: Thought making String generic here
    // may have been overdoing it a bit.
    // Consider.
    pub value: Option<String>,
}

/// The meat of the library, describes an `Argument` object and its methods

#[derive(Debug)]
pub struct Arguments<F: ArgumentType, A: ArgumentType> {
    /// A list of the user defined Flag types and optional values
    pub flags: Vec<Flag<F>>,
    /// A list of the user defined Action types
    pub actions: Vec<A>,
}

impl<F: ArgumentType, A: ArgumentType> Arguments<F, A> {
    /// A default constructor for the Arguments type.
    ///
    /// Note: explicit type must be specified when using a default constructor,
    /// unlike vectors etc.
    ///
    /// # Examples
    /// ```
    /// use adante::{Arguments, ArgumentType};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq)]
    /// enum FlagType {
    ///     Help,
    ///     Verbose,
    ///     Print,
    /// }
    /// impl ArgumentType for FlagType {
    ///     fn from_str<ErrorType>(key: &str, error: ErrorType)
    ///                                -> Result<Self, ErrorType> {
    ///         match key {
    ///             "-h" | "--help" => Ok(Self::Help),
    ///             "-v" | "--verbose" => Ok(Self::Verbose),
    ///             "-p" | "--print" => Ok(Self::Print),
    ///             _ => Err(error),
    ///         }
    ///     }
    /// }
    /// enum ActionType {
    ///     Add,
    ///     Remove,
    ///     Edit,
    /// }
    /// impl ArgumentType for ActionType {
    ///     fn from_str<ErrorType>(key: &str, error: ErrorType)
    ///         -> Result<Self, ErrorType> {
    ///         match key {
    ///             "a" | "add" => Ok(Self::Add),
    ///             "r" | "remove" => Ok(Self::Remove),
    ///             "e" | "edit" => Ok(Self::Edit),
    ///             _ => Err(error),
    ///         }
    ///     }
    /// }
    /// let blank_args: Arguments<FlagType, ActionType> = Arguments::new();
    ///
    /// assert_eq!(blank_args.flags.len(), 0);
    /// assert_eq!(blank_args.actions.len(), 0);
    /// ```
    pub fn new() -> Self {
        Arguments {
            flags: Vec::new(),
            actions: Vec::new(),
        }
    }
    /// The parsing function that returns a full Arguments object.
    ///
    /// More complicated usages and tests can be found in the tests.rs file.
    ///
    /// # Examples
    ///
    /// ```
    /// use adante::{ArgumentType, Error, Arguments};
    ///
    /// #[derive(Debug, Clone, Copy)]
    /// enum ErrorType {
    ///     Syntax, // EXTREMELY simple example
    ///             // More complex examples are shown in
    ///             // the documentation for Error
    /// }
    /// impl Error for ErrorType {
    ///     fn handle(&self) {
    ///         ()
    ///     }
    ///     fn as_str(&self) -> &str {
    ///         "Syntax Error"
    ///     }
    /// }
    ///
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq)]
    /// enum FlagType {
    ///     Help,
    ///     Verbose,
    ///     Print,
    ///     TestFail, // NOTE: For testing only
    ///               // Use Error
    /// }
    /// impl ArgumentType for FlagType {
    ///     fn from_str<ErrorType>(key: &str, error: ErrorType)
    ///                                -> Result<Self, ErrorType> {
    ///         match key {
    ///             "-h" | "--help" => Ok(Self::Help),
    ///             "-v" | "--verbose" => Ok(Self::Verbose),
    ///             "-p" | "--print" => Ok(Self::Print),
    ///             _ => Err(error),
    ///         }
    ///     }
    /// }
    /// enum ActionType {
    ///     Add,
    ///     Remove,
    ///     Edit,
    /// }
    /// impl ArgumentType for ActionType {
    ///     fn from_str<ErrorType>(key: &str, error: ErrorType)
    ///         -> Result<Self, ErrorType> {
    ///         match key {
    ///             "a" | "add" => Ok(Self::Add),
    ///             "r" | "remove" => Ok(Self::Remove),
    ///             "e" | "edit" => Ok(Self::Edit),
    ///             _ => Err(error),
    ///         }
    ///     }
    /// }
    ///
    /// let env_args = vec!["-v"];
    /// let env_args: Arguments<FlagType, ActionType> =
    ///     match Arguments::parse(env_args, ErrorType::Syntax) {
    ///         Ok(a) => a,
    ///         Err(e) => Arguments::new()
    ///     };
    ///
    /// let mut result: FlagType = FlagType::TestFail;
    ///
    /// if env_args.flags.len() != 0 {
    ///     result = env_args.flags[0].key;
    /// }
    ///
    /// assert_eq!(result, FlagType::Verbose);
    ///
    /// ```
    pub fn parse<E: Error + Clone + Copy>(env_args: Vec<&str>, error: E) -> Result<Arguments<F, A>, E> {
        let mut args = Arguments::new();
        let mut eq_pos: usize = 0;
        for arg in env_args.iter() {
            // Detect if argument is option or action:
            if &arg[0..1] == "-" {
                // Assume flag, find seperator:
                for (i, &byte) in arg.as_bytes().iter().enumerate() {
                    if byte == b'=' {
                        eq_pos = i;
                    }
                }
                // Assume no value if no =:
                if eq_pos == 0 {
                    args.flags.push(Flag {
                        key: match F::from_str(arg, error.clone()) {
                            Ok(v) => v,
                            Err(e) => return Err(e),
                        },
                        value: None,
                    })
                // Seperator found
                // FIXME: BREAKS HERE
                } else {
                    let key = &arg[0..eq_pos];
                    let val = &arg[(eq_pos + 1)..];
                    args.flags.push(Flag {
                        key: match F::from_str(key, error.clone()) {
                            Ok(v) => v,
                            Err(e) => return Err(e),
                        },
                        // TODO: make value field a &str by default
                        value: Some(val.to_string()),
                    })
                }
            // TODO: Recognize file path, omit or save to output
            } else {
                // Assume action, match string to type
                args.actions.push(match A::from_str(arg, error) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                })
            }
        }

        Ok(args)
    }
}
