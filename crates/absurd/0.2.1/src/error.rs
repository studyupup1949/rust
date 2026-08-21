pub type Result<T> = std::result::Result<T, Error>;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    Debug(String),
    Io(std::io::Error),

    FailedToHandleComponent(&'static str),
    ControllerAlreadyExists(String),
    ModelAlreadyExists(String),
    StoreAlreadyExists(String),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<&'static str> for Error {
    fn from(value: &'static str) -> Self {
        Self::Debug(value.to_string())
    }
}
impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Debug(value)
    }
}
