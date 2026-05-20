use std::collections::HashMap;
use std::rc::Rc;

use crate::errors::{Error, Result};
use crate::syntax::{name, BinOp, Name, Tm};

#[derive(Debug, Clone)]
pub enum Val {
    Nat(i64),
    Bool(bool),
    Lam(Rc<Closure>),
    Code(Rc<Tm>),
}

#[derive(Debug, Clone)]
pub struct Closure {
    pub env: Env,
    pub param: Name,
    pub body: Rc<Tm>,
}

#[derive(Debug, Clone)]
enum Entry {
    Val(Val),
    CodeVar(Name),
}

#[derive(Debug, Clone, Default)]
pub struct Env {
    scope: Vec<(Name, Entry)>,
}

impl Env {
    pub fn new() -> Self {
        Self::default()
    }

    fn push_val(&self, n: Name, v: Val) -> Self {
        let mut e = self.clone();
        e.scope.push((n, Entry::Val(v)));
        e
    }

    fn push_code(&self, n: Name, fresh: Name) -> Self {
        let mut e = self.clone();
        e.scope.push((n, Entry::CodeVar(fresh)));
        e
    }

    fn lookup(&self, n: &str) -> Option<&Entry> {
        self.scope
            .iter()
            .rev()
            .find(|(k, _)| k.as_ref() == n)
            .map(|(_, e)| e)
    }
}

#[derive(Debug, Default)]
pub struct Globals {
    map: HashMap<String, Val>,
}

impl Globals {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind(&mut self, n: Name, v: Val) {
        self.map.insert(n.to_string(), v);
    }

    fn lookup(&self, n: &str) -> Option<&Val> {
        self.map.get(n)
    }
}

#[derive(Debug, Default)]
pub struct Gen {
    counter: u64,
}

impl Gen {
    pub fn new() -> Self {
        Self::default()
    }

    fn fresh(&mut self, base: &str) -> Name {
        let n = self.counter;
        self.counter += 1;
        let trimmed = base
            .trim_end_matches(char::is_numeric)
            .trim_end_matches('_');
        let base = if trimmed.is_empty() { "x" } else { trimmed };
        name(&format!("{base}_{n}"))
    }
}

pub fn eval0(gl: &Globals, g: &mut Gen, env: &Env, t: &Tm) -> Result<Val> {
    match t {
        Tm::Var(n) => match env.lookup(n) {
            Some(Entry::Val(v)) => Ok(v.clone()),
            Some(Entry::CodeVar(_)) => Err(Error::Runtime(format!(
                "stage-1 variable `{n}` used at stage 0"
            ))),
            None => gl
                .lookup(n)
                .cloned()
                .ok_or_else(|| Error::Runtime(format!("unbound variable `{n}`"))),
        },
        Tm::NatLit(n) => Ok(Val::Nat(*n)),
        Tm::BoolLit(b) => Ok(Val::Bool(*b)),
        Tm::Lam(n, _, b) => Ok(Val::Lam(Rc::new(Closure {
            env: env.clone(),
            param: n.clone(),
            body: Rc::clone(b),
        }))),
        Tm::App(f, a) => {
            let vf = eval0(gl, g, env, f)?;
            let va = eval0(gl, g, env, a)?;
            apply(gl, g, vf, va)
        }
        Tm::Let(n, _, v, b) => {
            let vv = eval0(gl, g, env, v)?;
            let env2 = env.push_val(n.clone(), vv);
            eval0(gl, g, &env2, b)
        }
        Tm::Bin(op, a, b) => {
            let va = eval0(gl, g, env, a)?;
            let vb = eval0(gl, g, env, b)?;
            bin_val(*op, &va, &vb)
        }
        Tm::If(c, th, el) => match eval0(gl, g, env, c)? {
            Val::Bool(true) => eval0(gl, g, env, th),
            Val::Bool(false) => eval0(gl, g, env, el),
            _ => Err(Error::Runtime("if on non-bool".into())),
        },
        Tm::Quote(e) => {
            let body = eval1(gl, g, env, e)?;
            Ok(Val::Code(Rc::new(body)))
        }
        Tm::Splice(_) => Err(Error::Runtime("splice at stage 0".into())),
        Tm::Ann(e, _) => eval0(gl, g, env, e),
    }
}

fn apply(gl: &Globals, g: &mut Gen, f: Val, a: Val) -> Result<Val> {
    match f {
        Val::Lam(cl) => {
            let env2 = cl.env.push_val(cl.param.clone(), a);
            eval0(gl, g, &env2, &cl.body)
        }
        _ => Err(Error::Runtime("applying non-function".into())),
    }
}

fn bin_val(op: BinOp, a: &Val, b: &Val) -> Result<Val> {
    match (a, b) {
        (Val::Nat(x), Val::Nat(y)) => Ok(match op {
            BinOp::Add => Val::Nat(x + y),
            BinOp::Sub => Val::Nat(x - y),
            BinOp::Mul => Val::Nat(x * y),
            BinOp::Eq => Val::Bool(x == y),
        }),
        _ => Err(Error::Runtime("binary op on non-nat".into())),
    }
}

fn eval1(gl: &Globals, g: &mut Gen, env: &Env, t: &Tm) -> Result<Tm> {
    match t {
        Tm::Var(n) => match env.lookup(n) {
            Some(Entry::CodeVar(fresh)) => Ok(Tm::Var(fresh.clone())),
            Some(Entry::Val(_)) => Err(Error::Runtime(format!(
                "stage-0 variable `{n}` used at stage 1"
            ))),
            None => Err(Error::Runtime(format!("unbound stage-1 variable `{n}`"))),
        },
        Tm::NatLit(n) => Ok(Tm::NatLit(*n)),
        Tm::BoolLit(b) => Ok(Tm::BoolLit(*b)),
        Tm::Lam(n, ann, b) => {
            let f = g.fresh(n);
            let env2 = env.push_code(n.clone(), f.clone());
            let body = eval1(gl, g, &env2, b)?;
            Ok(Tm::Lam(f, ann.clone(), Rc::new(body)))
        }
        Tm::App(f, a) => Ok(Tm::App(
            Rc::new(eval1(gl, g, env, f)?),
            Rc::new(eval1(gl, g, env, a)?),
        )),
        Tm::Let(n, ann, v, b) => {
            let nv = eval1(gl, g, env, v)?;
            let f = g.fresh(n);
            let env2 = env.push_code(n.clone(), f.clone());
            let nb = eval1(gl, g, &env2, b)?;
            Ok(Tm::Let(f, ann.clone(), Rc::new(nv), Rc::new(nb)))
        }
        Tm::Bin(op, a, b) => Ok(Tm::Bin(
            *op,
            Rc::new(eval1(gl, g, env, a)?),
            Rc::new(eval1(gl, g, env, b)?),
        )),
        Tm::If(c, th, el) => Ok(Tm::If(
            Rc::new(eval1(gl, g, env, c)?),
            Rc::new(eval1(gl, g, env, th)?),
            Rc::new(eval1(gl, g, env, el)?),
        )),
        Tm::Quote(_) => Err(Error::Runtime("nested quote at stage 1".into())),
        Tm::Splice(e) => match eval0(gl, g, env, e)? {
            Val::Code(t) => Ok((*t).clone()),
            _ => Err(Error::Runtime("splice of non-code".into())),
        },
        Tm::Ann(e, _) => eval1(gl, g, env, e),
    }
}

pub fn val_to_tm(v: &Val) -> Tm {
    match v {
        Val::Nat(n) => Tm::NatLit(*n),
        Val::Bool(b) => Tm::BoolLit(*b),
        Val::Code(t) => Tm::Quote(Rc::clone(t)),
        Val::Lam(cl) => Tm::Lam(cl.param.clone(), None, Rc::clone(&cl.body)),
    }
}
