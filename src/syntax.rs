use std::fmt;
use std::rc::Rc;

pub type Name = Rc<str>;

pub fn name(s: &str) -> Name {
    Rc::from(s)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Nat,
    Bool,
    Arr(Rc<Ty>, Rc<Ty>),
    Code(Rc<Ty>),
}

impl Ty {
    pub fn arr(a: Self, b: Self) -> Self {
        Self::Arr(Rc::new(a), Rc::new(b))
    }

    pub fn code(a: Self) -> Self {
        Self::Code(Rc::new(a))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Eq,
}

impl BinOp {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Eq => "==",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Tm {
    Var(Name),
    NatLit(i64),
    BoolLit(bool),
    Lam(Name, Option<Rc<Ty>>, Rc<Tm>),
    App(Rc<Tm>, Rc<Tm>),
    Let(Name, Option<Rc<Ty>>, Rc<Tm>, Rc<Tm>),
    Bin(BinOp, Rc<Tm>, Rc<Tm>),
    If(Rc<Tm>, Rc<Tm>, Rc<Tm>),
    Quote(Rc<Tm>),
    Splice(Rc<Tm>),
    Ann(Rc<Tm>, Rc<Ty>),
}

#[derive(Debug, Clone)]
pub enum Decl {
    Let(Name, Rc<Ty>, Rc<Tm>),
    Eval(Rc<Tm>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Zero,
    One,
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zero => write!(f, "0"),
            Self::One => write!(f, "1"),
        }
    }
}
