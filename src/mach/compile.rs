use super::{Function, Link, Opcode, Program, Stack, Val};
use crate::error;
use crate::lang::ast::{self, AcceptVisitor};
use crate::lang::{Column, Error, LineNumber};
use std::convert::TryFrom;
use std::rc::Rc;

type Result<T> = std::result::Result<T, Error>;

pub fn compile(program: &mut Program, ast: &[ast::Statement]) {
    Visitor::compile(program, ast)
}

struct Visitor<'a> {
    link: &'a mut Program,
    comp: Compiler,
}

impl<'a> Visitor<'a> {
    fn compile(program: &mut Program, ast: &[ast::Statement]) {
        let mut this = Visitor {
            link: program,
            comp: Compiler::new(),
        };
        for statement in ast {
            statement.accept(&mut this);
        }
        for (_col, frag) in this.comp.stmt.drain(..) {
            if let Some(error) = this.link.append(frag).err() {
                this.link.error(error);
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
        let mut link = Link::default();
        let col = match self.comp.statement(&mut link, statement) {
            Ok(col) => col,
            Err(e) => {
                self.link.error(e);
                0..0
            }
        };
        if let Some(error) = self.comp.stmt.push((col.clone(), link)).err() {
            self.link.error(error.in_column(&col))
        }
    }
    fn visit_ident(&mut self, ident: &ast::Ident) {
        let s = match ident {
            ast::Ident::Plain(s) => s,
            ast::Ident::String(s) => s,
            ast::Ident::Single(s) => s,
            ast::Ident::Double(s) => s,
            ast::Ident::Integer(s) => s,
        };
        if let Some(error) = self.comp.ident.push(s.clone()).err() {
            self.link.error(error)
        }
    }
    fn visit_variable(&mut self, var: &ast::Variable) {
        let mut link = Link::default();
        let (col, name) = match self.comp.variable(&mut link, var) {
            Ok((col, name)) => (col, name),
            Err(e) => {
                self.link.error(e);
                (0..0, "".into())
            }
        };
        if let Some(error) = self.comp.var.push((col.clone(), name, link)).err() {
            self.link.error(error.in_column(&col))
        }
    }
    fn visit_expression(&mut self, expression: &ast::Expression) {
        let mut link = Link::default();
        let col = match self.comp.expression(&mut link, expression) {
            Ok(col) => col,
            Err(e) => {
                self.link.error(e);
                0..0
            }
        };
        if let Some(error) = self.comp.expr.push((col.clone(), link)).err() {
            self.link.error(error.in_column(&col))
        }
    }
}

struct Compiler {
    ident: Stack<Rc<str>>,
    var: Stack<(Column, Rc<str>, Link)>,
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

    fn variable(&mut self, link: &mut Link, var: &ast::Variable) -> Result<(Column, Rc<str>)> {
        use ast::Variable;
        let col = match var {
            Variable::Unary(col, _ident) => col,
            Variable::Array(col, _ident, vec_expr) => {
                let len = vec_expr.len();
                let vec_expr = self.expr.pop_n(len)?;
                for (_col, ops) in vec_expr {
                    link.append(ops)?
                }
                let len_opcode = self.val_int_from_usize(len, col)?;
                link.push(len_opcode)?;
                col
            }
        };
        Ok((col.clone(), self.ident.pop()?))
    }

    fn expression(&mut self, link: &mut Link, expr: &ast::Expression) -> Result<Column> {
        fn binary_expression(this: &mut Compiler, link: &mut Link, op: Opcode) -> Result<Column> {
            let (col_rhs, rhs) = this.expr.pop()?;
            let (col_lhs, lhs) = this.expr.pop()?;
            link.append(lhs)?;
            link.append(rhs)?;
            link.push(op)?;
            Ok(col_lhs.start..col_rhs.end)
        }
        fn literal(link: &mut Link, col: &Column, val: Val) -> Result<Column> {
            link.push(Opcode::Literal(val))?;
            Ok(col.clone())
        }
        fn function(
            this: &mut Compiler,
            link: &mut Link,
            col: &Column,
            len: usize,
        ) -> Result<Column> {
            let mut args_col: std::ops::Range<usize> = 0..0;
            for (col, opcodes) in this.expr.drain(this.expr.len() - len..) {
                if args_col.start == 0 {
                    args_col.start = col.start
                }
                args_col.end = col.end;
                link.append(opcodes)?;
            }
            let ident = this.ident.pop()?;
            if let Some((opcode, arity)) = Function::opcode_and_arity(&ident) {
                if arity.contains(&len) {
                    if arity.start() != arity.end() {
                        link.push(this.val_int_from_usize(len, &col)?)?;
                    }
                    link.push(opcode)?;
                    return Ok(args_col);
                }
                return Err(error!(SyntaxError, ..&args_col; "WRONG NUMBER OF ARGUMENTS"));
            }
            if ident.starts_with("FN") {
                return Err(error!(UndefinedUserFunction, ..col));
            }
            link.push(this.val_int_from_usize(len, &col)?)?;
            link.push(Opcode::PushArr(ident))?;
            Ok(col.start..args_col.end)
        }
        use ast::Expression;
        match expr {
            Expression::Single(col, val) => literal(link, col, Val::Single(*val)),
            Expression::Double(col, val) => literal(link, col, Val::Double(*val)),
            Expression::Integer(col, val) => literal(link, col, Val::Integer(*val)),
            Expression::String(col, val) => literal(link, col, Val::String(val.clone())),
            Expression::UnaryVar(col, _) => {
                let ident = self.ident.pop()?;
                link.push(Opcode::Push(ident))?;
                Ok(col.clone())
            }
            Expression::Function(col, _, vec_exp) => function(self, link, col, vec_exp.len()),
            Expression::Negation(col, ..) => {
                let (expr_col, ops) = self.expr.pop()?;
                link.append(ops)?;
                link.push(Opcode::Neg)?;
                Ok(col.start..expr_col.end)
            }
            Expression::Exponentiation(..) => binary_expression(self, link, Opcode::Exp),
            Expression::Multiply(..) => binary_expression(self, link, Opcode::Mul),
            Expression::Divide(..) => binary_expression(self, link, Opcode::Div),
            Expression::DivideInt(..) => binary_expression(self, link, Opcode::DivInt),
            Expression::Modulus(..) => binary_expression(self, link, Opcode::Mod),
            Expression::Add(..) => binary_expression(self, link, Opcode::Add),
            Expression::Subtract(..) => binary_expression(self, link, Opcode::Sub),
            Expression::Equal(..) => binary_expression(self, link, Opcode::Eq),
            Expression::NotEqual(..) => binary_expression(self, link, Opcode::NotEq),
            Expression::Less(..) => binary_expression(self, link, Opcode::Lt),
            Expression::LessEqual(..) => binary_expression(self, link, Opcode::LtEq),
            Expression::Greater(..) => binary_expression(self, link, Opcode::Gt),
            Expression::GreaterEqual(..) => binary_expression(self, link, Opcode::GtEq),
            Expression::Not(..) => binary_expression(self, link, Opcode::Not),
            Expression::And(..) => binary_expression(self, link, Opcode::And),
            Expression::Or(..) => binary_expression(self, link, Opcode::Or),
            Expression::Xor(..) => binary_expression(self, link, Opcode::Xor),
            Expression::Imp(..) => binary_expression(self, link, Opcode::Imp),
            Expression::Eqv(..) => binary_expression(self, link, Opcode::Eqv),
        }
    }

    fn statement(&mut self, link: &mut Link, statement: &ast::Statement) -> Result<Column> {
        use ast::Statement;
        match statement {
            Statement::Clear(col, ..) => self.r#clear(link, col),
            Statement::Cont(col, ..) => self.r#cont(link, col),
            Statement::Dim(col, ..) => self.r#dim(link, col),
            Statement::End(col, ..) => self.r#end(link, col),
            Statement::For(col, ..) => self.r#for(link, col),
            Statement::Gosub(col, ..) => self.r#gosub(link, col),
            Statement::Goto(col, ..) => self.r#goto(link, col),
            Statement::If(col, _, th, el) => self.r#if(link, col, th.len(), el.len()),
            Statement::Input(col, ..) => self.r#input(link, col),
            Statement::Let(col, ..) => self.r#let(link, col),
            Statement::List(col, ..) => self.r#list(link, col),
            Statement::New(col, ..) => self.r#new_(link, col),
            Statement::Next(col, ..) => self.r#next(link, col),
            Statement::OnGoto(col, ..) => self.r#on(link, col, false),
            Statement::OnGosub(col, ..) => self.r#on(link, col, true),
            Statement::Print(col, ..) => self.r#print(link, col),
            Statement::Return(col, ..) => self.r#return(link, col),
            Statement::Run(col, ..) => self.r#run(link, col),
            Statement::Stop(col, ..) => self.r#stop(link, col),
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

    fn r#clear(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        link.push(Opcode::Clear)?;
        Ok(col.clone())
    }

    fn r#cont(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        link.push(Opcode::Cont)?;
        Ok(col.clone())
    }

    fn r#dim(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let (var_col, name, ops) = self.var.pop()?;
        if ops.is_empty() {
            return Err(error!(SyntaxError, ..&var_col; "NOT AN ARRAY"));
        } else {
            link.append(ops)?;
            link.push(Opcode::DimArr(name))?;
        }
        Ok(col.clone())
    }

    fn r#end(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        link.push(Opcode::End)?;
        Ok(col.clone())
    }

    fn r#for(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let (step_col, step_ops) = self.expr.pop()?;
        let (_to_col, to_ops) = self.expr.pop()?;
        let (_from_col, from_ops) = self.expr.pop()?;
        let var_name = self.ident.pop()?;
        link.append(from_ops)?;
        link.push(Opcode::Pop(var_name.clone()))?;
        link.append(to_ops)?;
        link.append(step_ops)?;
        link.push(Opcode::Literal(Val::String(var_name)))?;
        link.push_for(col.start..step_col.end)?;
        Ok(col.start..step_col.end)
    }

    fn r#gosub(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let (sub_col, line_number) = self.expr_pop_line_number()?;
        let full_col = col.start..sub_col.end;
        link.push_gosub(sub_col, line_number)?;
        Ok(full_col)
    }

    fn r#goto(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let (sub_col, line_number) = self.expr_pop_line_number()?;
        let full_col = col.start..sub_col.end;
        link.push_goto(sub_col, line_number)?;
        Ok(full_col)
    }

    fn r#if(
        &mut self,
        link: &mut Link,
        col: &Column,
        then_len: usize,
        else_len: usize,
    ) -> Result<Column> {
        let (_predicate_col, predicate) = self.expr.pop()?;
        link.append(predicate)?;
        let else_sym = link.next_symbol();
        link.push_ifnot(col.clone(), else_sym)?;
        let elses = self.stmt.pop_n(else_len)?;
        for (_col, stmt_ops) in self.stmt.pop_n(then_len)? {
            link.append(stmt_ops)?;
        }
        if else_len == 0 {
            link.push_symbol(else_sym);
        } else {
            let finished_sym = link.next_symbol();
            link.push_jump(col.clone(), finished_sym)?;
            link.push_symbol(else_sym);
            for (_col, stmt_ops) in elses {
                link.append(stmt_ops)?;
            }
            link.push_symbol(finished_sym);
        }
        Ok(0..0)
    }

    fn r#input(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let len = self.val_int_from_usize(self.var.len(), col)?;
        let (_prompt_col, prompt) = self.expr.pop()?;
        let (_caps_col, caps) = self.expr.pop()?;
        link.append(prompt)?;
        link.append(caps)?;
        link.push(len)?;
        for (_var_col, var_name, var_ops) in self.var.drain(..) {
            link.push(Opcode::Input(var_name.clone()))?;
            if var_ops.is_empty() {
                link.push(Opcode::Pop(var_name))?
            } else {
                link.append(var_ops)?;
                link.push(Opcode::PopArr(var_name))?
            }
        }
        link.push(Opcode::Input("".into()))?;
        Ok(col.clone())
    }

    fn r#let(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let (expr_col, expr_ops) = self.expr.pop()?;
        link.append(expr_ops)?;
        let (_var_col, var_name, var_ops) = self.var.pop()?;
        if var_ops.is_empty() {
            link.push(Opcode::Pop(var_name))?
        } else {
            link.append(var_ops)?;
            link.push(Opcode::PopArr(var_name))?
        }
        Ok(col.start..expr_col.end)
    }

    fn r#list(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let (col_to, ln_to) = self.expr_pop_line_number()?;
        let (_col_from, ln_from) = self.expr_pop_line_number()?;
        link.push(Opcode::Literal(Val::try_from(ln_from)?))?;
        link.push(Opcode::Literal(Val::try_from(ln_to)?))?;
        link.push(Opcode::List)?;
        Ok(col.start..col_to.end)
    }

    fn r#new_(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        link.push(Opcode::New)?;
        Ok(col.clone())
    }

    fn r#next(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let ident = self.ident.pop()?;
        link.push(Opcode::Next(ident))?;
        Ok(col.clone())
    }

    fn r#on(&mut self, link: &mut Link, col: &Column, is_gosub: bool) -> Result<Column> {
        let len = self.val_int_from_usize(self.expr.len(), col)?;
        let (mut sub_col, var_name, var_ops) = self.var.pop()?;
        let ret_symbol = link.next_symbol();
        if is_gosub {
            link.push_return_val(col.clone(), ret_symbol)?;
        }
        link.push(len)?;
        if var_ops.is_empty() {
            link.push(Opcode::Push(var_name))?
        } else {
            link.append(var_ops)?;
            link.push(Opcode::PushArr(var_name))?
        }
        link.push(Opcode::On)?;
        for (column, ops) in self.expr.drain(..) {
            sub_col.end = column.end;
            let ln = match LineNumber::try_from(ops) {
                Ok(ln) => ln,
                Err(e) => return Err(e),
            };
            link.push_goto(column, ln)?;
        }
        if is_gosub {
            link.push_symbol(ret_symbol);
        }
        Ok(col.start..sub_col.end)
    }

    fn r#print(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let (_expr_col, expr) = self.expr.pop()?;
        link.append(expr)?;
        link.push(Opcode::Print)?;
        Ok(col.clone())
    }

    fn r#return(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        link.push(Opcode::Return)?;
        Ok(col.clone())
    }

    fn r#run(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let mut line_number: LineNumber = None;
        let (sub_col, ops) = self.expr.pop()?;
        if let Ok(ln) = LineNumber::try_from(ops) {
            line_number = ln;
        }
        let full_col = col.start..sub_col.end;
        link.push_run(sub_col, line_number)?;
        Ok(full_col)
    }

    fn r#stop(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        link.push(Opcode::Stop)?;
        Ok(col.clone())
    }
}
