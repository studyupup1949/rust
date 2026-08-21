use std::fmt;

/// A constant value
#[derive(Debug, Default, Clone, PartialEq)]
pub enum Constant {
    String(String),
    Integer(String),
    Double(String),
    Boolean(bool),
    /// data type and value (e.g. "192.168.1.1":ipAddress)
    Custom(CustomType, String),
    #[default]
    Undefined,
}

/// Custom type short names, which need to be looked up.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CustomType {
    pub name: String,
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Constant::String(s) => write!(f, "{s:?}"),
            Constant::Integer(i) => write!(f, "{i}"),
            Constant::Double(d) => write!(f, "{d}"),
            Constant::Boolean(b) => write!(f, "{b}"),
            _ => write!(f, "unhandled"),
        }
    }
}
