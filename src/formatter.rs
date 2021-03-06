use crate::syntax::{ArithOp, Pretype, Qualifier, Term, TermCtx, Type};

/// The tab width is 4 spaces
const INDENT: &str = "    ";
pub const DEFAULT_LINE_WIDTH: usize = 80;

pub struct TermFormatter {
    indent: usize,
    line_width: usize,
}

pub fn format_termctx(t: &TermCtx) -> String {
    let mut formatter = TermFormatter::new(DEFAULT_LINE_WIDTH);
    formatter.format_termctx(t)
}

impl TermFormatter {
    pub fn new(line_width: usize) -> Self {
        Self {
            indent: 0,
            line_width,
        }
    }

    pub fn format_termctx(&mut self, t: &TermCtx) -> String {
        self.write_termctx(t, false)
    }

    pub fn format_type(&mut self, t: &Type) -> String {
        self.write_type(t, false)
    }

    fn write_termctx(&mut self, t: &TermCtx, need_bracket: bool) -> String {
        let TermCtx(_, t) = t;
        self.write_term(&t, need_bracket)
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        self.indent -= 1;
    }

    fn line_limit(&self) -> usize {
        self.line_width - self.indent * INDENT.len()
    }

    fn write_indent(&mut self, i: usize) -> String {
        let mut s = String::new();
        for _ in 0..self.indent + i {
            s.push_str(INDENT);
        }
        s
    }

    fn write_qualifer(&mut self, q: &Qualifier) -> String {
        match q {
            Qualifier::Nop => "",
            Qualifier::Linear => "$",
        }
        .to_string()
    }

    fn write_type(&mut self, t: &Type, need_bracket: bool) -> String {
        let Type(q, t) = t;
        let s = match t {
            Pretype::Boolean => "bool".to_owned(),
            Pretype::Integer => "int".to_owned(),
            Pretype::Function(t1, t2) => {
                // the arrow is right-associated.
                let left_is_arrow = match **t1 {
                    Type(Qualifier::Nop, Pretype::Function(..)) => true,
                    _ => false,
                };
                format!(
                    "{}->{}",
                    self.write_type(t1, left_is_arrow),
                    self.write_type(t2, false)
                )
            }
            Pretype::Compound(t1, t2) => {
                format!(
                    "<{}, {}>",
                    self.write_type(t1, false),
                    self.write_type(t2, false)
                )
            }
        };
        let q = self.write_qualifer(q);
        // TODO: refactor
        let t_if_function = match t {
            Pretype::Function(..) => true,
            _ => false,
        };
        let s = if !q.is_empty() && t_if_function {
            format!("({})", s)
        } else {
            s
        };
        let result = format!("{}{}", q, s);
        let result = if need_bracket {
            format!("({})", result)
        } else {
            result
        };
        result
    }

    fn write_term(&mut self, t: &Term, need_bracket: bool) -> String {
        match t {
            Term::Variable(v) => v.to_string(),
            Term::Boolean(q, b) => format!("{}{}", self.write_qualifer(q), b),
            Term::Integer(q, i) => format!("{}{}", self.write_qualifer(q), i),
            Term::Compound(..) => self.write_term_compound(t, need_bracket),
            Term::Let(..) => self.write_term_let(t, need_bracket),
            Term::Letc(..) => self.write_term_letc(t, need_bracket),
            Term::Arith1(..) | Term::Arith2(..) => self.write_term_arith(t, need_bracket),
            Term::Application(t1, t2) => {
                let need_backet_on_s1 = match **t1 {
                    TermCtx(_, Term::Abstraction(..)) => true,
                    TermCtx(_, Term::Fix(..)) => true,
                    TermCtx(_, Term::Let(..)) => true,
                    TermCtx(_, Term::Letc(..)) => true,
                    _ => false,
                };
                let s1 = self.write_termctx(t1, need_backet_on_s1);
                let s2 = self.write_termctx(t2, false);
                let oneline = format!("{} ({})", s1, s2);
                let result = if s1.contains("\n")
                    || s2.contains("\n")
                    || oneline.len() > self.line_limit()
                {
                    format!("{}\n{}({})", s1, self.write_indent(0), s2)
                } else {
                    oneline
                };
                result
            }
            Term::Conditional(..) => self.write_term_conditional(t, need_bracket),
            Term::Abstraction(q, x, t, t1) => {
                self.indent();
                let s1 = self.write_termctx(t1, false);
                self.dedent();
                let t = match t {
                    Some(t) => format!(": {}", self.format_type(t)),
                    None => "".to_owned(),
                };
                let oneline = format!("{}|{}{}| {}", self.write_qualifer(q), x, t, s1);
                let oneline = if need_bracket {
                    format!("({})", oneline)
                } else {
                    oneline
                };
                if s1.contains("\n") || oneline.len() > self.line_limit() {
                    let result = format!(
                        "{}|{}{}|\n{}{}",
                        self.write_qualifer(q),
                        x,
                        t,
                        self.write_indent(1),
                        s1
                    );
                    let result = if need_bracket {
                        format!("({})", result)
                    } else {
                        result
                    };
                    result
                } else {
                    oneline
                }
            }
            Term::Fix(t) => {
                let s = self.write_termctx(t, false);
                let result = format!("fix {}", s);
                let result = if need_bracket {
                    format!("({})", result)
                } else {
                    result
                };
                result
            }
        }
    }

    fn write_term_compound(&mut self, t: &Term, _need_bracket: bool) -> String {
        if let Term::Compound(q, t1, t2) = t {
            let t1 = self.write_termctx(&**t1, false);
            let t2 = self.write_termctx(&**t2, false);
            format!("{}<{}, {}>", self.write_qualifer(q), t1, t2)
            // TODO: insert new line if t1 + t2 is too long
        } else {
            unreachable!();
        }
    }

    fn write_term_conditional(&mut self, t: &Term, _need_bracket: bool) -> String {
        if let Term::Conditional(t1, t2, t3) = t {
            self.indent();
            let s1 = self.write_termctx(t1, false);
            let s2 = self.write_termctx(t2, false);
            let s3 = self.write_termctx(t3, false);
            self.dedent();
            let oneline = format!("if {} {{ {} }} else {{ {} }}", s1, s2, s3);
            if s1.contains("\n")
                || s2.contains("\n")
                || s3.contains("\n")
                || oneline.len() > self.line_limit()
            {
                format!(
                    "if {} {{\n{}{}\n{}}} else {{\n{}{}\n{}}}",
                    s1,
                    self.write_indent(1),
                    s2,
                    self.write_indent(0),
                    self.write_indent(1),
                    s3,
                    self.write_indent(0),
                )
            } else {
                oneline
            }
        } else {
            unreachable!();
        }
    }

    fn write_term_let(&mut self, t: &Term, need_bracket: bool) -> String {
        if let Term::Let(v, t1, t2) = t {
            self.indent();
            let s1 = self.write_termctx(t1, false);
            self.dedent();
            let s2 = self.write_termctx(t2, false);
            let oneline = format!("let {} = {} in {}", v, s1, s2);
            let result =
                if s1.contains("\n") || s2.contains("\n") || oneline.len() > self.line_limit() {
                    format!("let {} = {} in\n{}{}", v, s1, self.write_indent(0), s2,)
                } else {
                    oneline
                };
            let result = if need_bracket {
                format!("({})", result)
            } else {
                result
            };
            result
        } else {
            unreachable!();
        }
    }

    fn write_term_letc(&mut self, t: &Term, need_bracket: bool) -> String {
        if let Term::Letc(v1, v2, t1, t2) = t {
            self.indent();
            let s1 = self.write_termctx(t1, false);
            self.dedent();
            let s2 = self.write_termctx(t2, false);
            let oneline = format!("let <{}, {}> = {} in {}", v1, v2, s1, s2);
            let result =
                if s1.contains("\n") || s2.contains("\n") || oneline.len() > self.line_limit() {
                    format!(
                        "let <{}, {}> = {} in\n{}{}",
                        v1,
                        v2,
                        s1,
                        self.write_indent(0),
                        s2,
                    )
                } else {
                    oneline
                };
            let result = if need_bracket {
                format!("({})", result)
            } else {
                result
            };
            result
        } else {
            unreachable!();
        }
    }

    fn write_term_arith(&mut self, t: &Term, _need_bracket: bool) -> String {
        match t {
            Term::Arith2(q, ArithOp::Diff, t1, t2) => {
                let t1 = self.write_termctx(&**t1, false);
                let t2 = self.write_termctx(&**t2, false);
                format!("{}diff({}, {})", self.write_qualifer(q), t1, t2)
                // TODO: insert new line if the result is too long
            }
            Term::Arith1(q, ArithOp::IsZero, t1) => {
                let t1 = self.write_termctx(&**t1, false);
                format!("{}iszero({})", self.write_qualifer(q), t1)
                // TODO: insert new line if the result is too long
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::syntax::parse_program;

    use super::*;

    #[test]
    fn test_formatter_01() {
        let prog = [
            "if x { y } else { z }",
            "$123",
            "|x| y",
            "$|x| $true",
            "x (y)",
            "x (y) (z)",
            "x (y (z))",
            "x (y (z)) (w)",
            "(|x: bool| x) (false)",
            "(|x: bool->bool->bool| x) (false)",
            "(|x: (bool->bool)->bool| x) (false)",
            "|x: $($int->bool)| x ($5)",
            "|x: $($int->bool)->int| x",
            "(fix 1) (2)",
            "(let x = 1 in true) (2)",
            "|x: $<int, bool>| x",
            "|x: $<int, bool>->int| x",
            "|x: $(<int, bool>->int)| x",
            "let <a, b> = 3 in 4",
            "$<5, $6>",
            "$iszero(false)",
            "diff(false, 1)",
        ];
        for p in prog.iter() {
            let result = format_termctx(&parse_program(p).unwrap());
            assert_eq!(result, p.to_owned());
        }
    }
}
