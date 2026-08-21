//! User-friendly status reporting.
//!
//! This library provides a simple API for advising the end-user about program
//! status to [`stderr`](std::io::stderr).
//!
//! # Usage
//!
//! The primary use of the advise crate is through the five advising macros:
//! [`error!`], [`warn!`], [`info!`], [`debug!`], and [`trace!`], corresponding
//! to the respective levels from the [log] crate.
//!
//! Users can also implement their own custom message prefix formats by
//! implementing [`Render`] on a struct (called a tag), then advising using
//! [`advise!`].
//!
//! ## Examples
//!
//! ```
//! use advise::{debug, error, info, trace, warn};
//!
//! // Advise the user at various message levels
//! error!("It broke. Don't say you weren't warned!");
//! warn!("Are you sure you want to do that?");
//! info!("For your information...");
//! debug!("This should help you find the problem.");
//! trace!("Here's a really specific thing that happened...");
//! ```
//!
//! Produces:
//!
//! <pre>
//! <b><span style="color:red">error</span>:</b> It broke. Don't say you weren't warned!
//! <span style="color:orange">warn</span><b>:</b> Are you sure you want to do that?
//! <span style="color:green">info</span><b>:</b> For your information...
//! <span style="color:blue">debug</span><b>:</b> This should help you find the problem.
//! <span style="color:magenta">trace</span><b>:</b> Here's a really specific thing that happened...
//! </pre>

#![warn(clippy::pedantic)]

use std::fmt::Display;

use anstyle::{AnsiColor, Style};
use log::Level;

#[macro_use]
mod macros;

pub use {anstream, log};

/// Advisable tag and message.
///
/// When [displayed][`Display`], this struct will render according to the
/// supplied tag and message.
#[derive(Debug)]
pub struct Advise<T: Render> {
    tag: T,
    msg: String,
}

impl<T: Render> Advise<T> {
    /// Constructs a new `Advise`.
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(tag: T, msg: impl ToString) -> Self {
        Self {
            tag,
            msg: msg.to_string(),
        }
    }
}

impl<T: Render> Display for Advise<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{tag}{sep}{msg}",
            tag = self.tag.render(),
            sep = Separator::default().render(),
            msg = self.msg
        )
    }
}

/// Specifies how to render as a tag.
pub trait Render {
    /// Converts the given value to a style.
    fn style(&self) -> Style;

    /// Displays the given value without styling.
    fn label(&self) -> impl Display;

    /// Renders the given value as a colorized `String`.
    fn render(&self) -> String {
        format!(
            "{style}{label}{style:#}",
            style = self.style(),
            label = self.label()
        )
    }
}

impl Render for Level {
    fn style(&self) -> Style {
        match self {
            Level::Error => AnsiColor::Red.on_default().bold(),
            Level::Warn => AnsiColor::Yellow.on_default(),
            Level::Info => AnsiColor::Green.on_default(),
            Level::Debug => AnsiColor::Blue.on_default(),
            Level::Trace => AnsiColor::Magenta.on_default(),
        }
    }

    fn label(&self) -> impl Display {
        self.to_string().to_lowercase()
    }
}

/// Message separator.
#[derive(Debug, Default)]
enum Separator {
    #[default]
    Colon,
}

impl Render for Separator {
    fn style(&self) -> Style {
        Style::new().bold()
    }

    fn label(&self) -> impl Display {
        match self {
            Separator::Colon => ": ",
        }
    }
}
