use core::{any::Any, fmt::Display};

pub mod cancelable;

#[derive(Debug)]
/// [::std::thread::JoinHandle::join] returns a `Result<T, Box<dyn Any + Send + 'static>>` because pre-2021
/// [::core::panic!] supported everything. Though nowadays, outside of a few niche usecases, panic messages will always
/// be of type `String` or `&'static str`.
pub enum ThreadPanicError {
	Str(&'static str),
	String(String),
	// I could put a `Custom(T)` here where where `T: Display + Any + Send + 'static`, but typed panics aren't really
	// a thing, and usually represent exceptionally irrecoverable failures with reasons that only make sense to
	// programmers. Typed results are the things that are supposed to give digestible failure states, so we don't
	// needed to revisit that here.
	Unknown,
}
impl Display for ThreadPanicError {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			ThreadPanicError::Str(str) => f.write_str(str),
			ThreadPanicError::String(string) => f.write_str(string),
			ThreadPanicError::Unknown => f.write_str("unknown panic reason"),
		}
	}
}
impl core::error::Error for ThreadPanicError {}

#[cfg(feature = "app")]
impl crate::providers::ProvidesExitCode for ThreadPanicError {
	fn exit_code(&self) -> std::process::ExitCode {
		crate::app::consts::EX_SOFTWARE.into()
	}
}

impl From<Box<dyn Any + Send + 'static>> for ThreadPanicError {
	fn from(value: Box<dyn Any + Send + 'static>) -> Self {
		match value.downcast::<&str>() {
			Ok(panic_str) => Self::Str(*panic_str),
			Err(value) => match value.downcast::<String>() {
				Ok(panic_string) => Self::String(*panic_string),
				Err(_) => Self::Unknown,
			},
		}
	}
}

pub trait ResultIntoThreadPanicError<T> {
	fn map_err_panic(self) -> Result<T, ThreadPanicError>;
}
impl<T> ResultIntoThreadPanicError<T> for std::thread::Result<T> {
	fn map_err_panic(self) -> Result<T, ThreadPanicError> {
		match self {
			Ok(inner) => Ok(inner),
			Err(inner) => Err(inner.into()),
		}
	}
}
