use super::{Op, Program, Val};
use crate::error;
use crate::lang::ast::{self, AcceptVisitor};
use crate::lang::{Column, LineNumber};
use std::convert::TryFrom;

pub fn compile(program: &mut Program, ast: &[ast::Statement]) {
    Compiler::compile(program, ast)
}

type PIE<'a> = (&'a mut Program, &'a mut Vec<String>, &'a mut Vec<Vec<Op>>);

struct Compiler<'a> {
    program: &'a mut Program,
    ident: Vec<String>,
    expression: Vec<Vec<Op>>,
}

impl<'a> Compiler<'a> {
    fn compile(program: &mut Program, ast: &[ast::Statement]) {
        let mut this = Compiler {
            program: program,
            ident: vec![],
            expression: vec![],
        };
        for statement in ast {
            statement.accept(&mut this);
        }
    }

    fn expression_binary_op(&mut self, op: Op) -> Vec<Op> {
        let mut rhs = self.expression.pop().unwrap();
        let mut ops = self.expression.pop().unwrap();
        ops.append(&mut rhs);
        ops.push(op);
        ops
    }
}

impl<'a> ast::Visitor for Compiler<'a> {
    fn visit_statement(&mut self, statement: &ast::Statement) {
        use ast::Statement;
        let pie = (&mut *self.program, &mut self.ident, &mut self.expression);
        match statement {
            Statement::Goto(col, ..) => r#goto(pie, col),
            Statement::Let(..) => r#let(pie),
            Statement::Print(col, ..) => r#print(pie, col),
        };
        debug_assert_eq!(0, self.ident.len());
        debug_assert_eq!(0, self.expression.len());
    }
    fn visit_ident(&mut self, ident: &ast::Ident) {
        use ast::Ident;
        self.ident.push(match ident {
            Ident::Plain(s)
            | Ident::String(s)
            | Ident::Single(s)
            | Ident::Double(s)
            | Ident::Integer(s) => s.clone(),
        })
    }
    fn visit_expression(&mut self, expression: &ast::Expression) {
        use ast::Expression;
        let ops = match expression {
            Expression::Single(_, val) => vec![Op::Literal(Val::Single(*val))],
            Expression::Double(_, val) => vec![Op::Literal(Val::Double(*val))],
            Expression::Integer(_, val) => vec![Op::Literal(Val::Integer(*val))],
            Expression::String(_, val) => vec![Op::Literal(Val::String(val.clone()))],
            Expression::Char(_, val) => vec![Op::Literal(Val::Char(*val))],
            Expression::Var(_, _) => vec![Op::Push(self.ident.pop().unwrap().to_string())],

            Expression::Add(..) => self.expression_binary_op(Op::Add),
            Expression::Subtract(..) => self.expression_binary_op(Op::Add),
            Expression::Multiply(..) => self.expression_binary_op(Op::Mul),
            Expression::Divide(..) => self.expression_binary_op(Op::Mul),
            _ => unimplemented!(),
        };
        self.expression.push(ops);
    }
}

fn r#goto((program, _ident, expression): PIE, col: &Column) {
    let mut v = expression.pop().unwrap();
    let mut line_number = Some(u16::max_value());
    loop {
        if v.len() == 1 {
            if let Op::Literal(value) = v.pop().unwrap() {
                match LineNumber::try_from(value) {
                    Ok(n) => line_number = n,
                    Err(e) => program.error(col, e),
                }
                break;
            }
        }
        program.error(col, error!(SyntaxError));
        break;
    }
    let sym = program.symbol_for_line_number(line_number);
    program.link_next_op_to(sym);
    program.push(Op::Jump(0));
}

fn r#let((program, ident, expression): PIE) {
    program.append(&mut expression.pop().unwrap());
    program.push(Op::Pop(ident.pop().unwrap()));
}

fn r#print((program, _ident, expression): PIE, col: &Column) {
    let len = expression.len();
    program.append(&mut expression.drain(..).flatten().collect());
    match i16::try_from(len) {
        Ok(len) => program.push(Op::Literal(Val::Integer(len))),
        Err(_) => program.error(col, error!(SyntaxError)),
    };
    program.push(Op::Print);
}
