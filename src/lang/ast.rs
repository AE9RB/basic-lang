use super::Column;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum Statement {
    Clear(Column),
    Cont(Column),
    Def(Column, Variable, Vec<Variable>, Expression),
    Dim(Column, Variable),
    End(Column),
    For(Column, Variable, Expression, Expression, Expression),
    Gosub(Column, Expression),
    Goto(Column, Expression),
    If(Column, Expression, Vec<Statement>, Vec<Statement>),
    Input(Column, Expression, Expression, Vec<Variable>),
    Let(Column, Variable, Expression),
    List(Column, Expression, Expression),
    New(Column),
    Next(Column, Variable),
    OnGoto(Column, Expression, Vec<Expression>),
    OnGosub(Column, Expression, Vec<Expression>),
    Print(Column, Expression),
    Return(Column),
    Run(Column, Expression),
    Stop(Column),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Variable {
    Unary(Column, Ident),
    Array(Column, Ident, Vec<Expression>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Variable(Variable),
    Single(Column, f32),
    Double(Column, f64),
    Integer(Column, i16),
    String(Column, Rc<str>),
    Negation(Column, Box<Expression>),
    Power(Column, Box<Expression>, Box<Expression>),
    Multiply(Column, Box<Expression>, Box<Expression>),
    Divide(Column, Box<Expression>, Box<Expression>),
    DivideInt(Column, Box<Expression>, Box<Expression>),
    Modulus(Column, Box<Expression>, Box<Expression>),
    Add(Column, Box<Expression>, Box<Expression>),
    Subtract(Column, Box<Expression>, Box<Expression>),
    Equal(Column, Box<Expression>, Box<Expression>),
    NotEqual(Column, Box<Expression>, Box<Expression>),
    Less(Column, Box<Expression>, Box<Expression>),
    LessEqual(Column, Box<Expression>, Box<Expression>),
    Greater(Column, Box<Expression>, Box<Expression>),
    GreaterEqual(Column, Box<Expression>, Box<Expression>),
    Not(Column, Box<Expression>, Box<Expression>),
    And(Column, Box<Expression>, Box<Expression>),
    Or(Column, Box<Expression>, Box<Expression>),
    Xor(Column, Box<Expression>, Box<Expression>),
    Imp(Column, Box<Expression>, Box<Expression>),
    Eqv(Column, Box<Expression>, Box<Expression>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Ident {
    Plain(Rc<str>),
    String(Rc<str>),
    Single(Rc<str>),
    Double(Rc<str>),
    Integer(Rc<str>),
}

pub trait Visitor {
    fn visit_statement(&mut self, _: &Statement) {}
    fn visit_variable(&mut self, _: &Variable) {}
    fn visit_expression(&mut self, _: &Expression) {}
}

pub trait AcceptVisitor {
    fn accept<V: Visitor>(&self, visitor: &mut V);
}

impl AcceptVisitor for Variable {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        use Variable::*;
        match self {
            Unary(..) => {}
            Array(_, _, vec_expr) => {
                for expr in vec_expr {
                    expr.accept(visitor);
                }
            }
        }
        visitor.visit_variable(self)
    }
}

impl AcceptVisitor for Statement {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        use Statement::*;
        match self {
            Clear(_) | Cont(_) | End(_) | New(_) | Stop(_) | Return(_) => {}
            Def(_, var, vec_var, expr) => {
                var.accept(visitor);
                for v in vec_var {
                    v.accept(visitor);
                }
                expr.accept(visitor);
            }
            Dim(_, var) => {
                var.accept(visitor);
            }
            For(_, var, expr1, expr2, expr3) => {
                var.accept(visitor);
                expr1.accept(visitor);
                expr2.accept(visitor);
                expr3.accept(visitor);
            }
            Gosub(_, expr) | Goto(_, expr) | Print(_, expr) | Run(_, expr) => {
                expr.accept(visitor);
            }
            If(_, predicate, then_stmt, else_stmt) => {
                predicate.accept(visitor);
                for stmt in then_stmt {
                    stmt.accept(visitor);
                }
                for stmt in else_stmt {
                    stmt.accept(visitor);
                }
            }
            Let(_, var, expr) => {
                var.accept(visitor);
                expr.accept(visitor);
            }
            List(_, expr1, expr2) => {
                expr1.accept(visitor);
                expr2.accept(visitor);
            }
            Input(_, expr1, expr2, vec_var) => {
                expr1.accept(visitor);
                expr2.accept(visitor);
                for var in vec_var {
                    var.accept(visitor);
                }
            }
            Next(_, var) => {
                var.accept(visitor);
            }
            OnGoto(_, expr, vec_expr) | OnGosub(_, expr, vec_expr) => {
                expr.accept(visitor);
                for expr in vec_expr {
                    expr.accept(visitor);
                }
            }
        }
        visitor.visit_statement(self)
    }
}

impl AcceptVisitor for Expression {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        use Expression::*;
        match self {
            Single(..) | Double(..) | Integer(..) | String(..) => {}
            Variable(var) => var.accept(visitor),
            Negation(_, expr) => expr.accept(visitor),
            Power(_, expr1, expr2)
            | Multiply(_, expr1, expr2)
            | Divide(_, expr1, expr2)
            | DivideInt(_, expr1, expr2)
            | Modulus(_, expr1, expr2)
            | Add(_, expr1, expr2)
            | Subtract(_, expr1, expr2)
            | Equal(_, expr1, expr2)
            | NotEqual(_, expr1, expr2)
            | Less(_, expr1, expr2)
            | LessEqual(_, expr1, expr2)
            | Greater(_, expr1, expr2)
            | GreaterEqual(_, expr1, expr2)
            | Not(_, expr1, expr2)
            | And(_, expr1, expr2)
            | Or(_, expr1, expr2)
            | Xor(_, expr1, expr2)
            | Imp(_, expr1, expr2)
            | Eqv(_, expr1, expr2) => {
                expr1.accept(visitor);
                expr2.accept(visitor);
            }
        }
        visitor.visit_expression(self)
    }
}
