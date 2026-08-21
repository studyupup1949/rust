pub type Result<T> = std::result::Result<T, Error>;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
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
