use crate::error::Error;
use pest::iterators::{Pair, Pairs};
use pest::{Parser, Span};
use pest_derive::Parser;
use std::fmt;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct IdentParser;

pub struct TermCtx<'a>(Span<'a>, Term<'a>);

impl<'a> fmt::Debug for TermCtx<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.1.fmt(f)
    }
}

#[derive(Debug)]
pub enum Term<'a> {
    Variable(String),
    Boolean(bool),
    Abstraction(String, Option<Box<Type>>, Box<TermCtx<'a>>),
    Application(Box<TermCtx<'a>>, Box<TermCtx<'a>>),
    Conditional(Box<TermCtx<'a>>, Box<TermCtx<'a>>, Box<TermCtx<'a>>),
}

#[derive(Debug)]
pub struct Type(Qualifier, Pretype);

#[derive(Debug)]
pub enum Qualifier {
    Un,
    Lin,
}

#[derive(Debug)]
pub enum Pretype {
    Boolean,
    Function(Box<Type>, Box<Type>),
}

pub fn parse_program(input: &str) -> Result<TermCtx, Error> {
    let pairs: Pairs<Rule> = IdentParser::parse(Rule::program, input)?;
    Ok(parse_pairs(pairs)?.0)
}

fn parse_pairs(mut pairs: Pairs<Rule>) -> Result<(TermCtx, Pairs<Rule>), Error> {
    let pair1 = pairs.next().unwrap();
    let mut term1 = parse_pair(pair1)?;

    while let Some(Rule::application) = pairs.peek().map(|p| p.as_rule()) {
        let pairs2 = pairs.next().unwrap().into_inner();
        let (term2, _) = parse_pairs(pairs2)?;
        let source = term2.0.clone();
        term1 = TermCtx(source, Term::Application(Box::new(term1), Box::new(term2)))
    }

    Ok((term1, pairs))
}

fn parse_pair(pair: Pair<Rule>) -> Result<TermCtx, Error> {
    let term1 = match pair.as_rule() {
        Rule::literal => {
            let source = pair.as_span();
            let string = pair.as_str();
            match pair.into_inner().next().unwrap().as_rule() {
                Rule::boolean => {
                    let value = string.parse::<bool>().unwrap();
                    Ok(TermCtx(source, Term::Boolean(value)))
                }
                _ => Err(Error::ParseError {
                    source: source.as_str().to_string(),
                    message: "unexpected literal".to_string(),
                }),
            }
        }
        Rule::variable => {
            let source = pair.as_span();
            let name = pair.as_str();
            Ok(TermCtx(source, Term::Variable(name.to_string())))
        }
        Rule::conditional => {
            let mut inner = pair.into_inner();
            let kw_if = inner.next().unwrap();
            let span1 = kw_if.as_span();
            let (term1, inner) = parse_pairs(inner)?;
            let (term2, inner) = parse_pairs(inner)?;
            let (term3, _) = parse_pairs(inner)?;
            let span2 = &term1.0;
            let source = span1.start_pos().span(&span2.end_pos());
            Ok(TermCtx(
                source,
                Term::Conditional(Box::new(term1), Box::new(term2), Box::new(term3)),
            ))
        }
        Rule::bracket => {
            let inner = pair.into_inner();
            let (term1, _) = parse_pairs(inner)?;
            Ok(term1)
        }
        Rule::abstraction => {
            let source = pair.as_span();
            let mut inner = pair.into_inner();
            let variable = inner.next().unwrap().as_str();
            if let Some(Rule::typing) = inner.peek().map(|p| p.as_rule()) {
                let typing = parse_typing(inner.next().unwrap())?;
                let (term1, _) = parse_pairs(inner)?;
                Ok(TermCtx(
                    source,
                    Term::Abstraction(
                        variable.to_string(),
                        Some(Box::new(typing)),
                        Box::new(term1),
                    ),
                ))
            } else {
                let (term1, _) = parse_pairs(inner)?;
                Ok(TermCtx(
                    source,
                    Term::Abstraction(variable.to_string(), None, Box::new(term1)),
                ))
            }
        }
        _ => Err(Error::ParseError {
            message: format!("Unexpected rule: {:?}", pair.as_rule()),
            source: pair.as_str().to_string(),
        }),
    };
    term1
}

fn parse_typing(pair: Pair<Rule>) -> Result<Type, Error> {
    let mut inner = pair.into_inner();
    if let Some(Rule::qualifier) = inner.peek().map(|p| p.as_rule()) {
        let qualifier = parse_qualifier(inner.next().unwrap())?;
        let pretype = parse_pretype(inner.next().unwrap())?;
        Ok(Type(qualifier, pretype))
    } else {
        // By default, the type is unrestricted.
        let qualifier = Qualifier::Un;
        let pretype = parse_pretype(inner.next().unwrap())?;
        Ok(Type(qualifier, pretype))
    }
}

fn parse_qualifier(pair: Pair<Rule>) -> Result<Qualifier, Error> {
    assert!(pair.as_rule() == Rule::qualifier);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::kw_un => Ok(Qualifier::Un),
        Rule::kw_lin => Ok(Qualifier::Lin),
        _ => Err(Error::ParseError {
            message: format!("Unexpected rule: {:?}", pair.as_rule()),
            source: pair.as_str().to_string(),
        }),
    }
}

fn parse_pretype(pair: Pair<Rule>) -> Result<Pretype, Error> {
    assert!(pair.as_rule() == Rule::pretype);
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::kw_Bool => Ok(Pretype::Boolean),
        Rule::fntype => {
            let mut inner = pair.into_inner();
            let pretype1 = parse_typing(inner.next().unwrap())?;
            let pretype2 = parse_typing(inner.next().unwrap())?;
            Ok(Pretype::Function(Box::new(pretype1), Box::new(pretype2)))
        }
        _ => Err(Error::ParseError {
            message: format!("Unexpected rule: {:?}", pair.as_rule()),
            source: pair.as_str().to_string(),
        }),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_variable() {
        let input = "_";
        assert!(IdentParser::parse(Rule::program, input).is_ok());
        let input = "ifhello";
        assert!(IdentParser::parse(Rule::program, input).is_ok());
        let input = "__123";
        assert!(IdentParser::parse(Rule::program, input).is_ok());
        let input = "1";
        assert!(IdentParser::parse(Rule::program, input).is_err());
        let input = "1a";
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
    fn test_pretype() {
        let input = "Bool";
        assert!(IdentParser::parse(Rule::pretype, input).is_ok());
        let input = "fn(un Bool) -> un Bool";
        assert!(IdentParser::parse(Rule::pretype, input).is_ok());
        let input = "fn(un Bool)->un fn(un Bool)->un Bool";
        assert!(IdentParser::parse(Rule::pretype, input).is_ok());
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
        let input = "|x: lin Bool| y";
        let output = IdentParser::parse(Rule::program, input).unwrap();
        println!("{:#?}", output);
        println!("{:#?}", parse_program(input).unwrap());
    }

    #[test]
    fn test_parse_prog04() {
        let input = "|x: lin fn(un Bool) -> fn (Bool) -> Bool | y";
        let output = IdentParser::parse(Rule::program, input).unwrap();
        println!("{:#?}", output);
        println!("{:#?}", parse_program(input).unwrap());
    }

    #[test]
    fn test_parse_literal() {
        let input = "true";
        let output = IdentParser::parse(Rule::program, input).unwrap();
        println!("{:#?}", output);
        println!("{:#?}", parse_program(input).unwrap());
    }
}
