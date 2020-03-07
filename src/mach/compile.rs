use super::{Op, Program, Val};
use crate::error;
use crate::lang::ast::{self, AcceptVisitor};
use crate::lang::{Column, LineNumber};
use std::convert::TryFrom;

pub fn compile(program: &mut Program, ast: &[ast::Statement]) {
    Compiler::compile(program, ast)
}

type PIE<'a> = (
    &'a mut Program,
    &'a mut Vec<String>,
    &'a mut Vec<(Column, Vec<Op>)>,
);

struct Compiler<'a> {
    program: &'a mut Program,
    ident: Vec<String>,
    expression: Vec<(Column, Vec<Op>)>,
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

    fn expression_binary_op(&mut self, op: Op) -> (Column, Vec<Op>) {
        let (col_rhs, mut rhs) = self.expression.pop().unwrap();
        let (col_lhs, mut ops) = self.expression.pop().unwrap();
        ops.append(&mut rhs);
        ops.push(op);
        (col_lhs.start..col_rhs.end, ops)
    }
}

impl<'a> ast::Visitor for Compiler<'a> {
    fn visit_statement(&mut self, statement: &ast::Statement) {
        use ast::Statement;
        let pie = (&mut *self.program, &mut self.ident, &mut self.expression);
        match statement {
            Statement::Goto(col, ..) => r#goto(pie, col),
            Statement::Let(col, ..) => r#let(pie, col),
            Statement::Print(col, ..) => r#print(pie, col),
            Statement::Run(col) => r#run(pie, col),
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
            Expression::Single(col, val) => (col.clone(), vec![Op::Literal(Val::Single(*val))]),
            Expression::Double(col, val) => (col.clone(), vec![Op::Literal(Val::Double(*val))]),
            Expression::Integer(col, val) => (col.clone(), vec![Op::Literal(Val::Integer(*val))]),
            Expression::String(col, val) => {
                (col.clone(), vec![Op::Literal(Val::String(val.clone()))])
            }
            Expression::Char(col, val) => (col.clone(), vec![Op::Literal(Val::Char(*val))]),
            Expression::Var(col, _) => (
                col.clone(),
                vec![Op::Push(self.ident.pop().unwrap().to_string())],
            ),

            Expression::Add(..) => self.expression_binary_op(Op::Add),
            Expression::Subtract(..) => self.expression_binary_op(Op::Add),
            Expression::Multiply(..) => self.expression_binary_op(Op::Mul),
            Expression::Divide(..) => self.expression_binary_op(Op::Mul),
            _ => unimplemented!(),
        };
        self.expression.push(ops);
    }
}

fn r#goto((program, _ident, expression): PIE, _col: &Column) {
    let (sub_col, mut v) = expression.pop().unwrap();
    if v.len() == 1 {
        if let Op::Literal(value) = v.pop().unwrap() {
            match LineNumber::try_from(value) {
                Ok(line_number) => {
                    let sym = program.symbol_for_line_number(line_number);
                    program.link_next_op_to(&sub_col, sym);
                    program.push(Op::Jump(0));
                }
                Err(e) => {
                    program.error(e.in_column(&sub_col).message("INVALID LINE NUMBER"));
                    return;
                }
            }
        }
    }
    program.error(error!(SyntaxError, ..&sub_col; "EXPECTED LINE NUMBER"));
}

fn r#let((program, ident, expression): PIE, _col: &Column) {
    let (_sub_col, mut v) = expression.pop().unwrap();
    program.append(&mut v);
    program.push(Op::Pop(ident.pop().unwrap()));
}

fn r#print((program, _ident, expression): PIE, col: &Column) {
    let len = expression.len();
    program.append(&mut expression.drain(..).map(|(_c, v)| v).flatten().collect());
    match i16::try_from(len) {
        Ok(len) => program.push(Op::Literal(Val::Integer(len))),
        Err(_) => program.error(error!(SyntaxError, ..col; "TOO MANY ELEMENTS")),
    };
    program.push(Op::Print);
}

fn r#run((program, _ident, _expression): PIE, _col: &Column) {
    program.push(Op::Run);
}
