use logos::Logos;

use crate::errors::{Error, Result};

#[derive(Logos, Debug, Clone, PartialEq, Eq)]
#[logos(skip r"[ \t\r\n\f]+", skip(r"--[^\n]*", allow_greedy = true))]
pub enum Token {
    #[token("let")]
    Let,
    #[token("in")]
    In,
    #[token("if")]
    If,
    #[token("then")]
    Then,
    #[token("else")]
    Else,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("eval")]
    Eval,
    #[token("Nat")]
    NatTy,
    #[token("Bool")]
    BoolTy,
    #[token("Code")]
    Code,
    #[token("fn")]
    Fn,
    #[token("=>")]
    FatArrow,
    #[token("->")]
    Arrow,
    #[token("=")]
    Eq,
    #[token("==")]
    EqEq,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token(":")]
    Colon,
    #[token(";")]
    Semi,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("\\")]
    Lambda,
    #[token("~")]
    Tilde,
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Num(i64),
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_']*", |lex| lex.slice().to_string())]
    Ident(String),
}

#[derive(Debug, Clone)]
pub struct Spanned {
    pub tok: Token,
    pub span: (usize, usize),
}

pub fn lex(src: &str) -> Result<Vec<Spanned>> {
    let mut out = Vec::new();
    for (tok, span) in Token::lexer(src).spanned() {
        let t = tok.map_err(|()| Error::Parse(format!("unexpected character at {span:?}")))?;
        out.push(Spanned {
            tok: t,
            span: (span.start, span.end),
        });
    }
    Ok(out)
}
