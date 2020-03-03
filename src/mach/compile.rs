use super::op::*;
use super::program::*;
use super::val::*;
use crate::lang::ast;
use crate::lang::ast::AcceptVisitor;
use crate::lang::Error;
use crate::lang::Line;
use std::convert::TryFrom;

pub fn compile(program: &mut Program, line: &Line) {
    Compiler::compile(program, line)
}

struct Compiler<'a> {
    program: &'a mut Program,
    line: &'a Line,
    ident: Vec<String>,
    expression: Vec<Vec<Op>>,
}

impl<'a> Compiler<'a> {
    fn compile(program: &mut Program, line: &Line) {
        let ast = match line.ast() {
            Ok(ast) => ast,
            Err(e) => {
                program.error_push(e);
                return;
            }
        };
        let mut this = Compiler {
            program: program,
            line: line,
            ident: vec![],
            expression: vec![],
        };
        for statement in ast {
            statement.accept(&mut this);
        }
        if this.program.len() > Address::max_value() as usize {
            this.error(&(0..0), error!(OutOfMemory));
        }
    }

    fn error(&mut self, col: &std::ops::Range<usize>, error: Error) {
        self.program
            .error_push(error.in_column(col).in_line_number(self.line.number()));
    }

    fn append(&mut self, mut ops: &mut Vec<Op>) {
        self.program.append(&mut ops);
    }

    fn push(&mut self, op: Op) {
        self.program.push(op);
    }

    pub fn symbol_for_line_number(&mut self, line_number: u16) -> Symbol {
        self.program.symbol_for_line_number(line_number)
    }

    pub fn symbol_here(&mut self) -> Symbol {
        self.program.symbol_here()
    }

    pub fn link_next_op_to(&mut self, symbol: Symbol) {
        self.program.link_next_op_to(symbol)
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
        let mut ident = self.ident.split_off(0);
        let mut expression = self.expression.split_off(0);
        match statement {
            Statement::Goto(col, ..) => {
                let mut v = expression.pop().unwrap();
                let mut line_number = u16::max_value();
                loop {
                    if v.len() == 1 {
                        if let Op::Literal(value) = v.pop().unwrap() {
                            match u16::try_from(value) {
                                Ok(n) => line_number = n,
                                Err(e) => self.error(col, e),
                            }
                            break;
                        }
                    }
                    self.error(col, error!(SyntaxError));
                    break;
                }
                let sym = self.symbol_for_line_number(line_number);
                self.link_next_op_to(sym);
                self.push(Op::Goto(0));
            }
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

            Expression::Add(..) => self.expression_binary_op(Op::Add),
            Expression::Multiply(..) => self.expression_binary_op(Op::Mul),
            _ => unimplemented!(),
        };
        self.expression.push(ops);
    }
}
