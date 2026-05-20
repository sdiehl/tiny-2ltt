use std::fmt;
use std::rc::Rc;

pub type Name = Rc<str>;

#[must_use]
pub fn name(s: &str) -> Name {
    Rc::from(s)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Nat,
    Bool,
    Arr(Rc<Self>, Rc<Self>),
    Code(Rc<Self>),
}

impl Ty {
    #[must_use]
    pub fn arr(a: Self, b: Self) -> Self {
        Self::Arr(Rc::new(a), Rc::new(b))
    }

    #[must_use]
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
    #[must_use]
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
    Lam(Name, Option<Rc<Ty>>, Rc<Self>),
    App(Rc<Self>, Rc<Self>),
    Let(Name, Option<Rc<Ty>>, Rc<Self>, Rc<Self>),
    Bin(BinOp, Rc<Self>, Rc<Self>),
    If(Rc<Self>, Rc<Self>, Rc<Self>),
    Quote(Rc<Self>),
    Splice(Rc<Self>),
    Ann(Rc<Self>, Rc<Ty>),
}

#[must_use]
pub fn fold_lams(params: Vec<(Name, Option<Rc<Ty>>)>, body: Tm) -> Tm {
    params
        .into_iter()
        .rev()
        .fold(body, |acc, (nm, ann)| Tm::Lam(nm, ann, Rc::new(acc)))
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
