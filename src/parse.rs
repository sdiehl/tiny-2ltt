use std::rc::Rc;

use crate::errors::{Error, Result};
use crate::lexer::{lex, Spanned, Token};
use crate::syntax::{name, BinOp, Decl, Tm, Ty};

pub fn parse_program(src: &str) -> Result<Vec<Decl>> {
    let toks = lex(src)?;
    let mut p = Parser::new(toks);
    let mut decls = Vec::new();
    while !p.eof() {
        decls.push(p.decl()?);
    }
    Ok(decls)
}

pub fn parse_expr(src: &str) -> Result<Tm> {
    let toks = lex(src)?;
    let mut p = Parser::new(toks);
    let e = p.expr()?;
    if !p.eof() {
        return Err(Error::Parse(format!(
            "trailing tokens after expression: {:?}",
            p.peek()
        )));
    }
    Ok(e)
}

struct Parser {
    toks: Vec<Spanned>,
    pos: usize,
}

impl Parser {
    fn new(toks: Vec<Spanned>) -> Self {
        Self { toks, pos: 0 }
    }

    fn eof(&self) -> bool {
        self.pos >= self.toks.len()
    }

    fn peek(&self) -> Option<&Token> {
        self.toks.get(self.pos).map(|s| &s.tok)
    }

    fn advance(&mut self) -> Option<Token> {
        let t = self.toks.get(self.pos)?.tok.clone();
        self.pos += 1;
        Some(t)
    }

    fn expect(&mut self, t: &Token) -> Result<()> {
        match self.peek() {
            Some(tk) if std::mem::discriminant(tk) == std::mem::discriminant(t) => {
                self.advance();
                Ok(())
            }
            other => Err(Error::Parse(format!("expected {t:?}, got {other:?}"))),
        }
    }

    fn eat(&mut self, t: &Token) -> bool {
        if matches!(self.peek(), Some(tk) if std::mem::discriminant(tk) == std::mem::discriminant(t))
        {
            self.advance();
            true
        } else {
            false
        }
    }

    fn decl(&mut self) -> Result<Decl> {
        match self.peek() {
            Some(Token::Let) => {
                self.advance();
                let n = self.ident()?;
                self.expect(&Token::Colon)?;
                let ty = self.ty()?;
                self.expect(&Token::Eq)?;
                let body = self.expr()?;
                self.expect(&Token::Semi)?;
                Ok(Decl::Let(name(&n), Rc::new(ty), Rc::new(body)))
            }
            Some(Token::Eval) => {
                self.advance();
                let e = self.expr()?;
                self.expect(&Token::Semi)?;
                Ok(Decl::Eval(Rc::new(e)))
            }
            other => Err(Error::Parse(format!(
                "expected `let` or `eval`, got {other:?}"
            ))),
        }
    }

    fn ident(&mut self) -> Result<String> {
        match self.advance() {
            Some(Token::Ident(s)) => Ok(s),
            other => Err(Error::Parse(format!("expected identifier, got {other:?}"))),
        }
    }

    fn ty(&mut self) -> Result<Ty> {
        let a = self.ty_atom()?;
        if self.eat(&Token::Arrow) {
            let b = self.ty()?;
            Ok(Ty::arr(a, b))
        } else {
            Ok(a)
        }
    }

    fn ty_atom(&mut self) -> Result<Ty> {
        match self.peek() {
            Some(Token::NatTy) => {
                self.advance();
                Ok(Ty::Nat)
            }
            Some(Token::BoolTy) => {
                self.advance();
                Ok(Ty::Bool)
            }
            Some(Token::Code) => {
                self.advance();
                let inner = self.ty_atom()?;
                Ok(Ty::code(inner))
            }
            Some(Token::LParen) => {
                self.advance();
                let t = self.ty()?;
                self.expect(&Token::RParen)?;
                Ok(t)
            }
            other => Err(Error::Parse(format!("expected type, got {other:?}"))),
        }
    }

    fn expr(&mut self) -> Result<Tm> {
        self.expr_let()
    }

    fn expr_let(&mut self) -> Result<Tm> {
        match self.peek() {
            Some(Token::Let) => {
                self.advance();
                let n = self.ident()?;
                let ann = if self.eat(&Token::Colon) {
                    Some(Rc::new(self.ty()?))
                } else {
                    None
                };
                self.expect(&Token::Eq)?;
                let v = self.expr()?;
                self.expect(&Token::In)?;
                let b = self.expr()?;
                Ok(Tm::Let(name(&n), ann, Rc::new(v), Rc::new(b)))
            }
            Some(Token::If) => {
                self.advance();
                let c = self.expr()?;
                self.expect(&Token::Then)?;
                let t = self.expr()?;
                self.expect(&Token::Else)?;
                let e = self.expr()?;
                Ok(Tm::If(Rc::new(c), Rc::new(t), Rc::new(e)))
            }
            Some(Token::Lambda | Token::Fn) => {
                self.advance();
                let mut params = Vec::new();
                while !matches!(self.peek(), Some(Token::Dot | Token::FatArrow)) {
                    let has_paren = self.eat(&Token::LParen);
                    let n = self.ident()?;
                    let ann = if self.eat(&Token::Colon) {
                        Some(Rc::new(self.ty()?))
                    } else {
                        None
                    };
                    if has_paren {
                        self.expect(&Token::RParen)?;
                    }
                    params.push((n, ann));
                }
                if !self.eat(&Token::Dot) {
                    self.expect(&Token::FatArrow)?;
                }
                let mut body = self.expr()?;
                for (n, ann) in params.into_iter().rev() {
                    body = Tm::Lam(name(&n), ann, Rc::new(body));
                }
                Ok(body)
            }
            _ => self.expr_ann(),
        }
    }

    fn expr_ann(&mut self) -> Result<Tm> {
        let e = self.expr_cmp()?;
        if self.eat(&Token::Colon) {
            let t = self.ty()?;
            Ok(Tm::Ann(Rc::new(e), Rc::new(t)))
        } else {
            Ok(e)
        }
    }

    fn expr_cmp(&mut self) -> Result<Tm> {
        let mut lhs = self.expr_add()?;
        while matches!(self.peek(), Some(Token::EqEq)) {
            self.advance();
            let rhs = self.expr_add()?;
            lhs = Tm::Bin(BinOp::Eq, Rc::new(lhs), Rc::new(rhs));
        }
        Ok(lhs)
    }

    fn expr_add(&mut self) -> Result<Tm> {
        let mut lhs = self.expr_mul()?;
        loop {
            let op = match self.peek() {
                Some(Token::Plus) => BinOp::Add,
                Some(Token::Minus) => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let rhs = self.expr_mul()?;
            lhs = Tm::Bin(op, Rc::new(lhs), Rc::new(rhs));
        }
        Ok(lhs)
    }

    fn expr_mul(&mut self) -> Result<Tm> {
        let mut lhs = self.expr_app()?;
        while matches!(self.peek(), Some(Token::Star)) {
            self.advance();
            let rhs = self.expr_app()?;
            lhs = Tm::Bin(BinOp::Mul, Rc::new(lhs), Rc::new(rhs));
        }
        Ok(lhs)
    }

    fn expr_app(&mut self) -> Result<Tm> {
        let mut head = self.expr_pre()?;
        while self.is_atom_start() {
            let arg = self.expr_pre()?;
            head = Tm::App(Rc::new(head), Rc::new(arg));
        }
        Ok(head)
    }

    fn expr_pre(&mut self) -> Result<Tm> {
        match self.peek() {
            Some(Token::Tilde) => {
                self.advance();
                let e = self.expr_pre()?;
                Ok(Tm::Splice(Rc::new(e)))
            }
            Some(Token::Lt) => {
                self.advance();
                let e = self.expr()?;
                self.expect(&Token::Gt)?;
                Ok(Tm::Quote(Rc::new(e)))
            }
            _ => self.expr_atom(),
        }
    }

    fn is_atom_start(&self) -> bool {
        matches!(
            self.peek(),
            Some(
                Token::Ident(_)
                    | Token::Num(_)
                    | Token::True
                    | Token::False
                    | Token::LParen
                    | Token::Tilde
                    | Token::Lt
            )
        )
    }

    fn expr_atom(&mut self) -> Result<Tm> {
        match self.peek() {
            Some(Token::Num(n)) => {
                let v = *n;
                self.advance();
                Ok(Tm::NatLit(v))
            }
            Some(Token::True) => {
                self.advance();
                Ok(Tm::BoolLit(true))
            }
            Some(Token::False) => {
                self.advance();
                Ok(Tm::BoolLit(false))
            }
            Some(Token::Ident(s)) => {
                let n = s.clone();
                self.advance();
                Ok(Tm::Var(name(&n)))
            }
            Some(Token::LParen) => {
                self.advance();
                let e = self.expr()?;
                self.expect(&Token::RParen)?;
                Ok(e)
            }
            other => Err(Error::Parse(format!("expected atom, got {other:?}"))),
        }
    }
}
