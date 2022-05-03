// TODO(crz)
use crate::syntax::{Pretype, Qualifier, Term, TermCtx, Type};
use std::collections::HashMap;

type TypeCtx = HashMap<String, Type>;

fn type_ctx_eq(a: &TypeCtx, b: &TypeCtx) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for (k, v) in a {
        match b.get(k) {
            Some(v1) if v1 == v => {}
            _ => return false,
        }
    }
    true
}

fn type_check_aux(
    term_ctx: &TermCtx,
    type_ctx: &mut TypeCtx,
    type_map: &mut HashMap<(usize, usize), Type>,
) -> Option<Type> {
    let TermCtx(span, term) = term_ctx;
    let type_: Type = match term {
        Term::Variable(id) => {
            let ty = type_ctx.get(id)?;
            let Type(q, _) = ty;
            let ty = ty.clone();
            if *q == Qualifier::Linear {
                type_ctx.remove(id);
            }
            ty
        }
        Term::Boolean(q, _) => Type(q.clone(), Pretype::Boolean),
        Term::Integer(q, _) => Type(q.clone(), Pretype::Integer),
        Term::Conditional(cond, then, alter) => {
            let Type(_, cond_type) = type_check_aux(cond, type_ctx, type_map)?;
            let mut type_ctx1 = type_ctx.clone();
            let type_ctx1 = &mut type_ctx1;
            let then_type = type_check_aux(then, type_ctx, type_map)?;
            let alter_type = type_check_aux(alter, type_ctx1, type_map)?;
            if !type_ctx_eq(type_ctx, type_ctx1) {
                return None;
            }
            if cond_type != Pretype::Boolean || then_type != alter_type {
                return None;
            }
            then_type
        }
        Term::Abstraction(q, x, Some(ty), body) => {
            let type_ctx0 = type_ctx.clone();
            type_ctx.insert(x.clone(), ty.as_ref().clone());
            let body_type = type_check_aux(body, type_ctx, type_map)?;
            // output typing context should not contain introduced linear type
            if ty.0 == Qualifier::Linear && type_ctx.contains_key(x) {
                return None;
            }
            type_ctx.remove(x);
            // if the closure is unrestricted,
            // there should be no reference to linear variable in body
            if *q == Qualifier::Nop && !type_ctx_eq(type_ctx, &type_ctx0) {
                return None;
            }
            Type(
                q.clone(),
                Pretype::Function(ty.clone(), Box::new(body_type)),
            )
        }
        Term::Application(fun, arg) => {
            let fun_type = type_check_aux(fun, type_ctx, type_map)?;
            let arg_type = type_check_aux(arg, type_ctx, type_map)?;
            match fun_type {
                Type(_, Pretype::Function(ty1, ty2)) if *ty1 == arg_type => *ty2,
                _ => return None,
            }
        }
        _ => return None,
    };
    type_map.insert((span.start, span.end), type_.clone());
    Some(type_)
}

pub fn type_check(term_ctx: &TermCtx) -> Option<HashMap<(usize, usize), Type>> {
    let mut type_map = HashMap::<(usize, usize), Type>::new();
    let mut type_ctx = HashMap::<String, Type>::new();
    type_check_aux(term_ctx, &mut type_ctx, &mut type_map)?;
    Some(type_map)
}
