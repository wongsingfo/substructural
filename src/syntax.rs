use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct IdentParser;

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
    }

    #[test]
    fn test_keyword() {
        let input = "else";
        assert!(IdentParser::parse(Rule::program, input).is_err());
        let input = "1";
        assert!(IdentParser::parse(Rule::program, input).is_err());
        let input = "1a";
        assert!(IdentParser::parse(Rule::program, input).is_err());
    }

    #[test]
    fn test_conditional() {
        let input = "if x{ a}else {y}";
        assert!(IdentParser::parse(Rule::program, input).is_ok());
    }

    #[test]
    fn test_pretype() {
        let input = "Bool";
        assert!(IdentParser::parse(Rule::pretype, input).is_ok());
        let input = "fn( Bool) ->Bool";
        assert!(IdentParser::parse(Rule::pretype, input).is_ok());
        let input = "fn(Bool)->fn(Bool)->Bool";
        assert!(IdentParser::parse(Rule::pretype, input).is_ok());
        let input = "fn(fn( Bool) ->Bool) -> fn(Bool) ->Bool";
        assert!(IdentParser::parse(Rule::pretype, input).is_ok());
    }
}
