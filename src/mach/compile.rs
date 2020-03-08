use super::{Op, Program, Val};
use crate::error;
use crate::lang::ast::{self, AcceptVisitor};
use crate::lang::{Column, LineNumber};
use std::convert::TryFrom;

pub fn compile(program: &mut Program, ast: &[ast::Statement]) {
    Compiler::compile(program, ast)
}

struct Compiler<'a> {
    prog: &'a mut Program,
    ident: Vec<String>,
    expr: Vec<(Column, Vec<Op>)>,
}

impl<'a> Compiler<'a> {
    fn compile(program: &mut Program, ast: &[ast::Statement]) {
        let mut this = Compiler {
            prog: program,
            ident: vec![],
            expr: vec![],
        };
        for statement in ast {
            statement.accept(&mut this);
        }
    }

    fn expression_binary_op(&mut self, op: Op) -> (Column, Vec<Op>) {
        let (col_rhs, mut rhs) = self.expr.pop().unwrap();
        let (col_lhs, mut ops) = self.expr.pop().unwrap();
        ops.append(&mut rhs);
        ops.push(op);
        (col_lhs.start..col_rhs.end, ops)
    }

    fn ident_pop(&mut self) -> String {
        self.ident.pop().unwrap()
    }
}

impl<'a> ast::Visitor for Compiler<'a> {
    fn visit_statement(&mut self, statement: &ast::Statement) {
        use ast::Statement;
        match statement {
            Statement::Goto(col, ..) => r#goto(col, self),
            Statement::Let(col, ..) => r#let(col, self),
            Statement::Print(col, ..) => r#print(col, self),
            Statement::Run(col) => r#run(col, self),
        };
        debug_assert_eq!(0, self.ident.len());
        debug_assert_eq!(0, self.expr.len());
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
            Expression::Var(col, _) => (col.clone(), vec![Op::Push(self.ident.pop().unwrap())]),
            Expression::Negation(col, ..) => {
                let (expr_col, mut ops) = self.expr.pop().unwrap();
                ops.push(Op::Neg);
                (col.start..expr_col.end, ops)
            }
            Expression::Add(..) => self.expression_binary_op(Op::Add),
            Expression::Subtract(..) => self.expression_binary_op(Op::Sub),
            Expression::Multiply(..) => self.expression_binary_op(Op::Mul),
            Expression::Divide(..) => self.expression_binary_op(Op::Div),
            _ => {
                dbg!(expression);
                self.prog
                    .error(error!(SyntaxError; "EXPRESSION NOT YET COMPILING; PANIC"));
                (0..0, vec![])
            }
        };
        self.expr.push(ops);
    }
}

fn r#goto(_col: &Column, comp: &mut Compiler) {
    let (sub_col, mut v) = comp.expr.pop().unwrap();
    if v.len() == 1 {
        if let Op::Literal(value) = v.pop().unwrap() {
            match LineNumber::try_from(value) {
                Ok(line_number) => {
                    let sym = comp.prog.symbol_for_line_number(line_number);
                    comp.prog.link_next_op_to(&sub_col, sym);
                    comp.prog.push(Op::Jump(0));
                }
                Err(e) => {
                    comp.prog
                        .error(e.in_column(&sub_col).message("INVALID LINE NUMBER"));
                    return;
                }
            }
        }
    }
    comp.prog
        .error(error!(SyntaxError, ..&sub_col; "EXPECTED LINE NUMBER"));
}

fn r#let(_col: &Column, comp: &mut Compiler) {
    let (_sub_col, mut v) = comp.expr.pop().unwrap();
    comp.prog.append(&mut v);
    let ident = comp.ident_pop();
    comp.prog.push(Op::Pop(ident));
}

fn r#print(col: &Column, comp: &mut Compiler) {
    let len = comp.expr.len();
    comp.prog
        .append(&mut comp.expr.drain(..).map(|(_c, v)| v).flatten().collect());
    match i16::try_from(len) {
        Ok(len) => comp.prog.push(Op::Literal(Val::Integer(len))),
        Err(_) => comp
            .prog
            .error(error!(Overflow, ..col; "TOO MANY ELEMENTS")),
    };
    comp.prog.push(Op::Print);
}

fn r#run(_col: &Column, comp: &mut Compiler) {
    comp.prog.push(Op::Run);
}
