use crate::error::Error;
use crate::syntax::{ArithOp, Qualifier, Term, TermCtx};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    bindings: HashMap<String, TermCtx>,
    counter: u128,
}

impl Store {
    pub fn new_empty() -> Store {
        Store {
            bindings: HashMap::new(),
            counter: 0,
        }
    }

    fn push(&mut self, name: String, value: TermCtx) {
        assert!(get_qualifier(&value).is_some());
        self.bindings.insert(name, value);
    }

    fn extract(&mut self, name: &str) -> Option<TermCtx> {
        let value = self.bindings.get(name)?;
        return match get_qualifier(value) {
            Some(Qualifier::Linear) => self.bindings.remove(name),
            _ => self.bindings.get(name).map(|x| x.clone()),
        };
    }

    fn fresh_variable(&mut self, prefix: &str) -> String {
        self.counter += 1;
        format!("{}{}", prefix, self.counter)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TermEval {
    store: Store,
    term: TermCtx,
}

impl From<TermCtx> for TermEval {
    fn from(term: TermCtx) -> TermEval {
        TermEval {
            store: Store::new_empty(),
            term,
        }
    }
}

fn is_var(term: &TermCtx) -> bool {
    match term {
        TermCtx(_, Term::Variable(..)) => true,
        _ => false,
    }
}

fn is_value(term: &TermCtx) -> bool {
    let TermCtx(_, term) = term;
    match term {
        Term::Boolean(..) => true,
        Term::Integer(..) => true,
        Term::Abstraction(..) => true,
        Term::Compound(_, ref t1, ref t2) if is_var(t1) && is_var(t2) => true,
        _ => false,
    }
}

fn get_qualifier(term: &TermCtx) -> Option<Qualifier> {
    let TermCtx(_, term) = term;
    let q = match term {
        Term::Boolean(q, ..) => q,
        Term::Integer(q, ..) => q,
        Term::Abstraction(q, ..) => q,
        Term::Compound(q, ..) => q,
        _ => return None,
    };
    Some(*q)
}

fn subst_var(term_ctx: Box<TermCtx>, x: &str, x2: &str) -> Box<TermCtx> {
    let TermCtx(ctx, term) = *term_ctx;
    let term = match term {
        Term::Variable(y) if y == x => Term::Variable(x2.to_owned()),
        Term::Application(t1, t2) => Term::Application(subst_var(t1, x, x2), subst_var(t2, x, x2)),
        Term::Abstraction(q, y, ty, t) => Term::Abstraction(
            q,
            y.clone(),
            ty,
            if y == x { t } else { subst_var(t, x, x2) },
        ),
        Term::Conditional(t1, t2, t3) => Term::Conditional(
            subst_var(t1, x, x2),
            subst_var(t2, x, x2),
            subst_var(t3, x, x2),
        ),
        Term::Let(y, t1, t2) => Term::Let(
            y.clone(),
            subst_var(t1, x, x2),
            if y == x { t2 } else { subst_var(t2, x, x2) },
        ),
        Term::Fix(t) => Term::Fix(subst_var(t, x, x2)),
        Term::Letc(y1, y2, t1, t2) => Term::Letc(
            y1.clone(),
            y2.clone(),
            subst_var(t1, x, x2),
            if y1 == x || y2 == x {
                t2
            } else {
                subst_var(t2, x, x2)
            },
        ),
        Term::Compound(q, t1, t2) => Term::Compound(q, subst_var(t1, x, x2), subst_var(t2, x, x2)),
        Term::Arith1(q, op, t) => Term::Arith1(q, op, subst_var(t, x, x2)),
        Term::Arith2(q, op, t1, t2) => {
            Term::Arith2(q, op, subst_var(t1, x, x2), subst_var(t2, x, x2))
        }
        _ => term,
    };
    Box::new(TermCtx(ctx, term))
}

fn one_step_eval_aux(store: &mut Store, term_ctx: TermCtx) -> Result<TermCtx, Error> {
    let err = |msg| Error::EvaluateError {
        message: msg,
        source: term_ctx.0.to_string(),
    };
    let mut extract = |x: &str| -> Result<TermCtx, Error> {
        store
            .extract(x)
            .ok_or_else(|| err(format!("Variable {} not found", x)))
    };
    let TermCtx(ctx, term) = term_ctx;
    let dup_term = term.clone();
    let term = match term {
        Term::Variable(x) => extract(&x)?.1,
        Term::Boolean(..) | Term::Integer(..) | Term::Abstraction(..) => {
            let var = store.fresh_variable("%x");
            store.push(var.clone(), TermCtx(ctx, term));
            Term::Variable(var)
            // term
        }
        Term::Compound(q, t1, t2) => match (&*t1, &*t2) {
            (TermCtx(_, Term::Variable(..)), TermCtx(_, Term::Variable(..))) => {
                let var = store.fresh_variable("%x");
                store.push(var.clone(), TermCtx(ctx, dup_term));
                Term::Variable(var)
            }
            (TermCtx(_, Term::Variable(..)), _) => {
                Term::Compound(q, t1, Box::new(one_step_eval_aux(store, *t2)?))
            }
            _ => Term::Compound(q, Box::new(one_step_eval_aux(store, *t1)?), t2),
        },
        Term::Conditional(t1, t2, t3) => match *t1 {
            TermCtx(_, Term::Variable(x)) => match extract(&x)? {
                TermCtx(_, Term::Boolean(_, v)) => return Ok(if v { *t2 } else { *t3 }),
                t1_ @ TermCtx(_, Term::Fix(..)) => Term::Conditional(Box::new(t1_), t2, t3),
                _ => return Err(err(format!("Conditional term must be boolean"))),
            },
            _ => {
                let t1 = one_step_eval_aux(store, *t1)?;
                Term::Conditional(Box::new(t1), t2, t3)
            }
        },
        Term::Application(t1, t2) => match (&*t1, &*t2) {
            (TermCtx(_, Term::Variable(x1)), TermCtx(_, Term::Variable(x2))) => match extract(&x1)?
            {
                TermCtx(_, Term::Abstraction(_, x, _, body)) => {
                    return Ok(*subst_var(body, &x, &x2))
                }
                t1_ @ TermCtx(_, Term::Fix(..)) => Term::Application(Box::new(t1_), t2),
                _ => return Err(err(format!("Expect abstraction"))),
            },
            (TermCtx(_, Term::Variable(_)), _) => {
                Term::Application(t1, Box::new(one_step_eval_aux(store, *t2)?))
            }
            _ => Term::Application(Box::new(one_step_eval_aux(store, *t1)?), t2),
        },
        Term::Let(x, t1, t2) => match *t1 {
            TermCtx(_, Term::Variable(y)) => return Ok(*subst_var(t2, &x, &y)),
            _ => Term::Let(x, Box::new(one_step_eval_aux(store, *t1)?), t2),
        },
        Term::Letc(x1, x2, term, body) => match &*term {
            TermCtx(_, Term::Variable(x)) => match extract(&x)? {
                TermCtx(_, Term::Compound(_, y1, y2)) => match (&*y1, &*y2) {
                    (TermCtx(_, Term::Variable(y1)), TermCtx(_, Term::Variable(y2))) => {
                        return Ok(*subst_var(subst_var(body, &x1, &y1), &x2, &y2))
                    }
                    _ => return Err(err(format!("..."))),
                },
                _ => return Err(err(format!("Expect compound"))),
            },
            _ => Term::Letc(x1, x2, Box::new(one_step_eval_aux(store, *term)?), body),
        },
        Term::Fix(t) => match *t {
            TermCtx(ctx1, Term::Abstraction(q, f, ty, body)) => match store.extract(&f) {
                Some(_) => return Ok(*body),
                None => {
                    let new_f = store.fresh_variable("%f");
                    let body_var = store.fresh_variable("%f");
                    let body_ctx = body.0;
                    let new_body = subst_var(body, &f, &new_f);
                    store.bindings.insert(body_var.clone(), *new_body);
                    let new_body = TermCtx(body_ctx, Term::Variable(body_var));
                    let fix_term = TermCtx(
                        ctx,
                        Term::Fix(Box::new(TermCtx(
                            ctx1,
                            Term::Abstraction(q, new_f.clone(), ty, Box::new(new_body.clone())),
                        ))),
                    );
                    store.bindings.insert(new_f, fix_term);
                    return Ok(new_body);
                }
            },
            _ => Term::Fix(Box::new(one_step_eval_aux(store, *t)?)),
        },
        Term::Arith1(q, op, t) => match &*t {
            TermCtx(_, Term::Variable(x)) => match extract(&x)? {
                TermCtx(_, Term::Integer(_, v1)) => match op {
                    ArithOp::IsZero => Term::Boolean(q, v1 == 0),
                    _ => return Err(err(format!("Unknown op {:?}", op))),
                },
                _ => return Err(err("Expect an Integer".to_string())),
            },
            _ => Term::Arith1(q, op, Box::new(one_step_eval_aux(store, *t)?)),
        },
        Term::Arith2(q, op, t1, t2) => match (&*t1, &*t2) {
            (TermCtx(_, Term::Variable(x1)), TermCtx(_, Term::Variable(x2))) => {
                match (extract(&x1)?, extract(&x2)?) {
                    (TermCtx(_, Term::Integer(_, v1)), TermCtx(_, Term::Integer(_, v2))) => {
                        match op {
                            ArithOp::Diff => Term::Integer(q, v1 - v2),
                            _ => return Err(err(format!("Unknown op {:?}", op))),
                        }
                    }
                    _ => return Err(err(format!("Expect Integers"))),
                }
            }
            (TermCtx(_, Term::Variable(..)), _) => {
                Term::Arith2(q, op, t1, Box::new(one_step_eval_aux(store, *t2)?))
            }
            _ => Term::Arith2(q, op, Box::new(one_step_eval_aux(store, *t1)?), t2),
        },
    };
    Ok(TermCtx(ctx, term))
}

pub(crate) fn one_step_eval(term_eval: TermEval) -> Result<TermEval, Error> {
    let TermEval { mut store, term } = term_eval;
    let term = if is_value(&term) {
        term
    } else {
        one_step_eval_aux(&mut store, term)?
    };
    Ok(TermEval { store, term })
}

#[cfg(test)]
mod test {
    use crate::formatter::{self, TermFormatter};
    use crate::syntax::parse_program;

    use super::*;

    #[test]
    fn test_eval_simple() {
        let store = Store::new_empty();
        let input = "$5";
        let term = parse_program(input).unwrap();
        let result = one_step_eval(TermEval { store, term }).unwrap();
        println!("{:#?}", result);
    }

    #[test]
    fn test_eval_if() {
        let store = Store::new_empty();
        let input = "if true {1} else {2}";
        let term = parse_program(input).unwrap();
        let result = one_step_eval(TermEval { store, term }).unwrap();
        println!("{:#?}", result);
    }

    #[test]
    fn test_eval_application() {
        let store = Store::new_empty();
        let input = "(|x| x) ($true)";
        let term = parse_program(input).unwrap();
        let result = TermEval { store, term };
        let result = one_step_eval(result).unwrap();
        let result = one_step_eval(result).unwrap();
        let result = one_step_eval(result).unwrap();
        let result = one_step_eval(result).unwrap();
        println!("{:#?}", result);
    }

    #[test]
    fn test_eval_closure() {
        let store = Store::new_empty();
        let mut formatter = TermFormatter::new(formatter::DEFAULT_LINE_WIDTH);
        let input = "(|x| |y| x) (true) (false)";
        let term = parse_program(input).unwrap();
        let mut result = TermEval { store, term };
        for _ in 0..20 {
            result = one_step_eval(result).unwrap();
            println!(
                "{:?} | {}",
                result.store,
                formatter.format_termctx(&result.term)
            );
        }
    }

    #[test]
    fn test_eval_let_fix() {
        let store = Store::new_empty();
        let mut formatter = TermFormatter::new(formatter::DEFAULT_LINE_WIDTH);
        let input = "let f = fix(|ff||x| if x {ff(false)} else {ff(true)}) in f(true)";
        let term = parse_program(input).unwrap();
        let mut result = TermEval { store, term };
        for _ in 0..20 {
            result = one_step_eval(result).unwrap();
            println!(
                "{:?} \n {}\n",
                result.store,
                formatter.format_termctx(&result.term)
            );
        }
    }

    #[test]
    fn test_eval_min_file_handle() {
        let store = Store::new_empty();
        let mut formatter = TermFormatter::new(formatter::DEFAULT_LINE_WIDTH);
        let input = "
        let open = |x| $true in
        let read = |h| h in 
        let close = |h| (if h {1} else {0}) in
        let h = open(0) in 
        let h = read(h) in 
        let h = read(h) in 
        close(h)
        ";
        let term = parse_program(input).unwrap();
        let mut result = TermEval { store, term };
        for _ in 0..20 {
            result = one_step_eval(result).unwrap();
            println!(
                "{:?} \n {}\n",
                result.store,
                formatter.format_termctx(&result.term)
            );
        }
    }

    #[test]
    fn test_eval_diff() {
        let store = Store::new_empty();
        let mut formatter = TermFormatter::new(formatter::DEFAULT_LINE_WIDTH);
        let input = "let negate = |x| $diff($0, x) in negate($5)";
        let term = parse_program(input).unwrap();
        let mut result = TermEval { store, term };
        for _ in 0..20 {
            result = one_step_eval(result).unwrap();
            println!(
                "{:?} \n {}\n",
                result.store,
                formatter.format_termctx(&result.term)
            );
        }
    }
}
