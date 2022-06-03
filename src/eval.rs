use crate::error::Error;
use crate::syntax::{Qualifier, Term, TermCtx};
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

fn is_value(term: &TermCtx) -> bool {
    let TermCtx(_, term) = term;
    match term {
        Term::Boolean(_, _) => true,
        Term::Integer(_, _) => true,
        Term::Abstraction(_, _, _, _) => true,
        _ => false,
    }
}

fn get_qualifier(term: &TermCtx) -> Option<Qualifier> {
    let TermCtx(_, term) = term;
    let q = match term {
        Term::Boolean(q, _) => q,
        Term::Integer(q, _) => q,
        Term::Abstraction(q, _, _, _) => q,
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
    let term = match term {
        Term::Variable(x) => extract(&x)?.1,
        Term::Boolean(_, _) | Term::Integer(_, _) | Term::Abstraction(_, _, _, _) => {
            let var = store.fresh_variable("%x");
            store.push(var.clone(), TermCtx(ctx, term));
            Term::Variable(var)
            // term
        }
        Term::Conditional(t1, t2, t3) => match *t1 {
            TermCtx(_, Term::Variable(x)) => match extract(&x)? {
                TermCtx(_, Term::Boolean(_, v)) => return Ok(if v { *t2 } else { *t3 }),
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
                _ => return Err(err(format!("Expect abstraction"))),
            },
            (TermCtx(_, Term::Variable(_)), _) => {
                Term::Application(t1, Box::new(one_step_eval_aux(store, *t2)?))
            }
            _ => Term::Application(Box::new(one_step_eval_aux(store, *t1)?), t2),
        },
        Term::Fix(..) => unimplemented!(),
        Term::Let(..) => unimplemented!(),
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
    use crate::formatter::{TermFormatter, self};
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
        let result = TermEval { store, term };
        let result = one_step_eval(result).unwrap();
        println!(
            "{:?} | {}",
            result.store,
            formatter.format_termctx(&result.term)
        );
        let result = one_step_eval(result).unwrap();
        println!(
            "{:?} | {}",
            result.store,
            formatter.format_termctx(&result.term)
        );
        let result = one_step_eval(result).unwrap();
        println!(
            "{:?} | {}",
            result.store,
            formatter.format_termctx(&result.term)
        );
        let result = one_step_eval(result).unwrap();
        println!(
            "{:?} | {}",
            result.store,
            formatter.format_termctx(&result.term)
        );
        let result = one_step_eval(result).unwrap();
        println!(
            "{:?} | {}",
            result.store,
            formatter.format_termctx(&result.term)
        );
        let result = one_step_eval(result).unwrap();
        println!(
            "{:?} | {}",
            result.store,
            formatter.format_termctx(&result.term)
        );
        let result = one_step_eval(result).unwrap();
        println!(
            "{:?} | {}",
            result.store,
            formatter.format_termctx(&result.term)
        );
        let result = one_step_eval(result).unwrap();
        println!(
            "{:?} | {}",
            result.store,
            formatter.format_termctx(&result.term)
        );
        let result = one_step_eval(result).unwrap();
        println!(
            "{:?} | {}",
            result.store,
            formatter.format_termctx(&result.term)
        );
    }
}
