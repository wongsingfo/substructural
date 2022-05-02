use crate::syntax::{Qualifier, Term, TermCtx};

const INDENT: &str = "  ";

struct TermFormatter {
    indent: usize,
    line_width: usize,
}

pub fn format_termctx(t: &TermCtx) -> String {
    let mut formatter = TermFormatter::new();
    formatter.write_termctx(t)
}

impl TermFormatter {
    pub fn new() -> Self {
        Self {
            indent: 0,
            line_width: 80,
        }
    }

    pub fn write_termctx(&mut self, t: &TermCtx) -> String {
        let TermCtx(_, t) = t;
        self.write_term(&t)
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

    fn write_term(&mut self, t: &Term) -> String {
        match t {
            Term::Variable(v) => v.to_string(),
            Term::Boolean(q, b) => format!("{}{}", self.write_qualifer(q), b),
            Term::Integer(q, i) => format!("{}{}", self.write_qualifer(q), i),
            Term::Application(t1, t2) => {
                let s1 = self.write_termctx(t1);
                let s2 = self.write_termctx(t2);
                let oneline = format!("{} ({})", s1, s2);
                if s1.contains("\n") || s2.contains("\n") || oneline.len() > self.line_width {
                    format!("{}\n{}({})", s1, self.write_indent(0), s2)
                } else {
                    oneline
                }
            }
            Term::Conditional(t1, t2, t3) => {
                self.indent();
                let s1 = self.write_termctx(t1);
                let s2 = self.write_termctx(t2);
                let s3 = self.write_termctx(t3);
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
            Term::Abstraction(q, x, _xtype, t1) => {
                self.indent();
                let s1 = self.write_termctx(t1);
                self.dedent();
                let oneline = format!("{}|{}| {}", self.write_qualifer(q), x, s1);
                if s1.contains("\n") || oneline.len() > self.line_width {
                    format!(
                        "{}|{}|\n{}{}",
                        self.write_qualifer(q),
                        x,
                        self.write_indent(1),
                        s1
                    )
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
        ];
        for p in prog.iter() {
            let mut formatter = TermFormatter::new();
            let result = formatter.write_termctx(&parse_program(p).unwrap());
            assert_eq!(result, p.to_owned());
        }
    }
}
