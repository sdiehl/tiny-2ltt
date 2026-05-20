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
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn push_val(&self, nm: Name, val: Val) -> Self {
        let mut env = self.clone();
        env.scope.push((nm, Entry::Val(val)));
        env
    }

    fn push_code(&self, nm: Name, fresh: Name) -> Self {
        let mut env = self.clone();
        env.scope.push((nm, Entry::CodeVar(fresh)));
        env
    }

    fn lookup(&self, nm: &str) -> Option<&Entry> {
        self.scope
            .iter()
            .rev()
            .find(|(k, _)| k.as_ref() == nm)
            .map(|(_, e)| e)
    }
}

#[derive(Debug, Default)]
pub struct Globals {
    map: HashMap<String, Val>,
}

impl Globals {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind(&mut self, nm: &Name, val: Val) {
        self.map.insert(nm.to_string(), val);
    }

    fn lookup(&self, nm: &str) -> Option<&Val> {
        self.map.get(nm)
    }
}

#[derive(Debug, Default)]
pub struct Gen {
    counter: u64,
}

impl Gen {
    #[must_use]
    pub const fn new() -> Self {
        Self { counter: 0 }
    }

    fn fresh(&mut self, base: &str) -> Name {
        let id = self.counter;
        self.counter += 1;
        let trimmed = base
            .trim_end_matches(char::is_numeric)
            .trim_end_matches('_');
        let base = if trimmed.is_empty() { "x" } else { trimmed };
        name(&format!("{base}_{id}"))
    }
}

pub fn eval0(gl: &Globals, gen: &mut Gen, env: &Env, tm: &Tm) -> Result<Val> {
    match tm {
        Tm::Var(nm) => match env.lookup(nm) {
            Some(Entry::Val(val)) => Ok(val.clone()),
            Some(Entry::CodeVar(_)) => Err(Error::Runtime(format!(
                "stage-1 variable `{nm}` used at stage 0"
            ))),
            None => gl
                .lookup(nm)
                .cloned()
                .ok_or_else(|| Error::Runtime(format!("unbound variable `{nm}`"))),
        },
        Tm::NatLit(n) => Ok(Val::Nat(*n)),
        Tm::BoolLit(b) => Ok(Val::Bool(*b)),
        Tm::Lam(nm, _, body) => Ok(Val::Lam(Rc::new(Closure {
            env: env.clone(),
            param: nm.clone(),
            body: Rc::clone(body),
        }))),
        Tm::App(fun, arg) => {
            let vf = eval0(gl, gen, env, fun)?;
            let va = eval0(gl, gen, env, arg)?;
            apply(gl, gen, vf, va)
        }
        Tm::Let(nm, _, val, body) => {
            let vv = eval0(gl, gen, env, val)?;
            let env2 = env.push_val(nm.clone(), vv);
            eval0(gl, gen, &env2, body)
        }
        Tm::Bin(op, lhs, rhs) => {
            let vl = eval0(gl, gen, env, lhs)?;
            let vr = eval0(gl, gen, env, rhs)?;
            bin_val(*op, &vl, &vr)
        }
        Tm::If(cnd, th, el) => match eval0(gl, gen, env, cnd)? {
            Val::Bool(true) => eval0(gl, gen, env, th),
            Val::Bool(false) => eval0(gl, gen, env, el),
            _ => Err(Error::Runtime("if on non-bool".into())),
        },
        Tm::Quote(e) => {
            let body = eval1(gl, gen, env, e)?;
            Ok(Val::Code(Rc::new(body)))
        }
        Tm::Splice(_) => Err(Error::Runtime("splice at stage 0".into())),
        Tm::Ann(e, _) => eval0(gl, gen, env, e),
    }
}

fn apply(gl: &Globals, gen: &mut Gen, fun: Val, arg: Val) -> Result<Val> {
    if let Val::Lam(cl) = fun {
        let env2 = cl.env.push_val(cl.param.clone(), arg);
        eval0(gl, gen, &env2, &cl.body)
    } else {
        Err(Error::Runtime("applying non-function".into()))
    }
}

fn bin_val(op: BinOp, lhs: &Val, rhs: &Val) -> Result<Val> {
    match (lhs, rhs) {
        (Val::Nat(x), Val::Nat(y)) => Ok(match op {
            BinOp::Add => Val::Nat(x + y),
            BinOp::Sub => Val::Nat(x - y),
            BinOp::Mul => Val::Nat(x * y),
            BinOp::Eq => Val::Bool(x == y),
        }),
        _ => Err(Error::Runtime("binary op on non-nat".into())),
    }
}

fn eval1(gl: &Globals, gen: &mut Gen, env: &Env, tm: &Tm) -> Result<Tm> {
    match tm {
        Tm::Var(nm) => match env.lookup(nm) {
            Some(Entry::CodeVar(fresh)) => Ok(Tm::Var(fresh.clone())),
            Some(Entry::Val(_)) => Err(Error::Runtime(format!(
                "stage-0 variable `{nm}` used at stage 1"
            ))),
            None => Err(Error::Runtime(format!("unbound stage-1 variable `{nm}`"))),
        },
        Tm::NatLit(n) => Ok(Tm::NatLit(*n)),
        Tm::BoolLit(b) => Ok(Tm::BoolLit(*b)),
        Tm::Lam(nm, ann, body) => {
            let fresh = gen.fresh(nm);
            let env2 = env.push_code(nm.clone(), fresh.clone());
            let body2 = eval1(gl, gen, &env2, body)?;
            Ok(Tm::Lam(fresh, ann.clone(), Rc::new(body2)))
        }
        Tm::App(fun, arg) => Ok(Tm::App(
            Rc::new(eval1(gl, gen, env, fun)?),
            Rc::new(eval1(gl, gen, env, arg)?),
        )),
        Tm::Let(nm, ann, val, body) => {
            let nv = eval1(gl, gen, env, val)?;
            let fresh = gen.fresh(nm);
            let env2 = env.push_code(nm.clone(), fresh.clone());
            let nb = eval1(gl, gen, &env2, body)?;
            Ok(Tm::Let(fresh, ann.clone(), Rc::new(nv), Rc::new(nb)))
        }
        Tm::Bin(op, lhs, rhs) => Ok(Tm::Bin(
            *op,
            Rc::new(eval1(gl, gen, env, lhs)?),
            Rc::new(eval1(gl, gen, env, rhs)?),
        )),
        Tm::If(cnd, th, el) => Ok(Tm::If(
            Rc::new(eval1(gl, gen, env, cnd)?),
            Rc::new(eval1(gl, gen, env, th)?),
            Rc::new(eval1(gl, gen, env, el)?),
        )),
        Tm::Quote(_) => Err(Error::Runtime("nested quote at stage 1".into())),
        Tm::Splice(e) => match eval0(gl, gen, env, e)? {
            Val::Code(t) => Ok((*t).clone()),
            _ => Err(Error::Runtime("splice of non-code".into())),
        },
        Tm::Ann(e, _) => eval1(gl, gen, env, e),
    }
}

#[must_use]
pub fn val_to_tm(val: &Val) -> Tm {
    match val {
        Val::Nat(n) => Tm::NatLit(*n),
        Val::Bool(b) => Tm::BoolLit(*b),
        Val::Code(t) => Tm::Quote(Rc::clone(t)),
        Val::Lam(cl) => Tm::Lam(cl.param.clone(), None, Rc::clone(&cl.body)),
    }
}
