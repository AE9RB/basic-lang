use super::{Function, Link, Opcode, Program, Stack, Val};
use crate::error;
use crate::lang::ast::{self, AcceptVisitor};
use crate::lang::{Column, Error, LineNumber};
use std::convert::TryFrom;
use std::rc::Rc;

type Result<T> = std::result::Result<T, Error>;

pub fn codegen(program: &mut Program, ast: &[ast::Statement]) {
    Visitor::accept(program, ast)
}

struct Visitor<'a> {
    link: &'a mut Program,
    gen: Generator,
}

impl<'a> Visitor<'a> {
    fn accept(program: &mut Program, ast: &[ast::Statement]) {
        let mut this = Visitor {
            link: program,
            gen: Generator::new(),
        };
        for statement in ast {
            statement.accept(&mut this);
        }
        for (_col, frag) in this.gen.stmt.drain(..) {
            if let Some(error) = this.link.append(frag).err() {
                this.link.error(error);
                break;
            }
        }
        debug_assert_eq!(0, this.gen.var.len());
        debug_assert_eq!(0, this.gen.expr.len());
        debug_assert_eq!(0, this.gen.stmt.len());
    }
}

impl<'a> ast::Visitor for Visitor<'a> {
    fn visit_statement(&mut self, statement: &ast::Statement) {
        let mut link = Link::default();
        let col = match self.gen.statement(&mut link, statement) {
            Ok(col) => col,
            Err(e) => {
                self.link.error(e);
                0..0
            }
        };
        if let Some(error) = self.gen.stmt.push((col.clone(), link)).err() {
            self.link.error(error.in_column(&col))
        }
    }
    fn visit_variable(&mut self, var: &ast::Variable) {
        let mut link = Link::default();
        let (col, name, len) = match self.gen.variable(&mut link, var) {
            Ok((col, name, len)) => (col, name, len),
            Err(e) => {
                self.link.error(e);
                (0..0, "".into(), None)
            }
        };
        let var_item = VarItem::new(col.clone(), name, link, len);
        if let Some(error) = self.gen.var.push(var_item).err() {
            self.link.error(error.in_column(&col))
        }
    }
    fn visit_expression(&mut self, expression: &ast::Expression) {
        let mut link = Link::default();
        let col = match self.gen.expression(&mut link, expression) {
            Ok(col) => col,
            Err(e) => {
                self.link.error(e);
                0..0
            }
        };
        if let Some(error) = self.gen.expr.push((col.clone(), link)).err() {
            self.link.error(error.in_column(&col))
        }
    }
}

#[derive(Clone, Debug)]
struct VarItem {
    col: Column,
    name: Rc<str>,
    link: Link,
    arg_len: Option<usize>,
}

impl VarItem {
    fn new(col: Column, name: Rc<str>, link: Link, arg_len: Option<usize>) -> VarItem {
        VarItem {
            col,
            name,
            link,
            arg_len,
        }
    }

    fn test_for_built_in(&self, strict: bool) -> Result<()> {
        match Function::opcode_and_arity(&self.name) {
            Some((_, range)) if range == (0..=0) && self.arg_len.is_some() && !strict => Ok(()),
            Some((_, range)) if range != (0..=0) && self.arg_len.is_none() && !strict => Ok(()),
            Some(_) => Err(error!(SyntaxError, ..&self.col; "RESERVED FOR BUILT-IN")),
            None => Ok(()),
        }
    }

    fn push_as_dim(self, link: &mut Link) -> Result<Column> {
        self.test_for_built_in(true)?;
        if let Some(len) = self.arg_len {
            if len > 0 {
                link.append(self.link)?;
                link.push(Opcode::Literal(Val::try_from(len)?))?;
                link.push(Opcode::DimArr(self.name))?;
                return Ok(self.col);
            }
        }
        Err(error!(SyntaxError, ..&self.col; "NOT AN ARRAY"))
    }

    fn push_as_pop_unary(self, link: &mut Link) -> Result<Column> {
        self.test_for_built_in(false)?;
        debug_assert!(self.arg_len.is_none());
        debug_assert!(self.link.is_empty());
        link.push(Opcode::Pop(self.name))?;
        Ok(self.col)
    }

    fn push_as_pop(self, link: &mut Link) -> Result<Column> {
        self.test_for_built_in(false)?;
        if let Some(len) = self.arg_len {
            if len > 0 {
                link.append(self.link)?;
                link.push(Opcode::Literal(Val::try_from(len)?))?;
                link.push(Opcode::PopArr(self.name))?;
            } else {
                return Err(error!(SyntaxError, ..&self.col; "MISSING INDEX EXPRESSION"));
            }
        } else {
            debug_assert!(self.link.is_empty());
            link.push(Opcode::Pop(self.name))?;
        }
        Ok(self.col)
    }

    fn push_as_expression(self, link: &mut Link) -> Result<Column> {
        link.append(self.link)?;
        if let Some((opcode, arity)) = Function::opcode_and_arity(&self.name) {
            if arity == (0..=0) && self.arg_len.is_none() {
                link.push(opcode)?;
                return Ok(self.col.clone());
            } else if let Some(len) = self.arg_len {
                if arity.contains(&len) {
                    if arity.start() != arity.end() {
                        link.push(Opcode::Literal(Val::try_from(len)?))?;
                    }
                    link.push(opcode)?;
                    return Ok(self.col.clone());
                }
                return Err(error!(IllegalFunctionCall, ..&self.col; "WRONG NUMBER OF ARGUMENTS"));
            }
        }
        match self.arg_len {
            None => link.push(Opcode::Push(self.name))?,
            Some(len) => {
                if self.name.starts_with("FN") {
                    link.push(Opcode::Literal(Val::try_from(len)?))?;
                    link.push(Opcode::Fn(self.name))?;
                } else {
                    link.push(Opcode::Literal(Val::try_from(len)?))?;
                    link.push(Opcode::PushArr(self.name))?;
                }
            }
        }
        Ok(self.col)
    }
}

struct Generator {
    var: Stack<VarItem>,
    expr: Stack<(Column, Link)>,
    stmt: Stack<(Column, Link)>,
}

impl Generator {
    fn new() -> Generator {
        Generator {
            var: Stack::new("VARIABLE OVERFLOW"),
            expr: Stack::new("EXPRESSION OVERFLOW"),
            stmt: Stack::new("STATEMENT OVERFLOW"),
        }
    }

    fn variable(
        &mut self,
        link: &mut Link,
        var: &ast::Variable,
    ) -> Result<(Column, Rc<str>, Option<usize>)> {
        use ast::Variable;
        let (col, ident, len) = match var {
            Variable::Unary(col, ident) => (col, ident, None),
            Variable::Array(col, ident, vec_expr) => {
                let len = vec_expr.len();
                let vec_expr = self.expr.pop_n(len)?;
                for (_col, ops) in vec_expr {
                    link.append(ops)?
                }
                (col, ident, Some(len))
            }
        };
        let s = match ident {
            ast::Ident::Plain(s) => s,
            ast::Ident::String(s) => s,
            ast::Ident::Single(s) => s,
            ast::Ident::Double(s) => s,
            ast::Ident::Integer(s) => s,
        };
        Ok((col.clone(), s.clone(), len))
    }

    fn expression(&mut self, link: &mut Link, expr: &ast::Expression) -> Result<Column> {
        fn unary_expression(
            this: &mut Generator,
            link: &mut Link,
            op: Opcode,
            col: &Column,
        ) -> Result<Column> {
            let (expr_col, ops) = this.expr.pop()?;
            link.append(ops)?;
            link.push(op)?;
            Ok(col.start..expr_col.end)
        }
        fn binary_expression(this: &mut Generator, link: &mut Link, op: Opcode) -> Result<Column> {
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
        use ast::Expression;
        match expr {
            Expression::Single(col, val) => literal(link, col, Val::Single(*val)),
            Expression::Double(col, val) => literal(link, col, Val::Double(*val)),
            Expression::Integer(col, val) => literal(link, col, Val::Integer(*val)),
            Expression::String(col, val) => literal(link, col, Val::String(val.clone())),
            Expression::Variable(_) => self.var.pop()?.push_as_expression(link),
            Expression::Negation(col, ..) => unary_expression(self, link, Opcode::Neg, col),
            Expression::Power(..) => binary_expression(self, link, Opcode::Pow),
            Expression::Multiply(..) => binary_expression(self, link, Opcode::Mul),
            Expression::Divide(..) => binary_expression(self, link, Opcode::Div),
            Expression::DivideInt(..) => binary_expression(self, link, Opcode::DivInt),
            Expression::Modulo(..) => binary_expression(self, link, Opcode::Mod),
            Expression::Add(..) => binary_expression(self, link, Opcode::Add),
            Expression::Subtract(..) => binary_expression(self, link, Opcode::Sub),
            Expression::Equal(..) => binary_expression(self, link, Opcode::Eq),
            Expression::NotEqual(..) => binary_expression(self, link, Opcode::NotEq),
            Expression::Less(..) => binary_expression(self, link, Opcode::Lt),
            Expression::LessEqual(..) => binary_expression(self, link, Opcode::LtEq),
            Expression::Greater(..) => binary_expression(self, link, Opcode::Gt),
            Expression::GreaterEqual(..) => binary_expression(self, link, Opcode::GtEq),
            Expression::Not(col, ..) => unary_expression(self, link, Opcode::Not, col),
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
            Statement::Cls(col, ..) => self.r#cls(link, col),
            Statement::Cont(col, ..) => self.r#cont(link, col),
            Statement::Data(col, v) => self.r#data(link, col, v.len()),
            Statement::Def(col, _, v, _) => self.r#def(link, col, v.len()),
            Statement::Defdbl(col, ..) => self.r#defdbl(link, col),
            Statement::Defint(col, ..) => self.r#defint(link, col),
            Statement::Defsng(col, ..) => self.r#defsng(link, col),
            Statement::Defstr(col, ..) => self.r#defstr(link, col),
            Statement::Delete(col, ..) => self.r#delete(link, col),
            Statement::Dim(col, v) => self.r#dim(link, col, v.len()),
            Statement::End(col, ..) => self.r#end(link, col),
            Statement::Erase(col, v) => self.r#erase(link, col, v.len()),
            Statement::For(col, ..) => self.r#for(link, col),
            Statement::Gosub(col, ..) => self.r#gosub(link, col),
            Statement::Goto(col, ..) => self.r#goto(link, col),
            Statement::If(col, _, th, el) => self.r#if(link, col, th.len(), el.len()),
            Statement::Input(col, _, _, v) => self.r#input(link, col, v.len()),
            Statement::Let(col, ..) => self.r#let(link, col),
            Statement::List(col, ..) => self.r#list(link, col),
            Statement::Load(col, ..) => self.r#load(link, col),
            Statement::Mid(col, ..) => self.r#mid(link, col),
            Statement::New(col, ..) => self.r#new_(link, col),
            Statement::Next(col, v) => self.r#next(link, col, v.len()),
            Statement::OnGoto(col, _, v) => self.r#on(link, col, v.len(), false),
            Statement::OnGosub(col, _, v) => self.r#on(link, col, v.len(), true),
            Statement::Print(col, v) => self.r#print(link, col, v.len()),
            Statement::Read(col, v) => self.r#read(link, col, v.len()),
            Statement::Renum(col, ..) => self.r#renum(link, col),
            Statement::Restore(col, ..) => self.r#restore(link, col),
            Statement::Return(col, ..) => self.r#return(link, col),
            Statement::Run(col, ..) => self.r#run(link, col),
            Statement::Save(col, ..) => self.r#save(link, col),
            Statement::Stop(col, ..) => self.r#stop(link, col),
            Statement::Swap(col, ..) => self.r#swap(link, col),
            Statement::Troff(col, ..) => self.r#troff(link, col),
            Statement::Tron(col, ..) => self.r#tron(link, col),
            Statement::Wend(col, ..) => self.r#wend(link, col),
            Statement::While(col, ..) => self.r#while(link, col),
        }
    }

    fn expr_pop_line_number(&mut self) -> Result<(Column, LineNumber)> {
        let (sub_col, ops) = self.expr.pop()?;
        match LineNumber::try_from(&ops) {
            Ok(ln) => Ok((sub_col, ln)),
            Err(e) => Err(e.in_column(&sub_col)),
        }
    }

    fn r#clear(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        link.push(Opcode::Clear)?;
        Ok(col.clone())
    }

    fn r#cls(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        link.push(Opcode::Cls)?;
        Ok(col.clone())
    }

    fn r#cont(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        link.push(Opcode::Cont)?;
        Ok(col.clone())
    }

    fn r#data(&mut self, link: &mut Link, col: &Column, len: usize) -> Result<Column> {
        let exprs = self.expr.pop_n(len)?;
        for (expr_col, mut expr_link) in exprs {
            expr_link.transform_to_data(&expr_col)?;
            link.append(expr_link)?;
        }
        Ok(col.clone())
    }

    fn r#def(&mut self, link: &mut Link, col: &Column, len: usize) -> Result<Column> {
        let mut vars = self.var.pop_n(len)?;
        let fn_name = self.var.pop()?;
        debug_assert!(fn_name.arg_len.is_none());
        let (_expr_col, expr_ops) = self.expr.pop()?;
        let fn_vars: Vec<Rc<str>> = vars
            .drain(..)
            .map(|var_item| {
                debug_assert!(var_item.arg_len.is_none());
                var_item.name
            })
            .collect();
        link.push_def_fn(col.clone(), fn_name.name, fn_vars, expr_ops)?;
        Ok(col.clone())
    }

    fn r#defdbl(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let to = self.var.pop()?;
        let from = self.var.pop()?;
        link.push(Opcode::Literal(Val::String(from.name)))?;
        link.push(Opcode::Literal(Val::String(to.name)))?;
        link.push(Opcode::Defdbl)?;
        Ok(col.clone())
    }

    fn r#defint(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let to = self.var.pop()?;
        let from = self.var.pop()?;
        link.push(Opcode::Literal(Val::String(from.name)))?;
        link.push(Opcode::Literal(Val::String(to.name)))?;
        link.push(Opcode::Defint)?;
        Ok(col.clone())
    }

    fn r#defsng(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let to = self.var.pop()?;
        let from = self.var.pop()?;
        link.push(Opcode::Literal(Val::String(from.name)))?;
        link.push(Opcode::Literal(Val::String(to.name)))?;
        link.push(Opcode::Defsng)?;
        Ok(col.clone())
    }

    fn r#defstr(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let to = self.var.pop()?;
        let from = self.var.pop()?;
        link.push(Opcode::Literal(Val::String(from.name)))?;
        link.push(Opcode::Literal(Val::String(to.name)))?;
        link.push(Opcode::Defstr)?;
        Ok(col.clone())
    }

    fn r#delete(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let (col_to, ln_to) = self.expr_pop_line_number()?;
        let (_col_from, ln_from) = self.expr_pop_line_number()?;
        link.push(Opcode::Literal(Val::try_from(ln_from)?))?;
        link.push(Opcode::Literal(Val::try_from(ln_to)?))?;
        link.push(Opcode::Delete)?;
        Ok(col.start..col_to.end)
    }

    fn r#dim(&mut self, link: &mut Link, col: &Column, len: usize) -> Result<Column> {
        let mut col = col.clone();
        for var in self.var.pop_n(len)? {
            let sub_col = var.push_as_dim(link)?;
            col.end = sub_col.end;
        }
        Ok(col)
    }

    fn r#end(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        link.push(Opcode::End)?;
        Ok(col.clone())
    }

    fn r#erase(&mut self, link: &mut Link, col: &Column, len: usize) -> Result<Column> {
        for var in self.var.pop_n(len)? {
            link.push(Opcode::EraseArr(var.name))?;
        }
        Ok(col.clone())
    }

    fn r#for(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let (step_col, step_ops) = self.expr.pop()?;
        let (_to_col, to_ops) = self.expr.pop()?;
        let (_from_col, from_ops) = self.expr.pop()?;
        let var = self.var.pop()?;
        let var_name = var.name.clone();
        link.append(from_ops)?;
        var.push_as_pop_unary(link)?;
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
        Ok(col.clone())
    }

    fn r#input(&mut self, link: &mut Link, col: &Column, len: usize) -> Result<Column> {
        let (_prompt_col, prompt) = self.expr.pop()?;
        let (_caps_col, caps) = self.expr.pop()?;
        link.append(prompt)?;
        link.append(caps)?;
        link.push(Opcode::Literal(Val::try_from(len)?))?;
        for var in self.var.pop_n(len)? {
            link.push(Opcode::Input(var.name.clone()))?;
            var.push_as_pop(link)?;
        }
        link.push(Opcode::Input("".into()))?;
        Ok(col.clone())
    }

    fn r#let(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let (expr_col, expr_ops) = self.expr.pop()?;
        link.append(expr_ops)?;
        self.var.pop()?.push_as_pop(link)?;
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

    fn r#load(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let (sub_col, expr) = self.expr.pop()?;
        link.append(expr)?;
        link.push(Opcode::Load)?;
        Ok(col.start..sub_col.end)
    }

    fn r#mid(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let var = self.var.pop()?;
        let (expr_col, expr_link) = self.expr.pop()?;
        let (_len_col, len_link) = self.expr.pop()?;
        let (_pos_col, pos_link) = self.expr.pop()?;
        var.clone().push_as_expression(link)?;
        link.append(expr_link)?;
        link.append(len_link)?;
        link.append(pos_link)?;
        link.push(Opcode::LetMid)?;
        var.push_as_pop(link)?;
        Ok(col.start..expr_col.end)
    }

    fn r#new_(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        link.push(Opcode::New)?;
        Ok(col.clone())
    }

    fn r#next(&mut self, link: &mut Link, col: &Column, len: usize) -> Result<Column> {
        for var in self.var.pop_n(len)? {
            var.test_for_built_in(false)?;
            link.push(Opcode::Next(var.name))?;
        }
        Ok(col.clone())
    }

    fn r#on(
        &mut self,
        link: &mut Link,
        col: &Column,
        len: usize,
        is_gosub: bool,
    ) -> Result<Column> {
        let line_numbers = self.expr.pop_n(len)?;
        let len = Val::try_from(len)?;
        let (mut sub_col, var_ops) = self.expr.pop()?;
        let ret_symbol = link.next_symbol();
        if is_gosub {
            link.push_return_val(col.clone(), ret_symbol)?;
        }
        link.push(Opcode::Literal(len))?;
        link.append(var_ops)?;
        link.push(Opcode::On)?;
        for (column, ops) in line_numbers {
            sub_col.end = column.end;
            let ln = match LineNumber::try_from(&ops) {
                Ok(ln) => ln,
                Err(e) => return Err(e.in_column(&column)),
            };
            link.push_goto(column, ln)?;
        }
        if is_gosub {
            link.push_symbol(ret_symbol);
        }
        Ok(col.start..sub_col.end)
    }

    fn r#print(&mut self, link: &mut Link, col: &Column, len: usize) -> Result<Column> {
        for (_col, expr_ops) in self.expr.pop_n(len)? {
            link.append(expr_ops)?;
            link.push(Opcode::Print)?;
        }
        Ok(col.clone())
    }

    fn r#read(&mut self, link: &mut Link, col: &Column, len: usize) -> Result<Column> {
        for var in self.var.pop_n(len)? {
            link.push(Opcode::Read)?;
            var.push_as_pop(link)?;
        }
        Ok(col.clone())
    }

    fn r#renum(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let (_col_step, step) = self.expr_pop_line_number()?;
        let (_col_old_start, old_start) = self.expr_pop_line_number()?;
        let (_col_new_start, new_start) = self.expr_pop_line_number()?;
        if let Some(new_start) = new_start {
            if let Some(old_start) = old_start {
                if let Some(step) = step {
                    link.push(Opcode::Literal(Val::Single(new_start as f32)))?;
                    link.push(Opcode::Literal(Val::Single(old_start as f32)))?;
                    link.push(Opcode::Literal(Val::Single(step as f32)))?;
                    link.push(Opcode::Renum)?;
                }
            }
        }
        Ok(col.clone())
    }

    fn r#restore(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let mut line_number: LineNumber = None;
        let (sub_col, ops) = self.expr.pop()?;
        if let Ok(ln) = LineNumber::try_from(&ops) {
            line_number = ln;
        }
        link.push_restore(sub_col, line_number)?;
        Ok(col.clone())
    }

    fn r#return(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        link.push(Opcode::Return)?;
        Ok(col.clone())
    }

    fn r#run(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let (sub_col, ops) = self.expr.pop()?;
        let full_col = col.start..sub_col.end;
        if let Ok(filename) = Rc::<str>::try_from(&ops) {
            link.push(Opcode::Literal(Val::String(filename)))?;
            link.push(Opcode::LoadRun)?;
        } else if let Ok(ln) = LineNumber::try_from(&ops) {
            link.push_run(sub_col, ln)?;
        } else {
            link.push_run(sub_col, None)?;
        }
        Ok(full_col)
    }

    fn r#save(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let (sub_col, expr) = self.expr.pop()?;
        link.append(expr)?;
        link.push(Opcode::Save)?;
        Ok(col.start..sub_col.end)
    }

    fn r#stop(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        link.push(Opcode::Stop)?;
        Ok(col.clone())
    }

    fn r#swap(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let var1 = self.var.pop()?;
        let var2 = self.var.pop()?;
        var1.test_for_built_in(false)?;
        var2.test_for_built_in(false)?;
        var1.clone().push_as_expression(link)?;
        var2.clone().push_as_expression(link)?;
        link.push(Opcode::Swap)?;
        var1.push_as_pop(link)?;
        var2.push_as_pop(link)?;
        Ok(col.clone())
    }

    fn r#troff(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        link.push(Opcode::Troff)?;
        Ok(col.clone())
    }

    fn r#tron(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        link.push(Opcode::Tron)?;
        Ok(col.clone())
    }

    fn r#wend(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        link.push_wend(col.clone())?;
        Ok(col.clone())
    }

    fn r#while(&mut self, link: &mut Link, col: &Column) -> Result<Column> {
        let (sub_col, expr) = self.expr.pop()?;
        link.push_while(col.clone(), expr)?;
        Ok(col.start..sub_col.end)
    }
}
