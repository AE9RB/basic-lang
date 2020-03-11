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
                debug_assert_eq!(0, self.comp.ident.len());
                debug_assert_eq!(0, self.comp.expr.len());
            }
            Err(e) => self.prog.error(e),
        };
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
        if let Some(error) = self.comp.ident.push(ident).err() {
            self.prog.error(error)
        }
    }
    fn visit_expression(&mut self, expression: &ast::Expression) {
        let mut prog: Stack<Op> = Stack::new("COMPILED EXPRESSION TOO LARGE");
        match self.comp.expression(&mut prog, expression) {
            Ok(col) => {
                if let Some(error) = self.comp.expr.push((col.clone(), prog)).err() {
                    self.prog.error(error.in_column(&col))
                }
            }
            Err(e) => self.prog.error(e),
        };
    }
}

struct Compiler {
    ident: Stack<String>,
    expr: Stack<(Column, Stack<Op>)>,
}

impl Compiler {
    fn new() -> Compiler {
        Compiler {
            ident: Stack::new("COMPILER IDENT STACK OVERFLOW"),
            expr: Stack::new("COMPILER EXPRESSION STACK OVERFLOW"),
        }
    }

    fn expression(&mut self, prog: &mut Stack<Op>, expr: &ast::Expression) -> Result<Column> {
        fn binary_expression(this: &mut Compiler, prog: &mut Stack<Op>, op: Op) -> Result<Column> {
            let (col_rhs, mut rhs) = this.expr.pop()?;
            let (col_lhs, mut lhs) = this.expr.pop()?;
            prog.append(&mut lhs)?;
            prog.append(&mut rhs)?;
            prog.push(op)?;
            Ok(col_lhs.start..col_rhs.end)
        }
        fn op(prog: &mut Stack<Op>, col: &Column, op: Op) -> Result<Column> {
            prog.push(op)?;
            Ok(col.clone())
        }
        use ast::Expression;
        match expr {
            Expression::Single(col, val) => op(prog, col, Op::Literal(Val::Single(*val))),
            Expression::Double(col, val) => op(prog, col, Op::Literal(Val::Double(*val))),
            Expression::Integer(col, val) => op(prog, col, Op::Literal(Val::Integer(*val))),
            Expression::String(col, val) => op(prog, col, Op::Literal(Val::String(val.clone()))),
            Expression::Char(col, val) => op(prog, col, Op::Literal(Val::Char(*val))),
            Expression::Var(col, _) => {
                let ident = self.ident.pop()?;
                op(prog, col, Op::Push(ident))
            }
            Expression::Negation(col, ..) => {
                let (expr_col, mut ops) = self.expr.pop()?;
                prog.append(&mut ops)?;
                prog.push(Op::Neg)?;
                Ok(col.start..expr_col.end)
            }
            Expression::Add(..) => binary_expression(self, prog, Op::Add),
            Expression::Subtract(..) => binary_expression(self, prog, Op::Sub),
            Expression::Multiply(..) => binary_expression(self, prog, Op::Mul),
            Expression::Divide(..) => binary_expression(self, prog, Op::Div),
            _ => {
                dbg!(expr);
                Err(error!(SyntaxError; "EXPRESSION NOT YET COMPILING; PANIC"))
            }
        }
    }

    fn statement(&mut self, statement: &ast::Statement, prog: &mut Program) -> Result<Column> {
        use ast::Statement;
        match statement {
            Statement::Goto(col, ..) => self.r#goto(prog, col),
            Statement::Let(col, ..) => self.r#let(prog, col),
            Statement::List(col, ..) => self.r#list(prog, col),
            Statement::Print(col, ..) => self.r#print(prog, col),
            Statement::Run(col) => self.r#run(prog, col),
        }
    }

    fn expr_pop_line_number(&mut self) -> Result<(Column, LineNumber)> {
        let (sub_col, mut ops) = self.expr.pop()?;
        if ops.len() == 1 {
            if let Op::Literal(val) = ops.pop()? {
                return Ok((sub_col, LineNumber::try_from(val)?));
            }
        }
        Err(error!(UndefinedLine, ..&sub_col; "INVALID LINE NUMBER"))
    }

    fn r#goto(&mut self, prog: &mut Program, col: &Column) -> Result<Column> {
        let (sub_col, line_number) = self.expr_pop_line_number()?;
        let sym = prog.symbol_for_line_number(line_number)?;
        prog.link_next_op_to(&sub_col, sym);
        prog.push(Op::Jump(0))?;
        return Ok(col.start..sub_col.end);
    }

    fn r#let(&mut self, prog: &mut Program, col: &Column) -> Result<Column> {
        let (sub_col, mut ops) = self.expr.pop()?;
        prog.append(&mut ops)?;
        let ident = self.ident.pop()?;
        prog.push(Op::Pop(ident))?;
        Ok(col.start..sub_col.end)
    }

    fn r#list(&mut self, prog: &mut Program, col: &Column) -> Result<Column> {
        let (col_to, ln_to) = self.expr_pop_line_number()?;
        let (_col_from, ln_from) = self.expr_pop_line_number()?;
        prog.push(Op::Literal(Val::try_from(ln_from)?))?;
        prog.push(Op::Literal(Val::try_from(ln_to)?))?;
        prog.push(Op::List)?;
        Ok(col.start..col_to.end)
    }

    fn r#print(&mut self, prog: &mut Program, col: &Column) -> Result<Column> {
        let len = self.expr.len();
        let mut exprs = self.expr.pop_n(len)?;
        let mut col = col.clone();
        for (sub_col, mut expr) in exprs.drain(..) {
            prog.append(&mut expr)?;
            col.end = sub_col.end;
        }
        match i16::try_from(len) {
            Ok(len) => prog.push(Op::Literal(Val::Integer(len)))?,
            Err(_) => return Err(error!(Overflow, ..&col; "TOO MANY ELEMENTS")),
        };
        prog.push(Op::Print)?;
        Ok(col)
    }

    fn r#run(&mut self, prog: &mut Program, col: &Column) -> Result<Column> {
        prog.push(Op::Run)?;
        Ok(col.clone())
    }
}
