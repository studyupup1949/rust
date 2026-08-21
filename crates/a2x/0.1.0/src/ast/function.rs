//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use super::AsAlfa;
use super::PrettyPrint;
use super::QualifiedName;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum FunctionInputArg {
    /// A reference to a specific atomic type (`string`, `inetAddress`, etc.)
    Atomic(String),
    /// A reference to a bag of a specific atomic type (`bag[bool]`)
    AtomicBag(String),
    /// A reference to a bag of any atomic type (`bag[anyAtomic]`)
    AnyAtomicBag,
    /// A placeholder for any atomic type (`anyAtomic`)
    AnyAtomic,
    /// A placeholder for any atomic or bag
    AnyAtomicOrBag,
    /// A placeholder for a function
    Function,
}

/// Possible function outputs.
///
/// Notably, functions cannot produce other functions, or an output
/// that may be either atomic or bag (`AnyAtomicOrBag`).
#[derive(Debug, PartialEq, Clone)]
pub enum FunctionOutputArg {
    /// A reference to a specific atomic type (string, inetAddress, etc.)
    Atomic(String),
    /// A reference to a bag of a specific atomic type (`bag[bool]`)
    AtomicBag(String),
    /// A reference to a bag of any atomic type (`bag[anyAtomic]`)
    AnyAtomicBag,
    /// A placeholder for any atomic type (`anyAtomic`)
    AnyAtomic,
}

/// A collection of inputs to a function definition.
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionInputs {
    /// All the input arguments in order
    pub args: Vec<FunctionInputArg>,
    /// Is the last input argument a wildcard?
    pub wildcard: bool,
}

/// A function declaration
#[derive(Debug, Clone)]
pub struct Function {
    pub id: String,
    /// The namespace from general to most specific
    pub ns: Vec<String>,
    /// The URN of the function
    pub function_uri: String,
    /// Input arguments
    pub input_args: FunctionInputs,
    /// Output arguments
    pub output_arg: FunctionOutputArg,
}

impl AsAlfa for Function {
    fn to_alfa(&self, indent_level: usize) -> String {
        let indent = "  ".repeat(indent_level);
        // Ex: function stringEqual = "urn:oasis:...:string-equal"
        //         : string string -> boolean
        let mut output = format!(
            "{}function {} = \"{}\" : ",
            indent, self.id, self.function_uri
        );
        for a in &self.input_args.args {
            output.push_str(&a.to_string());
            output.push(' ');
        }
        if self.input_args.wildcard {
            output.push_str("* ");
        }
        // output
        output.push_str("-> ");
        output.push_str(&self.output_arg.to_string());
        output.push('\n');
        output
    }
}

/// Target equality, ignoring context
impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.ns == other.ns
            && self.function_uri == other.function_uri
            && self.input_args == other.input_args
            && self.output_arg == other.output_arg
    }
}

/// Generate the fully-qualified name for a function
impl QualifiedName for Function {
    fn fully_qualified_name(&self) -> Option<String> {
        let mut qn = self.ns.join(".");
        if !self.ns.is_empty() {
            qn.push('.');
        }
        qn.push_str(&self.id);
        Some(qn.to_string())
    }
}

/// Pretty print function
impl PrettyPrint for Function {
    fn pretty_print(&self, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        print!("{indent}{self}");
        println!();
    }
}

/// Display the function name, arguments, and output
impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s: String = String::default();
        s.push_str(&format!("fn {} <{}> ", self.id, self.function_uri));
        // print out each arg
        let args: Vec<String> = self
            .input_args
            .args
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
        s.push('(');
        s.push_str(&args.join(", "));
        if self.input_args.wildcard {
            s.push('*');
        }
        s.push(')');
        // print output type
        s.push_str(&format!(" ==> {}", self.output_arg));
        write!(f, "{s}")
    }
}

impl fmt::Display for FunctionInputArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FunctionInputArg::AnyAtomic => write!(f, "anyAtomic"),
            FunctionInputArg::AnyAtomicBag => write!(f, "bag[anyAtomic]"),
            FunctionInputArg::AnyAtomicOrBag => write!(f, "anyAtomicOrBag"),
            FunctionInputArg::Atomic(s) => write!(f, "{s}"),
            FunctionInputArg::AtomicBag(s) => write!(f, "bag[{s}]"),
            FunctionInputArg::Function => write!(f, "function"),
        }
    }
}

impl fmt::Display for FunctionOutputArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FunctionOutputArg::AnyAtomic => write!(f, "anyAtomic"),
            FunctionOutputArg::AnyAtomicBag => write!(f, "bag[anyAtomic]"),
            FunctionOutputArg::Atomic(s) => write!(f, "{s}"),
            FunctionOutputArg::AtomicBag(s) => write!(f, "bag[{s}]"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        // ensure that serialization works to ALFA, especially for the
        // function input args.
        let args = vec![
            FunctionInputArg::Atomic("string".to_owned()),
            FunctionInputArg::AtomicBag("boolean".to_owned()),
            FunctionInputArg::AnyAtomic,
            FunctionInputArg::AnyAtomicBag,
            FunctionInputArg::AnyAtomicOrBag,
            FunctionInputArg::Function,
        ];
        let output_arg = FunctionOutputArg::AtomicBag("string".to_owned());
        let f = Function {
            id: "fn_name".to_owned(),
            ns: vec!["main".to_owned()],
            function_uri: "urn:oasis:sample".to_owned(),
            input_args: FunctionInputs {
                args,
                wildcard: true,
            },
            output_arg,
        };
        let a = f.to_alfa(1);
        assert_eq!(
            a,
            "  function fn_name = \"urn:oasis:sample\" : string bag[boolean] anyAtomic bag[anyAtomic] anyAtomicOrBag function * -> bag[string]\n"
        );
    }
}
