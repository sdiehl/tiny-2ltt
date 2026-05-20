use lalrpop_util::ParseError as LalrParseError;

use crate::errors::{Error, Result};
use crate::lexer::{Lexer, LexicalError, Token};
use crate::syntax::{Decl, Tm};

#[allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    dead_code,
    unreachable_pub
)]
mod parser_impl {
    use lalrpop_util::lalrpop_mod;
    lalrpop_mod!(pub parser);
}
use parser_impl::parser;

pub fn parse_program(src: &str) -> Result<Vec<Decl>> {
    parser::ProgramParser::new()
        .parse(Lexer::new(src))
        .map_err(convert)
}

pub fn parse_expr(src: &str) -> Result<Tm> {
    parser::ExprParser::new()
        .parse(Lexer::new(src))
        .map_err(convert)
}

fn convert(err: LalrParseError<usize, Token, LexicalError>) -> Error {
    Error::Parse(match err {
        LalrParseError::InvalidToken { location } => {
            format!("invalid token at offset {location}")
        }
        LalrParseError::UnrecognizedEof { location, expected } => {
            format!("unexpected end of input at {location}, expected one of {expected:?}")
        }
        LalrParseError::UnrecognizedToken {
            token: (start, tok, end),
            expected,
        } => format!("unexpected `{tok}` at {start}..{end}, expected one of {expected:?}"),
        LalrParseError::ExtraToken {
            token: (start, tok, end),
        } => format!("extra token `{tok}` at {start}..{end}"),
        LalrParseError::User { error } => error.to_string(),
    })
}
