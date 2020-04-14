use super::{ast::*, lex, parse, token, Column, Error, LineNumber, MaxValue};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Line {
    number: LineNumber,
    tokens: Vec<token::Token>,
}

impl Line {
    pub fn new(source_line: &str) -> Line {
        let (number, tokens) = lex(&source_line);
        Line { number, tokens }
    }

    pub fn number(&self) -> LineNumber {
        self.number
    }

    pub fn is_direct(&self) -> bool {
        self.number.is_none()
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn ast(&self) -> Result<Vec<Statement>, Error> {
        parse(self.number, &self.tokens)
    }

    pub fn renum(&self, changes: &HashMap<u16, u16>) -> Self {
        let number = if let Some(line_number) = self.number {
            changes.get(&line_number).cloned().or(self.number)
        } else {
            None
        };
        let ast = match parse(self.number, &self.tokens) {
            Ok(ast) => ast,
            Err(_) => {
                return Line {
                    number: self.number,
                    tokens: self.tokens.clone(),
                }
            }
        };
        let mut visitor = RenumVisitor::new(changes);
        for statement in ast {
            statement.accept(&mut visitor);
        }
        if visitor.replace.is_empty() {
            return Line {
                number,
                tokens: self.tokens.clone(),
            };
        }
        let mut s: String = self.tokens.iter().map(|s| s.to_string()).collect();
        while let Some((col, num)) = visitor.replace.pop() {
            s.replace_range(col, &format!("{}", num));
        }
        let (_, tokens) = lex(&s);
        Line { number, tokens }
    }
}

#[derive(Debug)]
struct RenumVisitor<'a> {
    changes: &'a HashMap<u16, u16>,
    replace: Vec<(Column, u16)>,
}

impl<'a> RenumVisitor<'a> {
    fn new(changes: &HashMap<u16, u16>) -> RenumVisitor {
        RenumVisitor {
            changes,
            replace: vec![],
        }
    }
    fn line(&mut self, expr: &Expression) {
        use Expression::*;
        let (col, n) = match expr {
            Single(col, n) => (col, *n as f64),
            Double(col, n) => (col, *n),
            Integer(col, n) => (col, *n as f64),
            _ => return,
        };
        if n > LineNumber::max_value() as f64 {
            return;
        }
        let n = n as u16;
        if let Some(new_num) = self.changes.get(&n) {
            self.replace.push((col.clone(), *new_num));
        }
    }
}

impl<'a> Visitor for RenumVisitor<'a> {
    fn visit_statement(&mut self, stmt: &Statement) {
        use Statement::*;
        match stmt {
            Goto(_, ln) | Gosub(_, ln) | Restore(_, ln) | Run(_, ln) => self.line(ln),
            Delete(_, ln1, ln2) | List(_, ln1, ln2) => {
                self.line(ln1);
                self.line(ln2);
            }
            OnGoto(_, _, ve) => {
                for ln in ve {
                    self.line(ln);
                }
            }
            _ => {}
        }
    }
}

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s: String = self.tokens.iter().map(|s| s.to_string()).collect();
        if let Some(number) = self.number {
            write!(f, "{} {}", number, s)
        } else {
            write!(f, "{}", s)
        }
    }
}

impl<'a> IntoIterator for &'a Line {
    type Item = &'a Line;
    type IntoIter = std::option::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        Some(self).into_iter()
    }
}
