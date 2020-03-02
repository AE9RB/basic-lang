use super::op::*;
use super::runtime::Address;
use super::val::*;
use crate::lang::ast::*;
use crate::lang::error::*;
use crate::lang::line::*;
use std::convert::TryFrom;

pub fn compile(line: &Line) -> Result<Vec<Op>, Vec<Error>> {
    Compiler::compile(line)
}

struct Compiler {
    program: Vec<Op>,
    error: Vec<Error>,
    ident: Vec<String>,
    expression: Vec<Vec<Op>>,
}

impl Compiler {
    fn compile(line: &Line) -> Result<Vec<Op>, Vec<Error>> {
        let ast = match line.ast() {
            Ok(ast) => ast,
            Err(e) => return Err(vec![e]),
        };
        let mut this = Compiler {
            program: vec![],
            ident: vec![],
            expression: vec![],
            error: vec![],
        };
        for statement in ast {
            statement.accept(&mut this);
        }
        if this.program.len() > Address::max_value() as usize {
            this.error(&(0..0), error!(OutOfMemory));
        }
        if this.error.len() > 0 {
            for error in &mut this.error {
                *error = error.in_line_number(line.number())
            }
            Err(std::mem::take(&mut this.error))
        } else {
            Ok(std::mem::take(&mut this.program))
        }
    }

    fn error(&mut self, col: &std::ops::Range<usize>, error: Error) {
        self.error.push(error.in_column(col));
    }

    fn append(&mut self, mut ops: &mut Vec<Op>) {
        self.program.append(&mut ops);
    }

    fn push(&mut self, op: Op) {
        self.program.push(op);
    }

    fn expression_binary_op(&mut self, op: Op) -> Vec<Op> {
        let mut rhs = self.expression.pop().unwrap();
        let mut ops = self.expression.pop().unwrap();
        ops.append(&mut rhs);
        ops.push(op);
        ops
    }
}

impl Visitor for Compiler {
    fn visit_statement(&mut self, statement: &Statement) {
        let mut ident = std::mem::take(&mut self.ident);
        let mut expression = std::mem::take(&mut self.expression);
        match statement {
            Statement::Let(..) => {
                self.append(&mut expression.pop().unwrap());
                self.push(Op::Pop(ident.pop().unwrap()));
            }
            Statement::Print(col, ..) => {
                let len = expression.len();
                let mut expr = expression.drain(..).flatten().collect::<Vec<Op>>();
                self.append(&mut expr);
                match i16::try_from(len) {
                    Ok(len) => self.push(Op::Literal(Val::Integer(len))),
                    Err(_) => self.error(col, error!(SyntaxError)),
                };
                self.push(Op::Print);
            }
        };
        debug_assert_eq!(0, ident.len());
        debug_assert_eq!(0, expression.len());
    }
    fn visit_ident(&mut self, ident: &Ident) {
        self.ident.push(match ident {
            Ident::Plain(s)
            | Ident::String(s)
            | Ident::Single(s)
            | Ident::Double(s)
            | Ident::Integer(s) => s.clone(),
        })
    }
    fn visit_expression(&mut self, expression: &Expression) {
        let ops = match expression {
            Expression::Single(_, val) => vec![Op::Literal(Val::Single(*val))],
            Expression::Double(_, val) => vec![Op::Literal(Val::Double(*val))],
            Expression::Integer(_, val) => vec![Op::Literal(Val::Integer(*val))],
            Expression::String(_, val) => vec![Op::Literal(Val::String(val.clone()))],
            Expression::Char(_, val) => vec![Op::Literal(Val::Char(*val))],

            Expression::Add(..) => self.expression_binary_op(Op::Add),
            Expression::Multiply(..) => self.expression_binary_op(Op::Mul),
            _ => unimplemented!(),
        };
        self.expression.push(ops);
    }
}
