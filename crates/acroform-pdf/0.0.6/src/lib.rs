#![allow(non_camel_case_types)]  /* TODO temporary becaues of pdf_derive */
#![allow(unused_doc_comments)] // /* TODO temporary because of err.rs */
#![allow(clippy::len_zero, clippy::should_implement_trait, clippy::manual_map, clippy::from_over_into)]

#[macro_use] extern crate acroform_pdf_derive as pdf_derive;
#[macro_use] extern crate snafu;
#[macro_use] extern crate log;

#[macro_use]
pub mod error;
pub mod object;
pub mod xref;
pub mod primitive;
pub mod file;
pub mod backend;
pub mod parser;
pub mod any;

// Stream encoding/decoding
pub mod enc;

pub use crate::error::PdfError;
