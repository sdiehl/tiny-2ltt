#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

pub mod driver;
pub mod elab;
pub mod errors;
pub mod eval;
pub mod lexer;
pub mod parse;
pub mod pretty;
pub mod syntax;

pub use driver::run_program;
pub use errors::{Error, Result};
