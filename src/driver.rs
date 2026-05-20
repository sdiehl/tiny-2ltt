use std::fmt::Write;

use crate::elab::{check_decl, infer_top, Env as TyEnv};
use crate::errors::Result;
use crate::eval::{eval0, Env, Gen, Globals, Val};
use crate::parse::parse_program;
use crate::pretty::{tm as pretty_tm, ty as pretty_ty};
use crate::syntax::Decl;

pub fn run_program(src: &str) -> Result<String> {
    let decls = parse_program(src)?;
    let mut tenv = TyEnv::new();
    let mut globals = Globals::new();
    let mut gen = Gen::new();
    let mut out = String::new();
    for d in &decls {
        match d {
            Decl::Let(n, ty, body) => {
                tenv.bind(n.clone(), ty.clone());
                check_decl(&tenv, body, ty)?;
                let v = eval0(&globals, &mut gen, &Env::new(), body)?;
                globals.bind(n, v);
                writeln!(out, "let {n} : {}", pretty_ty(ty)).unwrap();
            }
            Decl::Eval(body) => {
                let ty = infer_top(&tenv, body)?;
                let v = eval0(&globals, &mut gen, &Env::new(), body)?;
                writeln!(out, "{} : {}", show_val(&v), pretty_ty(&ty)).unwrap();
            }
        }
    }
    Ok(out)
}

fn show_val(v: &Val) -> String {
    match v {
        Val::Nat(n) => n.to_string(),
        Val::Bool(b) => b.to_string(),
        Val::Code(t) => format!("<{}>", pretty_tm(t)),
        Val::Lam(cl) => format!("\\{}. {}", cl.param, pretty_tm(&cl.body)),
    }
}
