use super::{Function, Link, Opcode, Program, Stack, Val};
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
        for (_col, frag) in this.comp.stmt.drain(..) {
            if let Some(error) = this.prog.append(frag).err() {
                this.prog.error(error);
                break;
            }
        }
        debug_assert_eq!(0, this.comp.ident.len());
        debug_assert_eq!(0, this.comp.var.len());
        debug_assert_eq!(0, this.comp.expr.len());
        debug_assert_eq!(0, this.comp.stmt.len());
    }
}

impl<'a> ast::Visitor for Visitor<'a> {
    fn visit_statement(&mut self, statement: &ast::Statement) {
        let mut prog = self.prog.new_link();
        let col = match self.comp.statement(&mut prog, statement) {
            Ok(col) => col,
            Err(e) => {
                self.prog.error(e);
                0..0
            }
        };
        if let Some(error) = self.comp.stmt.push((col.clone(), prog)).err() {
            self.prog.error(error.in_column(&col))
        }
    }
    fn visit_ident(&mut self, ident: &ast::Ident) {
        if let Some(error) = self.comp.ident.push(ident.to_string()).err() {
            self.prog.error(error)
        }
    }
    fn visit_variable(&mut self, var: &ast::Variable) {
        let mut prog = self.prog.new_link();
        let (col, name) = match self.comp.variable(&mut prog, var) {
            Ok((col, name)) => (col, name),
            Err(e) => {
                self.prog.error(e);
                (0..0, "".to_string())
            }
        };
        if let Some(error) = self.comp.var.push((col.clone(), name, prog)).err() {
            self.prog.error(error.in_column(&col))
        }
    }
    fn visit_expression(&mut self, expression: &ast::Expression) {
        let mut prog = self.prog.new_link();
        let col = match self.comp.expression(&mut prog, expression) {
            Ok(col) => col,
            Err(e) => {
                self.prog.error(e);
                0..0
            }
        };
        if let Some(error) = self.comp.expr.push((col.clone(), prog)).err() {
            self.prog.error(error.in_column(&col))
        }
    }
}

struct Compiler {
    ident: Stack<String>,
    var: Stack<(Column, String, Link)>,
    expr: Stack<(Column, Link)>,
    stmt: Stack<(Column, Link)>,
}

impl Compiler {
    fn new() -> Compiler {
        Compiler {
            ident: Stack::new("COMPILER IDENT OVERFLOW"),
            var: Stack::new("COMPILER VARIABLE OVERFLOW"),
            expr: Stack::new("COMPILER EXPRESSION OVERFLOW"),
            stmt: Stack::new("COMPILER STATEMENT OVERFLOW"),
        }
    }

    fn variable(&mut self, prog: &mut Link, var: &ast::Variable) -> Result<(Column, String)> {
        use ast::Variable;
        let col = match var {
            Variable::Unary(col, _ident) => col,
            Variable::Array(col, _ident, vec_expr) => {
                let len = vec_expr.len();
                let vec_expr = self.expr.pop_n(len)?;
                for (_col, ops) in vec_expr {
                    prog.append(ops)?
                }
                let len_opcode = self.val_int_from_usize(len, col)?;
                prog.push(len_opcode)?;
                col
            }
        };
        Ok((col.clone(), self.ident.pop()?))
    }

    fn expression(&mut self, prog: &mut Link, expr: &ast::Expression) -> Result<Column> {
        fn binary_expression(this: &mut Compiler, prog: &mut Link, op: Opcode) -> Result<Column> {
            let (col_rhs, rhs) = this.expr.pop()?;
            let (col_lhs, lhs) = this.expr.pop()?;
            prog.append(lhs)?;
            prog.append(rhs)?;
            prog.push(op)?;
            Ok(col_lhs.start..col_rhs.end)
        }
        fn literal(prog: &mut Link, col: &Column, val: Val) -> Result<Column> {
            prog.push(Opcode::Literal(val))?;
            Ok(col.clone())
        }
        fn function(
            this: &mut Compiler,
            prog: &mut Link,
            col: &Column,
            len: usize,
        ) -> Result<Column> {
            let mut args_col: std::ops::Range<usize> = 0..0;
            for (col, opcodes) in this.expr.drain(this.expr.len() - len..) {
                if args_col.start == 0 {
                    args_col.start = col.start
                }
                args_col.end = col.end;
                prog.append(opcodes)?;
            }
            let ident = this.ident.pop()?;
            if let Some((opcode, arity)) = Function::opcode_and_arity(&ident) {
                if arity.contains(&len) {
                    if arity.start() != arity.end() {
                        prog.push(this.val_int_from_usize(len, &col)?)?;
                    }
                    prog.push(opcode)?;
                    return Ok(args_col);
                }
                return Err(error!(SyntaxError, ..&args_col; "WRONG NUMBER OF ARGUMENTS"));
            }
            if ident.starts_with("FN") {
                return Err(error!(UndefinedUserFunction, ..col));
            }
            prog.push(this.val_int_from_usize(len, &col)?)?;
            prog.push(Opcode::PushArr(ident))?;
            Ok(col.start..args_col.end)
        }
        use ast::Expression;
        match expr {
            Expression::Single(col, val) => literal(prog, col, Val::Single(*val)),
            Expression::Double(col, val) => literal(prog, col, Val::Double(*val)),
            Expression::Integer(col, val) => literal(prog, col, Val::Integer(*val)),
            Expression::String(col, val) => literal(prog, col, Val::String(val.clone())),
            Expression::UnaryVar(col, _) => {
                let ident = self.ident.pop()?;
                prog.push(Opcode::Push(ident))?;
                Ok(col.clone())
            }
            Expression::Function(col, _, vec_exp) => function(self, prog, col, vec_exp.len()),
            Expression::Negation(col, ..) => {
                let (expr_col, ops) = self.expr.pop()?;
                prog.append(ops)?;
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

    fn statement(&mut self, prog: &mut Link, statement: &ast::Statement) -> Result<Column> {
        use ast::Statement;
        match statement {
            Statement::Clear(col, ..) => self.r#clear(prog, col),
            Statement::Cont(col, ..) => self.r#cont(prog, col),
            Statement::Dim(col, ..) => self.r#dim(prog, col),
            Statement::End(col, ..) => self.r#end(prog, col),
            Statement::For(col, ..) => self.r#for(prog, col),
            Statement::Goto(col, ..) => self.r#goto(prog, col),
            Statement::If(col, _, th, el) => self.r#if(prog, col, th.len(), el.len()),
            Statement::Input(col, ..) => self.r#input(prog, col),
            Statement::Let(col, ..) => self.r#let(prog, col),
            Statement::List(col, ..) => self.r#list(prog, col),
            Statement::Next(col, ..) => self.r#next(prog, col),
            Statement::Print(col, ..) => self.r#print(prog, col),
            Statement::Run(col, ..) => self.r#run(prog, col),
            Statement::Stop(col, ..) => self.r#stop(prog, col),
        }
    }

    fn val_int_from_usize(&self, num: usize, col: &Column) -> Result<Opcode> {
        match i16::try_from(num) {
            Ok(len) => Ok(Opcode::Literal(Val::Integer(len))),
            Err(_) => Err(error!(Overflow, ..col; "TOO MANY ELEMENTS")),
        }
    }

    fn expr_pop_line_number(&mut self) -> Result<(Column, LineNumber)> {
        let (sub_col, ops) = self.expr.pop()?;
        match LineNumber::try_from(ops) {
            Ok(ln) => Ok((sub_col, ln)),
            Err(e) => Err(e),
        }
    }

    fn r#clear(&mut self, prog: &mut Link, col: &Column) -> Result<Column> {
        prog.push(Opcode::Clear)?;
        Ok(col.clone())
    }

    fn r#cont(&mut self, prog: &mut Link, col: &Column) -> Result<Column> {
        prog.push(Opcode::Cont)?;
        Ok(col.clone())
    }

    fn r#dim(&mut self, prog: &mut Link, col: &Column) -> Result<Column> {
        let (var_col, name, ops) = self.var.pop()?;
        if ops.is_empty() {
            return Err(error!(SyntaxError, ..&var_col; "NOT AN ARRAY"));
        } else {
            prog.append(ops)?;
            prog.push(Opcode::DimArr(name))?;
        }
        Ok(col.clone())
    }

    fn r#end(&mut self, prog: &mut Link, col: &Column) -> Result<Column> {
        prog.push(Opcode::End)?;
        Ok(col.clone())
    }

    fn r#for(&mut self, prog: &mut Link, col: &Column) -> Result<Column> {
        let (step_col, step_ops) = self.expr.pop()?;
        let (_to_col, to_ops) = self.expr.pop()?;
        let (_from_col, from_ops) = self.expr.pop()?;
        let var_name = self.ident.pop()?;
        prog.append(step_ops)?;
        prog.append(to_ops)?;
        prog.append(from_ops)?;
        prog.push(Opcode::Pop(var_name.clone()))?;
        prog.push(Opcode::Literal(Val::String(var_name.clone())))?;
        prog.push(Opcode::Literal(Val::Integer(0)))?;
        prog.push_for(col.start..step_col.end, var_name)?;
        Ok(col.start..step_col.end)
    }

    fn r#goto(&mut self, prog: &mut Link, col: &Column) -> Result<Column> {
        let (sub_col, line_number) = self.expr_pop_line_number()?;
        let full_col = col.start..sub_col.end;
        prog.push_goto(sub_col, line_number)?;
        Ok(full_col)
    }

    fn r#if(
        &mut self,
        prog: &mut Link,
        col: &Column,
        then_len: usize,
        else_len: usize,
    ) -> Result<Column> {
        let (_predicate_col, predicate) = self.expr.pop()?;
        prog.append(predicate)?;
        let else_sym = prog.next_symbol();
        prog.link_addr_to_symbol(prog.len(), col.clone(), else_sym);
        prog.push(Opcode::IfNot(0))?;
        let elses = self.stmt.pop_n(else_len)?;
        for (_col, link) in self.stmt.pop_n(then_len)? {
            prog.append(link)?;
        }
        if else_len == 0 {
            prog.insert(else_sym, prog.len());
        } else {
            let finished_sym = prog.next_symbol();
            prog.link_addr_to_symbol(prog.len(), col.clone(), finished_sym);
            prog.push(Opcode::Jump(0))?;
            prog.insert(else_sym, prog.len());
            for (_col, link) in elses {
                prog.append(link)?;
            }
            prog.insert(finished_sym, prog.len());
        }
        Ok(0..0)
    }

    fn r#input(&mut self, prog: &mut Link, col: &Column) -> Result<Column> {
        let len = self.val_int_from_usize(self.var.len(), col)?;
        let (_prompt_col, prompt) = self.expr.pop()?;
        let (_caps_col, caps) = self.expr.pop()?;
        prog.append(prompt)?;
        prog.append(caps)?;
        prog.push(len)?;
        for (_var_col, var_name, var_ops) in self.var.drain(..) {
            prog.push(Opcode::Input(var_name.clone()))?;
            if var_ops.is_empty() {
                prog.push(Opcode::Pop(var_name))?
            } else {
                prog.append(var_ops)?;
                prog.push(Opcode::PopArr(var_name))?
            }
        }
        prog.push(Opcode::Input("".to_string()))?;
        Ok(col.clone())
    }

    fn r#let(&mut self, prog: &mut Link, col: &Column) -> Result<Column> {
        let (expr_col, expr_ops) = self.expr.pop()?;
        prog.append(expr_ops)?;
        let (_var_col, var_name, var_ops) = self.var.pop()?;
        if var_ops.is_empty() {
            prog.push(Opcode::Pop(var_name))?
        } else {
            prog.append(var_ops)?;
            prog.push(Opcode::PopArr(var_name))?
        }
        Ok(col.start..expr_col.end)
    }

    fn r#list(&mut self, prog: &mut Link, col: &Column) -> Result<Column> {
        let (col_to, ln_to) = self.expr_pop_line_number()?;
        let (_col_from, ln_from) = self.expr_pop_line_number()?;
        prog.push(Opcode::Literal(Val::try_from(ln_from)?))?;
        prog.push(Opcode::Literal(Val::try_from(ln_to)?))?;
        prog.push(Opcode::List)?;
        Ok(col.start..col_to.end)
    }

    fn r#next(&mut self, prog: &mut Link, col: &Column) -> Result<Column> {
        let ident = self.ident.pop()?;
        prog.push_next(col.clone(), ident)?;
        Ok(col.clone())
    }

    fn r#print(&mut self, prog: &mut Link, col: &Column) -> Result<Column> {
        let (_expr_col, expr) = self.expr.pop()?;
        prog.append(expr)?;
        prog.push(Opcode::Print)?;
        Ok(col.clone())
    }

    fn r#run(&mut self, prog: &mut Link, col: &Column) -> Result<Column> {
        let mut line_number: LineNumber = None;
        let (sub_col, ops) = self.expr.pop()?;
        if let Ok(ln) = LineNumber::try_from(ops) {
            line_number = ln;
        }
        let full_col = col.start..sub_col.end;
        prog.push_run(sub_col, line_number)?;
        Ok(full_col)
    }

    fn r#stop(&mut self, prog: &mut Link, col: &Column) -> Result<Column> {
        prog.push(Opcode::Stop)?;
        Ok(col.clone())
    }
}
