use super::op::*;
use super::runtime::Address;
use super::val::*;
use crate::lang::ast::*;
use crate::lang::error::*;
use crate::lang::line::*;

type Result<T> = std::result::Result<T, Error>;

pub fn compile(line: &Line) -> Result<Vec<Op>> {
    Compiler::new().compile(line)
}

struct Compiler {
    program: Vec<Op>,
    statement: Vec<Vec<Op>>,
    ident: Vec<String>,
    expression: Vec<Vec<Op>>,
}

impl Compiler {
    fn new() -> Self {
        Compiler {
            program: vec![],
            statement: vec![],
            ident: vec![],
            expression: vec![],
        }
    }
    fn compile(&mut self, line: &Line) -> Result<Vec<Op>> {
        debug_assert!(self.program.len() < Address::max_value() as usize);
        for statement in line.ast()? {
            statement.accept(self);
            self.program.append(&mut self.statement.pop().unwrap());
            debug_assert_eq!(0, self.statement.len());
            debug_assert_eq!(0, self.ident.len());
            debug_assert_eq!(0, self.expression.len());
        }
        Ok(std::mem::take(&mut self.program))
    }
}

impl Visitor for Compiler {
    fn visit_statement(&mut self, statement: &Statement) {
        self.statement.push(match statement {
            Statement::Let(_, (_, _ident), _expr) => {
                let mut ops = self.expression.pop().unwrap();
                ops.push(Op::Pop(self.ident.pop().unwrap()));
                ops
            }
            Statement::Print(_, _vec_expr) => {
                let mut ops: Vec<Op> = vec![];
                self.expression.reverse();
                while let Some(mut expr) = self.expression.pop() {
                    ops.append(&mut expr)
                }
                ops.push(Op::Print);
                ops
            }
        });
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
        let e = match expression {
            Expression::Single(_, val) => vec![Op::Literal(Val::Single(*val))],
            Expression::Double(_, val) => vec![Op::Literal(Val::Double(*val))],
            Expression::Integer(_, val) => vec![Op::Literal(Val::Integer(*val))],
            Expression::String(_, val) => vec![Op::Literal(Val::String(val.clone()))],
            Expression::Char(_, val) => vec![Op::Literal(Val::Char(*val))],

            Expression::Add(_, _lhs, _rhs) => {
                let mut rhs = self.expression.pop().unwrap();
                let mut lhs = self.expression.pop().unwrap();
                let mut ops: Vec<Op> = vec![];
                ops.append(&mut lhs);
                ops.append(&mut rhs);
                ops.push(Op::Add);
                ops
            }
            Expression::Multiply(_, _lhs, _rhs) => {
                let mut rhs = self.expression.pop().unwrap();
                let mut lhs = self.expression.pop().unwrap();
                let mut ops: Vec<Op> = vec![];
                ops.append(&mut lhs);
                ops.append(&mut rhs);
                ops.push(Op::Mul);
                ops
            }
            _ => unimplemented!(),
        };
        self.expression.push(e);
    }
}
