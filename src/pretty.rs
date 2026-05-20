use std::fmt::{self, Write};

use crate::syntax::{BinOp, Tm, Ty};

#[must_use]
pub fn ty(t: &Ty) -> String {
    let mut s = String::new();
    fmt_ty(t, 0, &mut s).unwrap();
    s
}

#[must_use]
pub fn tm(t: &Tm) -> String {
    let mut s = String::new();
    fmt_tm(t, 0, &mut s).unwrap();
    s
}

fn fmt_ty(t: &Ty, prec: u8, out: &mut String) -> fmt::Result {
    match t {
        Ty::Nat => write!(out, "Nat"),
        Ty::Bool => write!(out, "Bool"),
        Ty::Code(inner) => {
            let wrap = prec > 10;
            if wrap {
                write!(out, "(")?;
            }
            write!(out, "Code ")?;
            fmt_ty(inner, 11, out)?;
            if wrap {
                write!(out, ")")?;
            }
            Ok(())
        }
        Ty::Arr(a, b) => {
            let wrap = prec > 0;
            if wrap {
                write!(out, "(")?;
            }
            fmt_ty(a, 1, out)?;
            write!(out, " -> ")?;
            fmt_ty(b, 0, out)?;
            if wrap {
                write!(out, ")")?;
            }
            Ok(())
        }
    }
}

const P_BOT: u8 = 0;
const P_EQ: u8 = 10;
const P_ADD: u8 = 20;
const P_MUL: u8 = 30;
const P_APP: u8 = 40;
const P_PRE: u8 = 50;

const fn op_prec(op: BinOp) -> (u8, u8, u8) {
    match op {
        BinOp::Eq => (P_EQ, P_EQ + 1, P_EQ + 1),
        BinOp::Add | BinOp::Sub => (P_ADD, P_ADD, P_ADD + 1),
        BinOp::Mul => (P_MUL, P_MUL, P_MUL + 1),
    }
}

fn fmt_tm(t: &Tm, prec: u8, out: &mut String) -> fmt::Result {
    match t {
        Tm::Var(n) => write!(out, "{n}"),
        Tm::NatLit(n) => write!(out, "{n}"),
        Tm::BoolLit(b) => write!(out, "{b}"),
        Tm::Lam(n, _, b) => {
            let wrap = prec > P_BOT;
            if wrap {
                write!(out, "(")?;
            }
            write!(out, "\\{n}. ")?;
            fmt_tm(b, P_BOT, out)?;
            if wrap {
                write!(out, ")")?;
            }
            Ok(())
        }
        Tm::App(f, a) => {
            let wrap = prec > P_APP;
            if wrap {
                write!(out, "(")?;
            }
            fmt_tm(f, P_APP, out)?;
            write!(out, " ")?;
            fmt_tm(a, P_APP + 1, out)?;
            if wrap {
                write!(out, ")")?;
            }
            Ok(())
        }
        Tm::Let(n, _, v, b) => {
            let wrap = prec > P_BOT;
            if wrap {
                write!(out, "(")?;
            }
            write!(out, "let {n} = ")?;
            fmt_tm(v, P_BOT, out)?;
            write!(out, " in ")?;
            fmt_tm(b, P_BOT, out)?;
            if wrap {
                write!(out, ")")?;
            }
            Ok(())
        }
        Tm::Bin(op, a, b) => {
            let (p, lp, rp) = op_prec(*op);
            let wrap = prec > p;
            if wrap {
                write!(out, "(")?;
            }
            fmt_tm(a, lp, out)?;
            write!(out, " {} ", op.as_str())?;
            fmt_tm(b, rp, out)?;
            if wrap {
                write!(out, ")")?;
            }
            Ok(())
        }
        Tm::If(c, th, el) => {
            let wrap = prec > P_BOT;
            if wrap {
                write!(out, "(")?;
            }
            write!(out, "if ")?;
            fmt_tm(c, P_BOT, out)?;
            write!(out, " then ")?;
            fmt_tm(th, P_BOT, out)?;
            write!(out, " else ")?;
            fmt_tm(el, P_BOT, out)?;
            if wrap {
                write!(out, ")")?;
            }
            Ok(())
        }
        Tm::Quote(e) => {
            write!(out, "<")?;
            fmt_tm(e, P_BOT, out)?;
            write!(out, ">")
        }
        Tm::Splice(e) => {
            write!(out, "~")?;
            fmt_tm(e, P_PRE, out)?;
            Ok(())
        }
        Tm::Ann(e, ty) => {
            let wrap = prec > P_BOT;
            if wrap {
                write!(out, "(")?;
            }
            fmt_tm(e, P_EQ + 1, out)?;
            write!(out, " : ")?;
            fmt_ty(ty, 0, out)?;
            if wrap {
                write!(out, ")")?;
            }
            Ok(())
        }
    }
}
