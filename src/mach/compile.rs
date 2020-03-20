use super::{Function, Opcode, Program, Stack, Val};
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
            Ident::Plain(col, s)
            | Ident::String(col, s)
            | Ident::Single(col, s)
            | Ident::Double(col, s)
            | Ident::Integer(col, s) => (col.clone(), s.clone()),
        };
        if let Some(error) = self.comp.ident.push(ident).err() {
            self.prog.error(error)
        }
    }
    fn visit_expression(&mut self, expression: &ast::Expression) {
        let mut prog: Stack<Opcode> = Stack::new("COMPILED EXPRESSION TOO LARGE");
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
    ident: Stack<(Column, String)>,
    expr: Stack<(Column, Stack<Opcode>)>,
}

impl Compiler {
    fn new() -> Compiler {
        Compiler {
            ident: Stack::new("COMPILER IDENT OVERFLOW"),
            expr: Stack::new("COMPILER EXPRESSION OVERFLOW"),
        }
    }

    fn expression(&mut self, prog: &mut Stack<Opcode>, expr: &ast::Expression) -> Result<Column> {
        fn binary_expression(
            this: &mut Compiler,
            prog: &mut Stack<Opcode>,
            op: Opcode,
        ) -> Result<Column> {
            let (col_rhs, mut rhs) = this.expr.pop()?;
            let (col_lhs, mut lhs) = this.expr.pop()?;
            prog.append(&mut lhs)?;
            prog.append(&mut rhs)?;
            prog.push(op)?;
            Ok(col_lhs.start..col_rhs.end)
        }
        fn literal(prog: &mut Stack<Opcode>, col: &Column, val: Val) -> Result<Column> {
            prog.push(Opcode::Literal(val))?;
            Ok(col.clone())
        }
        fn function(this: &mut Compiler, prog: &mut Stack<Opcode>, col: &Column) -> Result<Column> {
            let mut args_col: std::ops::Range<usize> = 0..0;
            let len = this.expr.len();
            for (col, mut opcodes) in this.expr.drain(..) {
                if args_col.start == 0 {
                    args_col.start = col.start
                }
                args_col.end = col.end;
                prog.append(&mut opcodes)?;
            }
            let (_, ident) = this.ident.pop()?;
            if let Some((opcode, arity)) = Function::opcode_and_arity(&ident) {
                if arity.contains(&len) {
                    prog.push(opcode)?;
                    return Ok(args_col);
                }
                return Err(error!(SyntaxError, ..&args_col; "WRONG NUMBER OF ARGUMENTS"));
            }
            Err(error!(InternalError, ..col; "NOT YET COMPILING; PANIC"))
        }
        use ast::Expression;
        match expr {
            Expression::Single(col, val) => literal(prog, col, Val::Single(*val)),
            Expression::Double(col, val) => literal(prog, col, Val::Double(*val)),
            Expression::Integer(col, val) => literal(prog, col, Val::Integer(*val)),
            Expression::String(col, val) => literal(prog, col, Val::String(val.clone())),
            Expression::Char(col, val) => literal(prog, col, Val::Char(*val)),
            Expression::Var(col, _) => {
                let (_, ident) = self.ident.pop()?;
                prog.push(Opcode::Push(ident))?;
                Ok(col.clone())
            }
            Expression::Function(col, _, _) => function(self, prog, col),
            Expression::Negation(col, ..) => {
                let (expr_col, mut ops) = self.expr.pop()?;
                prog.append(&mut ops)?;
                prog.push(Opcode::Neg)?;
                Ok(col.start..expr_col.end)
            }
            Expression::Exponentiation(..) => binary_expression(self, prog, Opcode::Exp),
            Expression::Multiply(..) => binary_expression(self, prog, Opcode::Mul),
            Expression::Divide(..) => binary_expression(self, prog, Opcode::Div),
            Expression::DivideInt(..) => binary_expression(self, prog, Opcode::DivInt),
            Expression::Modulus(..) => binary_expression(self, prog, Opcode::Mod),
            Expression::Add(..) => binary_expression(self, prog, Opcode::Add),
            Expression::Subtract(..) => binary_expression(self, prog, Opcode::Sub),
            Expression::Equal(..) => binary_expression(self, prog, Opcode::Eq),
            Expression::NotEqual(..) => binary_expression(self, prog, Opcode::NotEq),
            Expression::Less(..) => binary_expression(self, prog, Opcode::Lt),
            Expression::LessEqual(..) => binary_expression(self, prog, Opcode::LtEq),
            Expression::Greater(..) => binary_expression(self, prog, Opcode::Gt),
            Expression::GreaterEqual(..) => binary_expression(self, prog, Opcode::GtEq),
            Expression::Not(..) => binary_expression(self, prog, Opcode::Not),
            Expression::And(..) => binary_expression(self, prog, Opcode::And),
            Expression::Or(..) => binary_expression(self, prog, Opcode::Or),
            Expression::Xor(..) => binary_expression(self, prog, Opcode::Xor),
            Expression::Imp(..) => binary_expression(self, prog, Opcode::Imp),
            Expression::Eqv(..) => binary_expression(self, prog, Opcode::Eqv),
        }
    }

    fn statement(&mut self, statement: &ast::Statement, prog: &mut Program) -> Result<Column> {
        use ast::Statement;
        match statement {
            Statement::Clear(col, ..) => self.r#clear(prog, col),
            Statement::Cont(col, ..) => self.r#cont(prog, col),
            Statement::End(col, ..) => self.r#end(prog, col),
            Statement::For(col, ..) => self.r#for(prog, col),
            Statement::Goto(col, ..) => self.r#goto(prog, col),
            Statement::Input(col, ..) => self.r#input(prog, col),
            Statement::Let(col, ..) => self.r#let(prog, col),
            Statement::List(col, ..) => self.r#list(prog, col),
            Statement::Next(col, ..) => self.r#next(prog, col),
            Statement::Print(col, ..) => self.r#print(prog, col),
            Statement::Run(col, ..) => self.r#run(prog, col),
            Statement::Stop(col, ..) => self.r#stop(prog, col),
        }
    }

    fn expr_pop_line_number(&mut self) -> Result<(Column, LineNumber)> {
        let (sub_col, mut ops) = self.expr.pop()?;
        if ops.len() == 1 {
            if let Opcode::Literal(val) = ops.pop()? {
                return Ok((sub_col, LineNumber::try_from(val)?));
            }
        }
        Err(error!(UndefinedLine, ..&sub_col; "INVALID LINE NUMBER"))
    }

    fn r#clear(&mut self, prog: &mut Program, col: &Column) -> Result<Column> {
        prog.push(Opcode::Clear)?;
        Ok(col.clone())
    }

    fn r#cont(&mut self, prog: &mut Program, col: &Column) -> Result<Column> {
        prog.push(Opcode::Cont)?;
        Ok(col.clone())
    }

    fn r#end(&mut self, prog: &mut Program, col: &Column) -> Result<Column> {
        prog.push(Opcode::End)?;
        Ok(col.clone())
    }

    fn r#for(&mut self, prog: &mut Program, col: &Column) -> Result<Column> {
        let (step_col, mut step_ops) = self.expr.pop()?;
        let (_to_col, mut to_ops) = self.expr.pop()?;
        let (_from_col, mut from_ops) = self.expr.pop()?;
        let (_ident_col, var_name) = self.ident.pop()?;
        prog.append(&mut step_ops)?;
        prog.append(&mut to_ops)?;
        prog.append(&mut from_ops)?;
        prog.push(Opcode::Pop(var_name.clone()))?;
        prog.push(Opcode::Literal(Val::String(var_name.clone())))?;
        prog.push(Opcode::Literal(Val::Integer(0)))?;
        prog.push_for(col.start..step_col.end, var_name)?;
        Ok(col.start..step_col.end)
    }

    fn r#goto(&mut self, prog: &mut Program, col: &Column) -> Result<Column> {
        let (sub_col, line_number) = self.expr_pop_line_number()?;
        let full_col = col.start..sub_col.end;
        prog.push_goto(sub_col, line_number)?;
        Ok(full_col)
    }

    fn r#input(&mut self, prog: &mut Program, col: &Column) -> Result<Column> {
        let mut full_col = col.clone();
        let len = self.ident.len();
        for (col, var_name) in self.ident.drain(..) {
            full_col.end = col.end;
            prog.push(Opcode::Literal(Val::String(var_name)))?;
        }
        match i16::try_from(len) {
            Ok(len) => prog.push(Opcode::Literal(Val::Integer(len)))?,
            Err(_) => return Err(error!(Overflow, ..&col; "TOO MANY VARIABLES")),
        };
        let (_prompt_col, mut prompt) = self.expr.pop()?;
        let (_caps_col, mut caps) = self.expr.pop()?;
        prog.append(&mut prompt)?;
        prog.append(&mut caps)?;
        prog.push(Opcode::Input)?;
        Ok(full_col)
    }

    fn r#let(&mut self, prog: &mut Program, col: &Column) -> Result<Column> {
        let (sub_col, mut ops) = self.expr.pop()?;
        prog.append(&mut ops)?;
        let (_col, ident) = self.ident.pop()?;
        prog.push(Opcode::Pop(ident))?;
        Ok(col.start..sub_col.end)
    }

    fn r#list(&mut self, prog: &mut Program, col: &Column) -> Result<Column> {
        let (col_to, ln_to) = self.expr_pop_line_number()?;
        let (_col_from, ln_from) = self.expr_pop_line_number()?;
        prog.push(Opcode::Literal(Val::try_from(ln_from)?))?;
        prog.push(Opcode::Literal(Val::try_from(ln_to)?))?;
        prog.push(Opcode::List)?;
        Ok(col.start..col_to.end)
    }

    fn r#next(&mut self, prog: &mut Program, col: &Column) -> Result<Column> {
        let mut full_col = col.clone();
        if self.ident.is_empty() {
            prog.push_next(col.clone(), "".to_string())?;
        } else {
            for (col, var_name) in self.ident.drain(..) {
                full_col.end = col.end;
                prog.push_next(col, var_name)?;
            }
        }
        Ok(full_col)
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
            Ok(len) => prog.push(Opcode::Literal(Val::Integer(len)))?,
            Err(_) => return Err(error!(Overflow, ..&col; "TOO MANY ELEMENTS")),
        };
        prog.push(Opcode::Print)?;
        Ok(col)
    }

    fn r#run(&mut self, prog: &mut Program, col: &Column) -> Result<Column> {
        let mut line_number: LineNumber = None;
        let (sub_col, mut ops) = self.expr.pop()?;
        if ops.len() == 1 {
            if let Opcode::Literal(val) = ops.pop()? {
                if let Ok(ln) = LineNumber::try_from(val) {
                    line_number = ln;
                }
            }
        }
        let full_col = col.start..sub_col.end;
        prog.push_run(sub_col, line_number)?;
        Ok(full_col)
    }

    fn r#stop(&mut self, prog: &mut Program, col: &Column) -> Result<Column> {
        prog.push(Opcode::Stop)?;
        Ok(col.clone())
    }
}
