use std::fmt::Write;

use crate::elab;
use crate::errors::Result;
use crate::eval::{self, Env, Gen, Globals, Val};
use crate::parse;
use crate::pretty;
use crate::syntax::Decl;

pub fn run_program(src: &str) -> Result<String> {
    let decls = parse::parse_program(src)?;
    let mut tenv = elab::Env::new();
    let mut globals = Globals::new();
    let mut gen = Gen::new();
    let mut out = String::new();
    for d in &decls {
        match d {
            Decl::Let(n, ty, body) => {
                tenv.bind(n.clone(), ty.clone());
                elab::check_decl(&tenv, body, ty)?;
                let v = eval::eval0(&globals, &mut gen, &Env::new(), body)?;
                globals.bind(n.clone(), v);
                writeln!(out, "let {n} : {}", pretty::ty(ty)).unwrap();
            }
            Decl::Eval(body) => {
                let ty = elab::infer_top(&tenv, body)?;
                let v = eval::eval0(&globals, &mut gen, &Env::new(), body)?;
                writeln!(out, "{} : {}", show_val(&v), pretty::ty(&ty)).unwrap();
            }
        }
    }
    Ok(out)
}

fn show_val(v: &Val) -> String {
    match v {
        Val::Nat(n) => n.to_string(),
        Val::Bool(b) => b.to_string(),
        Val::Code(t) => format!("<{}>", pretty::tm(t)),
        Val::Lam(cl) => format!("\\{}. {}", cl.param, pretty::tm(&cl.body)),
    }
}
