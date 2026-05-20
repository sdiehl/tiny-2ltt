#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::many_single_char_names,
    clippy::similar_names,
    clippy::module_name_repetitions,
    clippy::too_many_lines,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::match_same_arms,
    clippy::needless_pass_by_value,
    clippy::option_if_let_else,
    clippy::use_self,
    clippy::wildcard_imports,
    clippy::single_match_else
)]

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
