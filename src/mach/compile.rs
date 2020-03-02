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
    statement: Vec<Vec<Op>>,
    ident: Vec<String>,
    expression: Vec<Vec<Op>>,
    error: Vec<Error>,
}

impl Compiler {
    fn compile(line: &Line) -> Result<Vec<Op>, Vec<Error>> {
        let ast = match line.ast() {
            Ok(ast) => ast,
            Err(e) => return Err(vec![e]),
        };
        let mut this = Compiler {
            program: vec![],
            statement: vec![],
            ident: vec![],
            expression: vec![],
            error: vec![],
        };
        for statement in ast {
            statement.accept(&mut this);
            this.program.append(&mut this.statement.pop().unwrap());
            debug_assert_eq!(0, this.statement.len());
            debug_assert_eq!(0, this.ident.len());
            debug_assert_eq!(0, this.expression.len());
        }
        if this.program.len() > Address::max_value() as usize {
            this.error(error!(OutOfMemory));
        }
        if this.error.len() > 0 {
            return Err(std::mem::take(&mut this.error));
        }
        Ok(std::mem::take(&mut this.program))
    }

    fn error(&mut self, error: Error) {
        //todo col and line number
        self.error.push(error);
    }

    fn expression_flat(&mut self) -> (usize, Vec<Op>) {
        (
            self.expression.len(),
            self.expression.drain(..).flatten().collect(),
        )
    }

    fn expression_pop(&mut self) -> Vec<Op> {
        self.expression.pop().unwrap()
    }

    fn expression_binary_op(&mut self, op: Op) -> Vec<Op> {
        let mut rhs = self.expression_pop();
        let mut ops = self.expression_pop();
        ops.append(&mut rhs);
        ops.push(op);
        ops
    }
}

impl Visitor for Compiler {
    fn visit_statement(&mut self, statement: &Statement) {
        let ops = match statement {
            Statement::Let(_, (_, _ident), _expr) => {
                let mut ops = self.expression_pop();
                ops.push(Op::Pop(self.ident.pop().unwrap()));
                ops
            }
            Statement::Print(_, _vec_expr) => {
                let (len, mut ops) = self.expression_flat();
                match i16::try_from(len) {
                    Ok(len) => ops.push(Op::Literal(Val::Integer(len))),
                    Err(_) => self.error(error!(SyntaxError)),
                };
                ops.push(Op::Print);
                ops
            }
        };
        self.statement.push(ops);
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

            Expression::Add(_, _lhs, _rhs) => self.expression_binary_op(Op::Add),
            Expression::Multiply(_, _lhs, _rhs) => self.expression_binary_op(Op::Mul),
            _ => unimplemented!(),
        };
        self.expression.push(ops);
    }
}
