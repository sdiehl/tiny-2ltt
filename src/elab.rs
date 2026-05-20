use std::rc::Rc;

use crate::errors::{Error, Result};
use crate::pretty;
use crate::syntax::{BinOp, Name, Stage, Tm, Ty};

#[derive(Debug, Clone)]
struct Binding {
    ty: Rc<Ty>,
    stage: Stage,
}

#[derive(Debug, Clone, Default)]
pub struct Env {
    scope: Vec<(Name, Binding)>,
}

impl Env {
    #[must_use]
    pub const fn new() -> Self {
        Self { scope: Vec::new() }
    }

    fn push(&self, nm: Name, ty: Rc<Ty>, stage: Stage) -> Self {
        let mut next = self.clone();
        next.scope.push((nm, Binding { ty, stage }));
        next
    }

    pub fn bind(&mut self, nm: Name, ty: Rc<Ty>) {
        self.scope.push((
            nm,
            Binding {
                ty,
                stage: Stage::Zero,
            },
        ));
    }

    fn lookup(&self, nm: &str) -> Option<&Binding> {
        self.scope
            .iter()
            .rev()
            .find(|(k, _)| k.as_ref() == nm)
            .map(|(_, b)| b)
    }
}

pub fn check_decl(env: &Env, tm: &Tm, ty: &Rc<Ty>) -> Result<()> {
    check(env, Stage::Zero, tm, ty)
}

pub fn infer_top(env: &Env, tm: &Tm) -> Result<Rc<Ty>> {
    infer(env, Stage::Zero, tm)
}

fn check(env: &Env, stage: Stage, tm: &Tm, ty: &Rc<Ty>) -> Result<()> {
    match (tm, ty.as_ref()) {
        (Tm::Lam(nm, ann, body), Ty::Arr(dom, cod)) => {
            if let Some(ann_ty) = ann {
                if ann_ty.as_ref() != dom.as_ref() {
                    return Err(Error::Type(format!(
                        "lambda parameter annotated {}, expected {}",
                        pretty::ty(ann_ty),
                        pretty::ty(dom)
                    )));
                }
            }
            let env2 = env.push(nm.clone(), Rc::clone(dom), stage);
            check(&env2, stage, body, cod)
        }
        (Tm::Quote(e), Ty::Code(inner)) if stage == Stage::Zero => check(env, Stage::One, e, inner),
        (Tm::If(cnd, th, el), _) => {
            check(env, stage, cnd, &Rc::new(Ty::Bool))?;
            check(env, stage, th, ty)?;
            check(env, stage, el, ty)
        }
        (Tm::Let(nm, ann, val, body), _) => {
            let v_ty = if let Some(a) = ann {
                check(env, stage, val, a)?;
                Rc::clone(a)
            } else {
                infer(env, stage, val)?
            };
            let env2 = env.push(nm.clone(), v_ty, stage);
            check(&env2, stage, body, ty)
        }
        _ => {
            let inferred = infer(env, stage, tm)?;
            if inferred.as_ref() != ty.as_ref() {
                return Err(Error::Type(format!(
                    "expected {}, got {}",
                    pretty::ty(ty),
                    pretty::ty(&inferred)
                )));
            }
            Ok(())
        }
    }
}

fn infer(env: &Env, stage: Stage, tm: &Tm) -> Result<Rc<Ty>> {
    match tm {
        Tm::Var(nm) => env.lookup(nm).map_or_else(
            || Err(Error::Type(format!("unbound variable `{nm}`"))),
            |b| {
                if b.stage == stage {
                    Ok(Rc::clone(&b.ty))
                } else {
                    Err(Error::Type(format!(
                        "variable `{nm}` bound at stage {} but used at stage {stage}",
                        b.stage
                    )))
                }
            },
        ),
        Tm::NatLit(_) => Ok(Rc::new(Ty::Nat)),
        Tm::BoolLit(_) => Ok(Rc::new(Ty::Bool)),
        Tm::Lam(nm, Some(dom), body) => {
            let env2 = env.push(nm.clone(), Rc::clone(dom), stage);
            let cod = infer(&env2, stage, body)?;
            Ok(Rc::new(Ty::Arr(Rc::clone(dom), cod)))
        }
        Tm::Lam(_, None, _) => Err(Error::Type(
            "cannot infer lambda without parameter annotation".into(),
        )),
        Tm::App(fun, arg) => {
            let ft = infer(env, stage, fun)?;
            match ft.as_ref() {
                Ty::Arr(dom, cod) => {
                    check(env, stage, arg, dom)?;
                    Ok(Rc::clone(cod))
                }
                _ => Err(Error::Type(format!(
                    "applying non-function of type {}",
                    pretty::ty(&ft)
                ))),
            }
        }
        Tm::Let(nm, ann, val, body) => {
            let v_ty = if let Some(a) = ann {
                check(env, stage, val, a)?;
                Rc::clone(a)
            } else {
                infer(env, stage, val)?
            };
            let env2 = env.push(nm.clone(), v_ty, stage);
            infer(&env2, stage, body)
        }
        Tm::Bin(op, lhs, rhs) => {
            let (in_ty, out_ty) = match op {
                BinOp::Add | BinOp::Sub | BinOp::Mul => (Ty::Nat, Ty::Nat),
                BinOp::Eq => (Ty::Nat, Ty::Bool),
            };
            let in_ty = Rc::new(in_ty);
            check(env, stage, lhs, &in_ty)?;
            check(env, stage, rhs, &in_ty)?;
            Ok(Rc::new(out_ty))
        }
        Tm::If(cnd, th, el) => {
            check(env, stage, cnd, &Rc::new(Ty::Bool))?;
            let tt = infer(env, stage, th)?;
            check(env, stage, el, &tt)?;
            Ok(tt)
        }
        Tm::Quote(e) => {
            if stage != Stage::Zero {
                return Err(Error::Type("quote `<..>` is only valid at stage 0".into()));
            }
            let inner = infer(env, Stage::One, e)?;
            Ok(Rc::new(Ty::Code(inner)))
        }
        Tm::Splice(e) => {
            if stage != Stage::One {
                return Err(Error::Type("splice `~e` is only valid at stage 1".into()));
            }
            let code = infer(env, Stage::Zero, e)?;
            match code.as_ref() {
                Ty::Code(inner) => Ok(Rc::clone(inner)),
                _ => Err(Error::Type(format!(
                    "splice of non-code type {}",
                    pretty::ty(&code)
                ))),
            }
        }
        Tm::Ann(e, ty) => {
            check(env, stage, e, ty)?;
            Ok(Rc::clone(ty))
        }
    }
}
