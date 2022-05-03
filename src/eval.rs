use crate::error::Error;
use crate::syntax::{Qualifier, Term, TermCtx};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Binding {
    name: String,
    value: TermCtx,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    bindings: Vec<Binding>,
}

impl Store {
    pub fn new_empty() -> Store {
        Store {
            bindings: Vec::new(),
        }
    }

    fn push(&mut self, name: String, value: TermCtx) {
        assert!(get_qualifier(&value).is_some());
        self.bindings.push(Binding { name, value });
    }

    fn lookup(&mut self, name: &str) -> Option<TermCtx> {
        let index = self
            .bindings
            .iter()
            .rev()
            .position(|binding| binding.name == name);
        match index {
            Some(index) => {
                let binding = self.bindings.iter().rev().nth(index).unwrap();
                if let Some(Qualifier::Linear) = get_qualifier(&binding.value) {
                    Some(self.bindings.remove(index).value)
                } else {
                    Some(binding.value.clone())
                }
            }
            None => None,
        }
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
        Term::Variable(_) => true,
        Term::Boolean(_, _) => true,
        Term::Integer(_, _) => true,
        Term::Abstraction(_, _, _, _) => true,
        _ => false,
    }
}

fn get_qualifier(term: &TermCtx) -> Option<Qualifier> {
    let TermCtx(_, term) = term;
    let qualifier = match term {
        Term::Boolean(q, _) => Some(q),
        Term::Integer(q, _) => Some(q),
        Term::Abstraction(q, _, _, _) => Some(q),
        _ => None,
    };
    qualifier.map(|q| q.clone())
}

fn eval_value(store: &mut Store, term_ctx: &TermCtx) -> Result<TermCtx, Error> {
    let TermCtx(ctx, term) = term_ctx;
    match term {
        Term::Variable(name) => match store.lookup(name) {
            Some(value) => Ok(value),
            None => Err(Error::EvaluateError {
                message: format!("Variable {} not found", name),
                source: ctx.to_string(),
            }),
        },
        Term::Boolean(_, _) => Ok(term_ctx.to_owned()),
        Term::Integer(_, _) => Ok(term_ctx.to_owned()),
        Term::Abstraction(_, _, _, _) => Ok(term_ctx.to_owned()),
        _ => Err(Error::EvaluateError {
            message: format!("Value expected, but found {:?}", term_ctx),
            source: ctx.to_string(),
        }),
    }
}

pub(crate) fn one_step_eval(term_eval: TermEval) -> Result<TermEval, Error> {
    let TermEval {
        mut store,
        term: TermCtx(ctx, term),
    } = term_eval;
    let term_ = match term {
        Term::Variable(_) => {
            let TermCtx(_, term) = eval_value(&mut store, &TermCtx(ctx.clone(), term))?;
            term
        }
        Term::Boolean(_, _) => term,
        Term::Integer(_, _) => term,
        Term::Abstraction(_, _, _, _) => term,
        Term::Conditional(t1, t2, t3) => {
            if !is_value(&*t1) {
                let TermEval { term: t1_, .. } = one_step_eval(TermEval {
                    store: store.clone(),
                    term: *t1,
                })?;
                Term::Conditional(Box::new(t1_), t2, t3)
            } else {
                let t1_ = eval_value(&mut store, &t1)?;
                let TermCtx(ctx, t1_) = t1_;
                let t1_ = match t1_ {
                    Term::Boolean(_qualifier, v) => v,
                    _ => {
                        return Err(Error::EvaluateError {
                            message: "Conditional term must be boolean".to_owned(),
                            source: ctx.to_string(),
                        })
                    }
                };
                let TermCtx(_, t2_) = *t2;
                let TermCtx(_, t3_) = *t3;
                if t1_ {
                    t2_
                } else {
                    t3_
                }
            }
        }
        Term::Application(t1, t2) => {
            if !is_value(&*t1) {
                let TermEval { term: t1_, .. } = one_step_eval(TermEval {
                    store: store.clone(),
                    term: *t1,
                })?;
                Term::Application(Box::new(t1_), t2)
            } else if !is_value(&*t2) {
                let TermEval { term: t2_, .. } = one_step_eval(TermEval {
                    store: store.clone(),
                    term: *t2,
                })?;
                Term::Application(t1, Box::new(t2_))
            } else {
                let t1_ = eval_value(&mut store, &t1)?;
                let t2_ = eval_value(&mut store, &t2)?;
                let TermCtx(_, t1_) = t1_;
                let TermCtx(_, t2_) = t2_;
                if let Term::Abstraction(_qualifier, v, _type, body) = t1_ {
                    store.push(v, TermCtx(ctx.clone(), t2_));
                    let TermCtx(_, body) = *body;
                    body
                } else {
                    return Err(Error::EvaluateError {
                        message: "Expected abstraction".to_owned(),
                        source: ctx.to_string(),
                    });
                }
            }
        }
    };
    let result = TermEval {
        store,
        term: TermCtx(ctx, term_),
    };
    Ok(result)
}

#[cfg(test)]
mod test {
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
        println!("{:#?}", result);
        assert!(
            result.store.bindings.is_empty(),
            "Store should be empty because `bool` is linear"
        );
    }
}
