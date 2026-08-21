#![allow(unused, non_camel_case_types, clippy::upper_case_acronyms)]
// #[derive(Debug, Clone, PartialEq)]
// pub enum FuncReturn {
//     Nested,
//     Single(String),
// }
// #[derive(Debug, Clone, PartialEq)]
// pub enum FuncReturnType {
//     Void,
//     Int,
//     Float,
//     String,
//     Bool,
//     Custom(String),
// }
// #[derive(Debug, Clone, PartialEq)]
// pub enum FuncReturnValue {
//     Void,
//     Int32(i32),
//     Int(i32),
//     Float(f64),
//     String(String),
//     Bool(bool),
//     Custom(String),
// }
// // FuncTion
// #[derive(Debug, Clone, PartialEq)]
// pub enum FuncArgType {
//     Void,
//     Int,
//     Float,
//     String,
//     Bool,
//     Custom(String),
// }

// #[derive(Debug, Clone, PartialEq)]
// pub struct FuncIdentifier {
//     location: crate::menu::func_return::FuncReturn,
//     name: String,
//     function_type: String,
//     //     nested_location: String,
//     return_type: crate::menu::func_return::FuncReturnType,
//     return_value: crate::menu::func_return::FuncReturnValue,
//     args: Vec<crate::menu::func_return::FuncArgType>,
//     number_of_args: usize,
// }
// let example = Func_Identifier {
//     location: FuncReturn::Nested,
//     name: "example_function".to_string(),
//     function_type: "void".to_string(),
//     nested_location: "example_module".to_string(),
//     return_type: crate::menu::func_return::FuncReturnType::Void,
//     return_value: crate::menu::func_return::FuncReturnValue::Void(),
// };

// ─────────────────────────────────────────────
// NoneReason — for normal "nothing to return" cases
// ─────────────────────────────────────────────
/* #[derive(Debug, Clone, PartialEq)]
pub enum NoneReason {
    NotFound(String),
    EmptyInput,
    OutOfBounds(String),
    ConditionNeverMet(String),
    Unknown(String),
    StdIo(String),
    StdParse(String),
}

impl std::fmt::Display for NoneReason {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NoneReason::NotFound(what) => write!(f, "not found: {what}"),
            NoneReason::EmptyInput => write!(f, "input was empty"),
            NoneReason::OutOfBounds(msg) => write!(f, "out of bounds: {msg}"),
            NoneReason::ConditionNeverMet(cond) => write!(f, "condition never met: {cond}"),
            NoneReason::Unknown(msg) => write!(f, "unknown reason: {msg}"),
            NoneReason::StdIo(msg) => write!(f, "I/O error: {msg}"),
            NoneReason::StdParse(msg) => write!(f, "parse error: {msg}"),
        }
    }
}

// ─────────────────────────────────────────────
// ErrType — for real failures/bugs
// ─────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub enum ErrType {
    DivideByZero,
    FileMissing(String),
    TypeMismatch(String),
    TypeConversion { from: String, to: String },
    NoneReason(NoneReason), // folds NoneReason in as one kind of ErrType
    Custom(String),
}

impl std::fmt::Display for ErrType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ErrType::DivideByZero => write!(f, "cannot divide by zero"),
            ErrType::FileMissing(path) => write!(f, "file missing: {path}"),
            ErrType::TypeMismatch(msg) => write!(f, "type mismatch: {msg}"),
            ErrType::TypeConversion { from, to } => write!(f, "failed to convert {from} to {to}"),
            ErrType::NoneReason(reason) => write!(f, "{reason}"),
            ErrType::Custom(msg) => write!(f, "{msg}"),
        }
    }
}

impl From<std::io::Error> for ErrType {
    fn from(e: std::io::Error) -> Self {
        ErrType::NoneReason(NoneReason::StdIo(e.to_string()))
    }
}

impl From<std::num::ParseIntError> for ErrType {
    fn from(e: std::num::ParseIntError) -> Self {
        ErrType::NoneReason(NoneReason::StdParse(e.to_string()))
    }
}

// ─────────────────────────────────────────────
// FuncReturnType — the DECLARED shape (no data, just describes it)
// ─────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub enum FuncReturnType {
    Void,
    Int32,
    UInt32,
    Int128,
    Float,
    String,
    Bool,
    Int32Vec,
    Float64Vec,
    Custom(String),
    Optional(Box<FuncReturnType>),          // e.g. Option<i32>
    Fallible(Box<FuncReturnType>, ErrType), // e.g. Result<i32, ErrType>
}

// ─────────────────────────────────────────────
// FuncReturnValue — the ACTUAL runtime value
// ─────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub enum FuncReturnValue {
    Void,
    Int32(i32),
    UInt32(u32),
    Int128(i128),
    Float(f64),
    String(String),
    Bool(bool),
    Int32Vec(Vec<i32>),
    Float64Vec(Vec<f64>),
    Custom(String),

}
pub enum FuncArgs {
        Tuple(),
        // Array[],
}
// ─────────────────────────────────────────────
// FuncArgType — for describing function arguments
// ─────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub enum FuncArgType {
    Int32,
    Float,
    String,
    Bool,
    Custom(String),
}

// ─────────────────────────────────────────────
// FuncIdentifier — ties it all together
// ─────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub struct FuncIdentifier {
    pub name: String,
    pub return_type: FuncReturnType,
    pub return_value: Result<FuncReturnValue, ErrType>,
    pub args_type: Vec<FuncArgType>,
    pub number_of_args: usize,
    pub args: ,
}

impl FuncIdentifier {
    pub fn validate(&self) -> Result<(), String> {
        let value = match &self.return_value {
            Ok(v) => v,
            Err(e) => return Err(format!("Function '{}' failed: {e}", self.name)),
        };

        let matches = match (&self.return_type, value) {
            (FuncReturnType::Void, FuncReturnValue::Void) => true,
            (FuncReturnType::Int32, FuncReturnValue::Int32(_)) => true,
            (FuncReturnType::UInt32, FuncReturnValue::UInt32(_)) => true,
            (FuncReturnType::Int128, FuncReturnValue::Int128(_)) => true,
            (FuncReturnType::Float, FuncReturnValue::Float(_)) => true,
            (FuncReturnType::String, FuncReturnValue::String(_)) => true,
            (FuncReturnType::Bool, FuncReturnValue::Bool(_)) => true,
            (FuncReturnType::Int32Vec, FuncReturnValue::Int32Vec(_)) => true,
            (FuncReturnType::Float64Vec, FuncReturnValue::Float64Vec(_)) => true,
            (FuncReturnType::Custom(t), FuncReturnValue::Custom(v)) => t == v,
            _ => false,
        };

        if matches {
            Ok(())
        } else {
            Err(format!(
                "Type mismatch in '{}': declared {:?}, got {:?}",
                self.name, self.return_type, value
            ))
        }
    }
}
 */
pub enum funcReturnType {
    INT(Int),
    UINT(UInt),
    FLOAT(Float),
}
pub enum Float {
    Float32(f32),
    Float64(f64),
}
pub enum Int {
    Int32(i32),
    Iint64(i64),
    Int128(i128),
    Int16(i16),
    Int8(i8),
}

pub enum UInt {
    UInt32(u32),
    UInt64(u64),
    UInt128(u128),
    UInt16(u16),
    UInt8(u8),
}
