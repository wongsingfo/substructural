use crate::error::Error;
use crate::formatter::{self, TermFormatter};
use crate::syntax::{ArithOp, Context, Pretype, Qualifier, Term, TermCtx, Type};
use serde::{Deserialize, Serialize};
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
    type_map: &mut HashMap<Context, Type>,
) -> Result<Type, Error> {
    let TermCtx(span, term) = term_ctx;
    let err = |s: String| Error::TypeError {
        start: span.start,
        end: span.end,
        message: s,
    };
    let need_type_tip = match term {
        Term::Application(..) => false,
        _ => true,
    };
    let type_: Type = match term {
        Term::Variable(id) => {
            let ty = type_ctx
                .get(id)
                .ok_or_else(|| err(format!("undefined variable: {}", id)))?;
            let Type(q, _) = ty;
            let ty = ty.clone();
            if *q == Qualifier::Linear {
                type_ctx.remove(id);
            }
            ty
        }
        Term::Boolean(q, _) => Type(*q, Pretype::Boolean),
        Term::Integer(q, _) => Type(*q, Pretype::Integer),
        Term::Conditional(cond, then, alter) => {
            let Type(_, cond_type) = type_check_aux(cond, type_ctx, type_map)?;
            let mut type_ctx1 = type_ctx.clone();
            let type_ctx1 = &mut type_ctx1;
            let then_type = type_check_aux(then, type_ctx, type_map)?;
            let alter_type = type_check_aux(alter, type_ctx1, type_map)?;
            if !type_ctx_eq(type_ctx, type_ctx1) {
                return Err(err(
                    "variables are consumed differently in different branches".to_string(),
                ));
            }
            if cond_type != Pretype::Boolean {
                return Err(err(format!("expect Boolean, given {:?}", cond_type)));
            }
            if then_type != alter_type {
                return Err(err(format!(
                    "different branch types: {:?} vs {:?}",
                    then_type, alter_type
                )));
            }
            then_type
        }
        Term::Abstraction(q, x, Some(ty), body) => {
            let type_ctx0 = type_ctx.clone();
            type_ctx.insert(x.clone(), ty.as_ref().clone());
            let body_type = type_check_aux(body, type_ctx, type_map)?;
            // output typing context should not contain introduced linear type
            if ty.0 == Qualifier::Linear && type_ctx.contains_key(x) {
                return Err(err(format!(
                    "linear variable {} is not consumed in function body",
                    x
                )));
            }
            type_ctx.remove(x);
            // if the closure is unrestricted,
            // there should be no reference to linear variable in body
            if *q == Qualifier::Nop && !type_ctx_eq(type_ctx, &type_ctx0) {
                return Err(err(
                    "free linear variable is refered in unrestricted function body".to_string(),
                ));
            }
            Type(*q, Pretype::Function(ty.clone(), Box::new(body_type)))
        }
        Term::Application(fun, arg) => {
            let fun_type = type_check_aux(fun, type_ctx, type_map)?;
            let arg_type = type_check_aux(arg, type_ctx, type_map)?;
            match fun_type {
                Type(_, Pretype::Function(ty1, ty2)) => {
                    if *ty1 != arg_type {
                        return Err(err(format!(
                            "expect {:?} as argument, given {:?}",
                            ty1, arg_type
                        )));
                    }
                    *ty2
                }
                _ => return Err(err(format!("expect Function, given {:?}", fun_type))),
            }
        }
        Term::Let(x, t1, t2) => {
            let t1_type = type_check_aux(t1, type_ctx, type_map)?;
            let q = t1_type.0;
            type_ctx.insert(x.clone(), t1_type);
            let t2_type = type_check_aux(t2, type_ctx, type_map)?;
            if q == Qualifier::Linear && type_ctx.contains_key(x) {
                return Err(err(format!(
                    "linear variable {} is not consumed in let body",
                    x
                )));
            }
            type_ctx.remove(x);
            t2_type
        }
        Term::Fix(t) => {
            let t_type = type_check_aux(t, type_ctx, type_map)?;
            match t_type {
                Type(q, Pretype::Function(ty1, ty2)) if *ty1 == *ty2 => {
                    let Type(q1, _) = *ty1;
                    if q == Qualifier::Linear || q1 == Qualifier::Linear {
                        return Err(err(format!("linear term is not allowed for recursion")));
                    }
                    *ty1
                }
                _ => {
                    return Err(err(format!(
                        "expect Function T -> T, given {:?}",
                        t_type
                    )))
                }
            }
        }
        Term::Compound(q, t1, t2) => {
            let type_ctx0 = type_ctx.clone();
            let t1_type = type_check_aux(t1, type_ctx, type_map)?;
            if *q == Qualifier::Nop && !type_ctx_eq(type_ctx, &type_ctx0) {
                return Err(err(
                    "free linear variable is refered in unrestricted compound pair".to_string(),
                ));
            }
            let type_ctx0 = type_ctx.clone();
            let t2_type = type_check_aux(t2, type_ctx, type_map)?;
            if *q == Qualifier::Nop && !type_ctx_eq(type_ctx, &type_ctx0) {
                return Err(err(
                    "free linear variable is refered in unrestricted compound pair".to_string(),
                ));
            }
            Type(*q, Pretype::Compound(Box::new(t1_type), Box::new(t2_type)))
        }
        Term::Letc(x1, x2, t1, t2) => {
            if x1 == x2 {
                return Err(err(format!("expect different identifier, given {}", x1)));
            }
            let t1_type = type_check_aux(t1, type_ctx, type_map)?;
            match t1_type {
                Type(_, Pretype::Compound(ty1, ty2)) => {
                    let (q1, q2) = (ty1.0, ty2.0);
                    type_ctx.insert(x1.clone(), *ty1);
                    type_ctx.insert(x2.clone(), *ty2);
                    let t2_type = type_check_aux(t2, type_ctx, type_map)?;
                    for (q, x) in [(q1, x1), (q2, x2)] {
                        if q == Qualifier::Linear && type_ctx.contains_key(x) {
                            return Err(err(format!(
                                "linear variable {} is not consumed in let body",
                                x
                            )));
                        }
                    }
                    type_ctx.remove(x1);
                    type_ctx.remove(x2);
                    t2_type
                }
                _ => return Err(err(format!("expect Compound, given {:?}", t1_type))),
            }
        }
        Term::Arith1(q, ArithOp::IsZero, t) => match type_check_aux(t, type_ctx, type_map)? {
            Type(_, Pretype::Integer) => Type(*q, Pretype::Boolean),
            ty => return Err(err(format!("expect Integer, given {:?}", ty))),
        },
        Term::Arith2(q, ArithOp::Diff, t1, t2) => {
            let t1_type = type_check_aux(t1, type_ctx, type_map)?;
            let t2_type = type_check_aux(t2, type_ctx, type_map)?;
            match (t1_type, t2_type) {
                (Type(_, Pretype::Integer), Type(_, Pretype::Integer)) => {
                    Type(*q, Pretype::Integer)
                }
                (ty1, ty2) => {
                    return Err(err(format!(
                        "expect Integers, given {:?} and {:?}",
                        ty1, ty2
                    )))
                }
            }
        }
        _ => return Err(err(format!("unknown term {:?}", term))),
    };
    if need_type_tip {
        type_map.insert(*span, type_.clone());
    }
    Ok(type_)
}

pub fn type_check(term_ctx: &TermCtx) -> Result<HashMap<Context, Type>, Error> {
    let mut type_map = HashMap::<Context, Type>::new();
    let mut type_ctx = HashMap::<String, Type>::new();
    type_check_aux(term_ctx, &mut type_ctx, &mut type_map)?;
    Ok(type_map)
}

#[derive(Eq, PartialEq)]
struct HeapState(usize, Type);

impl PartialOrd for HeapState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HeapState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TypedTermStr<'a> {
    ty: Option<String>,
    s: &'a str,
}

pub fn convert_hashmap_to_vec<'a>(
    type_map: &HashMap<Context, Type>,
    source: &'a str,
) -> Vec<TypedTermStr<'a>> {
    // (position, is_start, type)
    let mut event: Vec<(usize, bool, &Type)> = Vec::new();
    let mut max_pos = 0;
    for (span, ty) in type_map.iter() {
        event.push((span.start, true, ty));
        // the `end` is not inclusive
        event.push((span.end, false, ty));
        max_pos = max_pos.max(span.end);
    }
    event.sort_by_key(|&(pos, t, _)| (pos, t));
    let mut formatter = TermFormatter::new(formatter::DEFAULT_LINE_WIDTH);
    let mut tags: Vec<TypedTermStr<'a>> = Vec::new();
    let mut stack: Vec<&Type> = Vec::new();
    let mut event_i = event.iter().peekable();

    let mut start = 0;
    for (i, c) in source.char_indices() {
        let mut is_changed = false;
        let mut is_changed_type: Option<Type> = None;
        while let Some((pos, t, ty)) = event_i.peek() {
            if *pos == i {
                if !is_changed {
                    is_changed_type = stack.last().map(|&t| t.to_owned());
                }
                is_changed = true;
                if *t {
                    stack.push(ty);
                } else {
                    stack.pop();
                }
                event_i.next();
            } else {
                break;
            }
        }
        if is_changed && start < i {
            tags.push(TypedTermStr {
                ty: is_changed_type.map(|t| formatter.format_type(&t)),
                s: &source[start..i],
            });
            if c == '\n' {
                tags.push(TypedTermStr {
                    ty: None,
                    s: &source[i..i + 1],
                });
                start = i + 1;
            } else {
                start = i;
            }
        } else if c == '\n' {
            if start < i {
                tags.push(TypedTermStr {
                    ty: stack.last().map(|t| formatter.format_type(&t)),
                    s: &source[start..i],
                });
            }
            tags.push(TypedTermStr {
                ty: None,
                s: &source[i..i + 1],
            });
            start = i + 1;
        }
    }
    if start < source.len() {
        tags.push(TypedTermStr {
            ty: stack
                .last()
                .map(|&t| t.to_owned())
                .map(|t| formatter.format_type(&t)),
            s: &source[start..],
        });
    }

    tags
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::parse_program;

    #[test]
    fn test_type_atom() {
        let input = "$true";
        let term = parse_program(input).unwrap();
        let type_map = type_check(&term).unwrap();
        println!("{:?}", type_map);
        assert_eq!(type_map.len(), 1);
    }

    #[test]
    fn test_type_if() {
        let input = "if $true { 1 } else { 2 }";
        let term = parse_program(input).unwrap();
        let type_map = type_check(&term).unwrap();
        println!("{:#?}", type_map);
    }

    #[test]
    fn test_type_prog01() {
        let input = "(|x: bool| x) (false)";
        let term = parse_program(input).unwrap();
        let type_map = type_check(&term).unwrap();
        println!("{:?}", type_map);
    }

    #[test]
    fn test_hashmap_to_vec() {
        let input = "(|x: $bool| if x { false } else { true }) ($false)";
        let term = parse_program(input).unwrap();
        let type_map = type_check(&term).unwrap();
        let vec = convert_hashmap_to_vec(&type_map, &input);
        println!("{:#?}", type_map);
        println!("{:#?}", vec);
    }
}
