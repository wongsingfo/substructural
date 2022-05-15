use crate::syntax::{Pretype, Qualifier, Term, TermCtx, Type};

/// The tab width is 4 spaces
const INDENT: &str = "    ";

pub struct TermFormatter {
    indent: usize,
    line_width: usize,
}

pub fn format_termctx(t: &TermCtx) -> String {
    let mut formatter = TermFormatter::new();
    formatter.format_termctx(t)
}

impl TermFormatter {
    pub fn new() -> Self {
        Self {
            indent: 0,
            // TODO: allow changes to line_width
            line_width: 40,
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
        };
        let q = self.write_qualifer(q);
        // TODO: refactor 
        let t_if_function = match t {
            Pretype::Function(..) => true,
            _ => false,
        };
        let s = if !q.is_empty() && t_if_function {
            format!("({})", s)
        }else {
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
            Term::Application(t1, t2) => {
                let need_backet_on_s1 = match **t1 {
                    TermCtx(_, Term::Abstraction(..)) => true,
                    _ => false,
                };
                let s1 = self.write_termctx(t1, need_backet_on_s1);
                let s2 = self.write_termctx(t2, false);
                let oneline = format!("{} ({})", s1, s2);
                let result =
                    if s1.contains("\n") || s2.contains("\n") || oneline.len() > self.line_width {
                        format!("{}\n{}({})", s1, self.write_indent(0), s2)
                    } else {
                        oneline
                    };
                result
            }
            Term::Conditional(t1, t2, t3) => {
                self.indent();
                let s1 = self.write_termctx(t1, false);
                let s2 = self.write_termctx(t2, false);
                let s3 = self.write_termctx(t3, false);
                self.dedent();
                let oneline = format!("if {} {{ {} }} else {{ {} }}", s1, s2, s3);
                if s1.contains("\n")
                    || s2.contains("\n")
                    || s3.contains("\n")
                    || oneline.len() > self.line_width
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
            }
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
                if s1.contains("\n") || oneline.len() > self.line_width {
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
        ];
        for p in prog.iter() {
            let result = format_termctx(&parse_program(p).unwrap());
            assert_eq!(result, p.to_owned());
        }
    }
}
