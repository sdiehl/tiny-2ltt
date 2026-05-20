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
    pub fn new() -> Self {
        Self { scope: Vec::new() }
    }

    fn push(&self, n: Name, ty: Rc<Ty>, stage: Stage) -> Self {
        let mut next = self.clone();
        next.scope.push((n, Binding { ty, stage }));
        next
    }

    pub fn bind(&mut self, n: Name, ty: Rc<Ty>) {
        self.scope.push((
            n,
            Binding {
                ty,
                stage: Stage::Zero,
            },
        ));
    }

    fn lookup(&self, n: &str) -> Option<&Binding> {
        self.scope
            .iter()
            .rev()
            .find(|(k, _)| k.as_ref() == n)
            .map(|(_, b)| b)
    }
}

pub fn check_decl(env: &Env, t: &Tm, ty: &Rc<Ty>) -> Result<()> {
    check(env, Stage::Zero, t, ty)
}

pub fn infer_top(env: &Env, t: &Tm) -> Result<Rc<Ty>> {
    infer(env, Stage::Zero, t)
}

fn check(env: &Env, stage: Stage, t: &Tm, ty: &Rc<Ty>) -> Result<()> {
    match (t, ty.as_ref()) {
        (Tm::Lam(n, ann, b), Ty::Arr(a, r)) => {
            if let Some(ann_ty) = ann {
                if ann_ty.as_ref() != a.as_ref() {
                    return Err(Error::Type(format!(
                        "lambda parameter annotated {}, expected {}",
                        pretty::ty(ann_ty),
                        pretty::ty(a)
                    )));
                }
            }
            let env2 = env.push(n.clone(), Rc::clone(a), stage);
            check(&env2, stage, b, r)
        }
        (Tm::Quote(e), Ty::Code(inner)) if stage == Stage::Zero => check(env, Stage::One, e, inner),
        (Tm::If(c, th, el), _) => {
            check(env, stage, c, &Rc::new(Ty::Bool))?;
            check(env, stage, th, ty)?;
            check(env, stage, el, ty)
        }
        (Tm::Let(n, ann, v, b), _) => {
            let v_ty = match ann {
                Some(a) => {
                    check(env, stage, v, a)?;
                    Rc::clone(a)
                }
                None => infer(env, stage, v)?,
            };
            let env2 = env.push(n.clone(), v_ty, stage);
            check(&env2, stage, b, ty)
        }
        _ => {
            let inferred = infer(env, stage, t)?;
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

fn infer(env: &Env, stage: Stage, t: &Tm) -> Result<Rc<Ty>> {
    match t {
        Tm::Var(n) => match env.lookup(n) {
            Some(b) => {
                if b.stage != stage {
                    return Err(Error::Type(format!(
                        "variable `{n}` bound at stage {} but used at stage {stage}",
                        b.stage
                    )));
                }
                Ok(Rc::clone(&b.ty))
            }
            None => Err(Error::Type(format!("unbound variable `{n}`"))),
        },
        Tm::NatLit(_) => Ok(Rc::new(Ty::Nat)),
        Tm::BoolLit(_) => Ok(Rc::new(Ty::Bool)),
        Tm::Lam(n, Some(a), b) => {
            let env2 = env.push(n.clone(), Rc::clone(a), stage);
            let r = infer(&env2, stage, b)?;
            Ok(Rc::new(Ty::Arr(Rc::clone(a), r)))
        }
        Tm::Lam(_, None, _) => Err(Error::Type(
            "cannot infer lambda without parameter annotation".into(),
        )),
        Tm::App(f, a) => {
            let ft = infer(env, stage, f)?;
            match ft.as_ref() {
                Ty::Arr(dom, cod) => {
                    check(env, stage, a, dom)?;
                    Ok(Rc::clone(cod))
                }
                _ => Err(Error::Type(format!(
                    "applying non-function of type {}",
                    pretty::ty(&ft)
                ))),
            }
        }
        Tm::Let(n, ann, v, b) => {
            let v_ty = match ann {
                Some(a) => {
                    check(env, stage, v, a)?;
                    Rc::clone(a)
                }
                None => infer(env, stage, v)?,
            };
            let env2 = env.push(n.clone(), v_ty, stage);
            infer(&env2, stage, b)
        }
        Tm::Bin(op, a, b) => {
            let (in_ty, out_ty) = match op {
                BinOp::Add | BinOp::Sub | BinOp::Mul => (Ty::Nat, Ty::Nat),
                BinOp::Eq => (Ty::Nat, Ty::Bool),
            };
            let in_ty = Rc::new(in_ty);
            check(env, stage, a, &in_ty)?;
            check(env, stage, b, &in_ty)?;
            Ok(Rc::new(out_ty))
        }
        Tm::If(c, th, el) => {
            check(env, stage, c, &Rc::new(Ty::Bool))?;
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
