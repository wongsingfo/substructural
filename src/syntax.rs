use crate::error::Error;
use pest::iterators::{Pair, Pairs};
use pest::{Parser, Span};
use pest_derive::Parser;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct IdentParser;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Debug)]
pub struct Context {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TermCtx(pub Context, pub Term);

impl fmt::Debug for TermCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let TermCtx(Context { start, end }, _) = self;
        f.debug_tuple("")
            .field(&format!("context: {}~{}", start, end))
            .field(&self.1)
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArithOp {
    Diff,
    IsZero,
    // TODO: add more primitive operations
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Term {
    Variable(String),
    Boolean(Qualifier, bool),
    Integer(Qualifier, i64),
    Compound(Qualifier, Box<TermCtx>, Box<TermCtx>),
    Arith1(Qualifier, ArithOp, Box<TermCtx>),
    Arith2(Qualifier, ArithOp, Box<TermCtx>, Box<TermCtx>),
    Abstraction(Qualifier, String, Option<Box<Type>>, Box<TermCtx>),
    Application(Box<TermCtx>, Box<TermCtx>),
    Conditional(Box<TermCtx>, Box<TermCtx>, Box<TermCtx>),
    Fix(Box<TermCtx>), // all recursive functions are unrestricted data structures
    Let(String, Box<TermCtx>, Box<TermCtx>),
    Letc(String, String, Box<TermCtx>, Box<TermCtx>),
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Type(pub Qualifier, pub Pretype);

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Copy)]
pub enum Qualifier {
    Nop,
    Linear,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pretype {
    Boolean,
    Integer,
    Function(Box<Type>, Box<Type>),
    Compound(Box<Type>, Box<Type>),
}

pub fn parse_program(input: &str) -> Result<TermCtx, Error> {
    let pairs: Pairs<Rule> = IdentParser::parse(Rule::program, input)?;
    Ok(parse_pairs(pairs)?.0)
}

impl Context {
    pub fn to_string(&self) -> String {
        let mut s = String::new();
        s.push_str("[");
        s.push_str(&self.start.to_string());
        s.push_str(":");
        s.push_str(&self.end.to_string());
        s.push_str("]");
        s
    }
}

impl<'a> From<Span<'a>> for Context {
    fn from(span: Span<'a>) -> Self {
        Context {
            start: span.start(),
            end: span.end(),
        }
    }
}

impl PartialEq for Pretype {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Pretype::Boolean, Pretype::Boolean) => true,
            (Pretype::Integer, Pretype::Integer) => true,
            (Pretype::Function(a1, b1), Pretype::Function(a2, b2)) => a1 == a2 && b1 == b2,
            _ => false,
        }
    }
}

impl Eq for Pretype {}

fn parse_pairs(mut pairs: Pairs<Rule>) -> Result<(TermCtx, Pairs<Rule>), Error> {
    let pair1 = pairs.next().unwrap();
    let mut term1 = parse_pair(pair1)?;

    while let Some(Rule::application) = pairs.peek().map(|p| p.as_rule()) {
        let pairs2 = pairs.next().unwrap().into_inner();
        let (term2, _) = parse_pairs(pairs2)?;

        let TermCtx(Context { start, .. }, _) = term1;
        let TermCtx(Context { end, .. }, _) = term2;
        term1 = TermCtx(
            Context { start, end },
            Term::Application(Box::new(term1), Box::new(term2)),
        )
    }

    Ok((term1, pairs))
}

fn parse_pair(pair: Pair<Rule>) -> Result<TermCtx, Error> {
    let term1 = match pair.as_rule() {
        Rule::literal => parse_pair_literal(pair),
        Rule::variable => {
            let source = pair.as_span();
            let name = pair.as_str();
            Ok(TermCtx(source.into(), Term::Variable(name.to_string())))
        }
        Rule::conditional => {
            let mut inner = pair.into_inner();
            let kw_if = inner.next().unwrap();
            let span1 = kw_if.as_span();
            let span2 = inner.peek().unwrap().as_span();
            let (term1, inner) = parse_pairs(inner)?;
            let (term2, inner) = parse_pairs(inner)?;
            let (term3, _) = parse_pairs(inner)?;
            let source = span1.start_pos().span(&span2.end_pos());
            Ok(TermCtx(
                source.into(),
                Term::Conditional(Box::new(term1), Box::new(term2), Box::new(term3)),
            ))
        }
        Rule::bracket => {
            let inner = pair.into_inner();
            let (term1, _) = parse_pairs(inner)?;
            Ok(term1)
        }
        Rule::abstraction => parse_pair_abstraction(pair),
        Rule::fix => parse_pair_fix(pair),
        Rule::letv => parse_pair_let(pair),
        Rule::letc => parse_pair_letc(pair),
        Rule::diff => parse_pair_diff(pair),
        Rule::iszero => parse_pair_iszero(pair),
        _ => Err(Error::ParseError {
            message: format!("Unexpected rule: {:?}", pair.as_rule()),
            start: pair.as_span().start(),
            end: pair.as_span().end(),
        }),
    };
    term1
}

fn parse_pair_literal(pair: Pair<Rule>) -> Result<TermCtx, Error> {
    let source = pair.as_span();
    let mut inner = pair.into_inner();
    let qualifier = if let Rule::qualifier = inner.peek().unwrap().as_rule() {
        parse_qualifier(inner.next().unwrap())
    } else {
        Qualifier::Nop
    };
    let literal = inner.next().unwrap();
    let string = literal.as_str();
    match literal.as_rule() {
        Rule::boolean => {
            let value = string.parse::<bool>().unwrap();
            Ok(TermCtx(source.into(), Term::Boolean(qualifier, value)))
        }
        Rule::number => {
            let value = string.parse::<i64>().unwrap();
            Ok(TermCtx(source.into(), Term::Integer(qualifier, value)))
        }
        Rule::compound => {
            let inner = literal.into_inner();
            let (first, mut inner) = parse_pairs(inner)?;
            let comma = inner.next().unwrap();
            let (second, _) = parse_pairs(inner)?;
            Ok(TermCtx(
                comma.as_span().into(),
                Term::Compound(qualifier, Box::new(first), Box::new(second)),
            ))
        }
        _ => Err(Error::ParseError {
            message: "unexpected literal".to_string(),
            start: source.start(),
            end: source.end(),
        }),
    }
}

fn parse_pair_abstraction(pair: Pair<Rule>) -> Result<TermCtx, Error> {
    let mut inner = pair.into_inner();
    let mut qualifier_ctx: Option<Span> = None;
    let qualifier = if let Rule::qualifier = inner.peek().unwrap().as_rule() {
        let p = inner.next().unwrap();
        qualifier_ctx = Some(p.as_span());
        parse_qualifier(p)
    } else {
        Qualifier::Nop
    };
    let vertical_bar1 = inner.next().unwrap();
    let variable = inner.next().unwrap();
    let variable = variable.as_str();
    if let Some(Rule::typing) = inner.peek().map(|p| p.as_rule()) {
        let typing = parse_typing(inner.next().unwrap())?;
        let vertical_bar2 = inner.next().unwrap();
        let start = qualifier_ctx.map_or(vertical_bar1.as_span().start(), |p| p.start());
        let end = vertical_bar2.as_span().end();
        let source = Context { start, end };
        let (term1, _) = parse_pairs(inner)?;
        Ok(TermCtx(
            source,
            Term::Abstraction(
                qualifier,
                variable.to_string(),
                Some(Box::new(typing)),
                Box::new(term1),
            ),
        ))
    } else {
        let vertical_bar2 = inner.next().unwrap();
        let start = vertical_bar1.as_span().start();
        let end = vertical_bar2.as_span().end();
        let source = Context { start, end };
        let (term1, _) = parse_pairs(inner)?;
        Ok(TermCtx(
            source.into(),
            Term::Abstraction(qualifier, variable.to_string(), None, Box::new(term1)),
        ))
    }
}

fn parse_pair_diff(pair: Pair<Rule>) -> Result<TermCtx, Error> {
    let mut inner = pair.into_inner();
    let qualifier = if let Rule::qualifier = inner.peek().unwrap().as_rule() {
        let p = inner.next().unwrap();
        parse_qualifier(p)
    } else {
        Qualifier::Nop
    };
    let kw = inner.next().unwrap();
    let (t1, mut inner) = parse_pairs(inner)?;
    let _comma = inner.next();
    let (t2, _) = parse_pairs(inner)?;
    Ok(TermCtx(
        kw.as_span().into(),
        Term::Arith2(qualifier, ArithOp::Diff, Box::new(t1), Box::new(t2)),
    ))
}

fn parse_pair_iszero(pair: Pair<Rule>) -> Result<TermCtx, Error> {
    let mut inner = pair.into_inner();
    let qualifier = if let Rule::qualifier = inner.peek().unwrap().as_rule() {
        let p = inner.next().unwrap();
        parse_qualifier(p)
    } else {
        Qualifier::Nop
    };
    let kw = inner.next().unwrap();
    let (t1, mut _inner) = parse_pairs(inner)?;
    Ok(TermCtx(
        kw.as_span().into(),
        Term::Arith1(qualifier, ArithOp::IsZero, Box::new(t1)),
    ))
}

fn parse_pair_fix(pair: Pair<Rule>) -> Result<TermCtx, Error> {
    let mut inner = pair.into_inner();
    let kw = inner.next().unwrap().as_span();
    let (t, _) = parse_pairs(inner)?;
    Ok(TermCtx(kw.into(), Term::Fix(Box::new(t))))
}

fn parse_pair_let(pair: Pair<Rule>) -> Result<TermCtx, Error> {
    let mut inner = pair.into_inner();
    let kw_let = inner.next().unwrap().as_span();
    let var = inner.next().unwrap().as_str().to_owned();
    let (t1, mut inner) = parse_pairs(inner)?;
    let _kw_in = inner.next().unwrap().as_span();
    let (t2, _) = parse_pairs(inner)?;

    Ok(TermCtx(
        kw_let.into(),
        Term::Let(var, Box::new(t1), Box::new(t2)),
    ))
}

fn parse_pair_letc(pair: Pair<Rule>) -> Result<TermCtx, Error> {
    let mut inner = pair.into_inner();
    let kw_let = inner.next().unwrap().as_span();
    let var1 = inner.next().unwrap().as_str().to_owned();
    let _comma = inner.next().unwrap().as_span();
    let var2 = inner.next().unwrap().as_str().to_owned();
    let (t1, mut inner) = parse_pairs(inner)?;
    let _kw_in = inner.next().unwrap().as_span();
    let (t2, _) = parse_pairs(inner)?;

    Ok(TermCtx(
        kw_let.into(),
        Term::Letc(var1, var2, Box::new(t1), Box::new(t2)),
    ))
}

fn parse_typing(pair: Pair<Rule>) -> Result<Type, Error> {
    let inner = pair.into_inner();
    let t0 = inner.map(parse_typing0);
    t0.rev()
        .reduce(|rhs, lhs| match (rhs, lhs) {
            (Ok(rhs), Ok(lhs)) => Ok(Type(
                Qualifier::Nop,
                Pretype::Function(Box::new(lhs), Box::new(rhs)),
            )),
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e),
        })
        .unwrap()
}

fn parse_typing0(pair: Pair<Rule>) -> Result<Type, Error> {
    if pair.as_rule() != Rule::typing0 {
        Err(Error::ParseError {
            message: format!("Unexpected typing0 rule: {:?}", pair.as_rule()),
            start: pair.as_span().start(),
            end: pair.as_span().end(),
        })
    } else {
        let mut inner = pair.into_inner();
        let qualifier = if inner.peek().map(|p| p.as_rule()) == Some(Rule::qualifier) {
            let qualifier = inner.next().unwrap();
            Some(parse_qualifier(qualifier))
        } else {
            None
        };
        let pair = inner.next().unwrap();
        let start = pair.as_span().start();
        let end = pair.as_span().end();
        let Type(q, pretype) = match pair.as_rule() {
            Rule::kw_int => Type(Qualifier::Nop, Pretype::Integer),
            Rule::kw_bool => Type(Qualifier::Nop, Pretype::Boolean),
            Rule::typing => parse_typing(pair)?,
            Rule::typing_compound => {
                let mut inner = pair.into_inner();
                let first = parse_typing(inner.next().unwrap())?;
                let _comma = inner.next();
                let second = parse_typing(inner.next().unwrap())?;
                Type(
                    Qualifier::Nop,
                    Pretype::Compound(Box::new(first), Box::new(second)),
                )
            }
            _ => {
                return Err(Error::ParseError {
                    message: format!("Unexpected typing0: {:?}", pair.as_rule()),
                    start: pair.as_span().start(),
                    end: pair.as_span().end(),
                })
            }
        };
        if let Some(qualifier) = qualifier {
            if q != Qualifier::Nop {
                return Err(Error::ParseError {
                    message: format!("Incompatible qualifiers {:?} and {:?}", q, qualifier),
                    start,
                    end,
                });
            }
            Ok(Type(qualifier, pretype))
        } else {
            Ok(Type(q, pretype))
        }
    }
}

fn parse_qualifier(_pair: Pair<Rule>) -> Qualifier {
    // We only have one single qualifier here and we directly return it.
    return Qualifier::Linear;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_keyword() {
        let input = "else";
        assert!(IdentParser::parse(Rule::program, input).is_err());
        let input = "if";
        assert!(IdentParser::parse(Rule::program, input).is_err());
        let input = "if0";
        assert!(IdentParser::parse(Rule::program, input).is_ok());
    }

    #[test]
    fn test_variable() {
        let input = "_";
        assert!(IdentParser::parse(Rule::program, input).is_ok());
        let input = "ifhello";
        assert!(IdentParser::parse(Rule::program, input).is_ok());
        let input = "__123";
        assert!(IdentParser::parse(Rule::program, input).is_ok());
        let input = "0123";
        assert!(IdentParser::parse(Rule::program, input).is_ok());
        let input = "true0";
        assert!(IdentParser::parse(Rule::program, input).is_ok());
        let input = "123a";
        assert!(IdentParser::parse(Rule::program, input).is_err());
        let input = "0a";
        assert!(IdentParser::parse(Rule::program, input).is_err());
    }

    #[test]
    fn test_conditional() {
        let input = "if x{ a}else {y}";
        assert!(IdentParser::parse(Rule::program, input).is_ok());
        let input = "ifx{ a}else {y}";
        assert!(IdentParser::parse(Rule::program, input).is_err());
    }

    #[test]
    fn test_parse_prog01() {
        let input = "x (x (if y { a } else { b }) (y)) (z)";
        let output = IdentParser::parse(Rule::program, input).unwrap();
        println!("{:#?}", output);
        println!("{:#?}", parse_program(input).unwrap());
    }

    #[test]
    fn test_parse_prog02() {
        let input = "(x(y))(z)";
        let output = IdentParser::parse(Rule::program, input).unwrap();
        println!("{:#?}", output);
        println!("{:#?}", parse_program(input).unwrap());
    }

    #[test]
    fn test_parse_prog03() {
        let input = "|x: $(bool) -> $bool| y";
        let output = IdentParser::parse(Rule::program, input).unwrap();
        println!("{:#?}", output);
        println!("{:#?}", parse_program(input).unwrap());
    }

    #[test]
    fn test_parse_prog04() {
        let input = "|x: $($(bool) -> ($bool)) -> int| y";
        let output = IdentParser::parse(Rule::program, input).unwrap();
        println!("{:#?}", output);
        println!("{:#?}", parse_program(input).unwrap());
    }

    #[test]
    fn test_parse_prog05() {
        let input = "|x: $($bool)| y";
        let output = IdentParser::parse(Rule::program, input).unwrap();
        println!("{:#?}", output);
        assert!(parse_program(input).is_err());
    }

    #[test]
    fn test_parse_prog06() {
        let input = "(|x: $bool| if x { false } else { true }) ($true)";
        let output = IdentParser::parse(Rule::program, input).unwrap();
        println!("{:#?}", output);
        let output = parse_program(input).unwrap();
        println!("{:#?}", output);
    }

    #[test]
    fn test_parse_literal() {
        let input = "true";
        let output = IdentParser::parse(Rule::program, input).unwrap();
        println!("{:#?}", output);
        println!("{:#?}", parse_program(input).unwrap());
    }

    #[test]
    fn test_linear_values() {
        let input = "$123";
        let output = IdentParser::parse(Rule::program, input).unwrap();
        println!("{:#?}", output);
        println!("{:#?}", parse_program(input).unwrap());

        let input = "$    false";
        let output = IdentParser::parse(Rule::program, input).unwrap();
        println!("{:#?}", output);
        println!("{:#?}", parse_program(input).unwrap());

        let input = "$ |x| x";
        let output = IdentParser::parse(Rule::program, input).unwrap();
        println!("{:#?}", output);
        println!("{:#?}", parse_program(input).unwrap());
    }

    #[test]
    fn test_fix() {
        let input = "fix |x| |y| x";
        let output = IdentParser::parse(Rule::program, input).unwrap();
        println!("{:#?}", output);
        println!("{:#?}", parse_program(input).unwrap());
    }

    #[test]
    fn test_let() {
        let input = "let x = 1 (2) in 3 (4)";
        let output = IdentParser::parse(Rule::program, input).unwrap();
        println!("{:#?}", output);
        println!("{:#?}", parse_program(input).unwrap());
    }

    #[test]
    fn test_comment() {
        let input = "5 // comment\n";
        let _output = IdentParser::parse(Rule::program, input).unwrap();
        let input = "5 //comment";
        let _output = IdentParser::parse(Rule::program, input).unwrap();
        let input = "5 /* comment*/";
        let _output = IdentParser::parse(Rule::program, input).unwrap();
    }

    #[test]
    fn test_compound_value() {
        let input = "$<p (1),q (2)> ";
        let output = IdentParser::parse(Rule::program, input).unwrap();
        println!("{:#?}", output);
        println!("{:#?}", parse_program(input).unwrap());
    }

    #[test]
    fn test_compound_let() {
        let input = "let <x,y >= 1 (2) in 3 (4)";
        let output = IdentParser::parse(Rule::program, input).unwrap();
        println!("{:#?}", output);
        println!("{:#?}", parse_program(input).unwrap());
    }

    #[test]
    fn test_compound_typing() {
        let input = "|x : $<int->$bool, $bool> | 1";
        let output = IdentParser::parse(Rule::program, input).unwrap();
        println!("{:#?}", output);
        println!("{:#?}", parse_program(input).unwrap());
    }

    #[test]
    fn test_arith() {
        let input = "$diff(1, 2)";
        let output = IdentParser::parse(Rule::program, input).unwrap();
        println!("{:#?}", output);
        println!("{:#?}", parse_program(input).unwrap());
    }
}
