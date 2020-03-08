use super::{Op, Program, Stack, Val};
use crate::error;
use crate::lang::ast::{self, AcceptVisitor};
use crate::lang::{Column, Error, LineNumber};
use std::convert::TryFrom;

type Result<T> = std::result::Result<T, Error>;

pub fn compile(program: &mut Program, ast: &[ast::Statement]) {
    Visitor::compile(program, ast)
}

struct Visitor<'a> {
    prog: &'a mut Program,
    comp: Compiler,
}

impl<'a> Visitor<'a> {
    fn compile(program: &mut Program, ast: &[ast::Statement]) {
        let mut this = Visitor {
            prog: program,
            comp: Compiler::new(),
        };
        for statement in ast {
            statement.accept(&mut this);
        }
    }
}

impl<'a> ast::Visitor for Visitor<'a> {
    fn visit_statement(&mut self, statement: &ast::Statement) {
        match self.comp.statement(statement, self.prog) {
            Ok(_col) => {
                let ops = std::mem::take(&mut self.comp.ops);
                self.prog.append(&mut ops.into());
            }
            Err(e) => self.prog.error(e),
        };
        debug_assert_eq!(0, self.comp.ident.len());
        debug_assert_eq!(0, self.comp.expr.len());
    }
    fn visit_ident(&mut self, ident: &ast::Ident) {
        use ast::Ident;
        let ident = match ident {
            Ident::Plain(s)
            | Ident::String(s)
            | Ident::Single(s)
            | Ident::Double(s)
            | Ident::Integer(s) => s.clone(),
        };
        match self.comp.ident.push(ident) {
            Ok(_) => {}
            Err(e) => self.prog.error(e),
        };
    }
    fn visit_expression(&mut self, expression: &ast::Expression) {
        match self.comp.expression(expression) {
            Ok(col) => match self
                .comp
                .expr
                .push((col, std::mem::take(&mut self.comp.ops)))
            {
                Ok(_) => {}
                Err(e) => self.prog.error(e),
            },
            Err(e) => self.prog.error(e),
        };
    }
}

struct Compiler {
    ops: Stack<Op>,
    ident: Stack<String>,
    expr: Stack<(Column, Stack<Op>)>,
}

impl Compiler {
    fn new() -> Compiler {
        Compiler {
            ops: Stack::new(),
            ident: Stack::new(),
            expr: Stack::new(),
        }
    }
    fn expression(&mut self, expr: &ast::Expression) -> Result<Column> {
        use ast::Expression;
        match expr {
            Expression::Single(col, val) => self.operation(col, Op::Literal(Val::Single(*val))),
            Expression::Double(col, val) => self.operation(col, Op::Literal(Val::Double(*val))),
            Expression::Integer(col, val) => self.operation(col, Op::Literal(Val::Integer(*val))),
            Expression::String(col, val) => {
                self.operation(col, Op::Literal(Val::String(val.clone())))
            }
            Expression::Char(col, val) => self.operation(col, Op::Literal(Val::Char(*val))),
            Expression::Var(col, _) => {
                let ident = self.ident.pop()?;
                self.operation(col, Op::Push(ident))
            }
            Expression::Negation(col, ..) => {
                let (expr_col, mut ops) = self.expr.pop()?;
                self.ops.append(&mut ops)?;
                Ok(col.start..expr_col.end)
            }
            Expression::Add(..) => self.binary_expression(Op::Add),
            Expression::Subtract(..) => self.binary_expression(Op::Sub),
            Expression::Multiply(..) => self.binary_expression(Op::Mul),
            Expression::Divide(..) => self.binary_expression(Op::Div),
            _ => {
                dbg!(expr);
                Err(error!(SyntaxError; "EXPRESSION NOT YET COMPILING; PANIC"))
            }
        }
    }
    fn binary_expression(&mut self, op: Op) -> Result<Column> {
        let (col_rhs, mut rhs) = self.expr.pop()?;
        let (col_lhs, mut lhs) = self.expr.pop()?;
        self.ops.append(&mut rhs)?;
        self.ops.append(&mut lhs)?;
        self.ops.push(op)?;
        Ok(col_lhs.start..col_rhs.end)
    }
    fn operation(&mut self, col: &Column, op: Op) -> Result<Column> {
        self.ops.push(op)?;
        Ok(col.clone())
    }

    fn statement(&mut self, statement: &ast::Statement, prog: &mut Program) -> Result<Column> {
        use ast::Statement;
        match statement {
            Statement::Goto(col, ..) => self.r#goto(col, prog),
            Statement::Let(col, ..) => self.r#let(col),
            Statement::Print(col, ..) => self.r#print(col),
            Statement::Run(col) => self.r#run(col),
        }
    }

    fn r#goto(&mut self, col: &Column, prog: &mut Program) -> Result<Column> {
        let (sub_col, mut v) = self.expr.pop()?;
        if v.len() == 1 {
            if let Op::Literal(value) = v.pop()? {
                match LineNumber::try_from(value) {
                    Ok(line_number) => {
                        let sym = prog.symbol_for_line_number(line_number);
                        prog.link_next_op_to(&sub_col, sym);
                        prog.push(Op::Jump(0));
                        return Ok(col.start..sub_col.end);
                    }
                    Err(e) => return Err(e.in_column(&sub_col).message("INVALID LINE NUMBER")),
                }
            }
        }
        Err(error!(SyntaxError, ..&sub_col; "EXPECTED LINE NUMBER"))
    }

    fn r#let(&mut self, col: &Column) -> Result<Column> {
        let (sub_col, mut v) = self.expr.pop().unwrap();
        self.ops.append(&mut v)?;
        let ident = self.ident.pop()?;
        self.ops.push(Op::Pop(ident))?;
        Ok(col.start..sub_col.end)
    }

    fn r#print(&mut self, col: &Column) -> Result<Column> {
        let len = self.expr.len();
        let mut exprs = self.expr.pop_n(len).unwrap();
        self.ops
            .append(&mut exprs.drain(..).map(|(_c, v)| v).flatten().collect())?;
        match i16::try_from(len) {
            Ok(len) => self.ops.push(Op::Literal(Val::Integer(len)))?,
            Err(_) => return Err(error!(Overflow, ..col; "TOO MANY ELEMENTS")),
        };
        self.ops.push(Op::Print)?;
        Ok(col.clone())
    }

    fn r#run(&mut self, col: &Column) -> Result<Column> {
        self.operation(col, Op::Run)
    }
}
