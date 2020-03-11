pub use super::ident::Ident;
use super::Column;

#[derive(Debug, PartialEq)]
pub enum Statement {
    Goto(Column, Expression),
    Let(Column, (Column, Ident), Expression),
    List(Column, Expression, Expression),
    Print(Column, Vec<Expression>),
    Run(Column),
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Single(Column, f32),
    Double(Column, f64),
    Integer(Column, i16),
    String(Column, String),
    Char(Column, char),
    Var(Column, Ident),
    Function(Column, Ident, Vec<Expression>),
    Negation(Column, Box<Expression>),
    Add(Column, Box<Expression>, Box<Expression>),
    Subtract(Column, Box<Expression>, Box<Expression>),
    Multiply(Column, Box<Expression>, Box<Expression>),
    Divide(Column, Box<Expression>, Box<Expression>),
}

pub trait Visitor {
    fn visit_statement(&mut self, _: &Statement) {}
    fn visit_ident(&mut self, _: &Ident) {}
    fn visit_expression(&mut self, _: &Expression) {}
}

pub trait AcceptVisitor {
    fn accept<V: Visitor>(&self, visitor: &mut V);
}

impl AcceptVisitor for Ident {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_ident(self)
    }
}

impl AcceptVisitor for Statement {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        use Statement::*;
        match self {
            Goto(_, expr) => {
                expr.accept(visitor);
            }
            Let(_, (_, ident), expr) => {
                ident.accept(visitor);
                expr.accept(visitor);
            }
            Print(_, vec_expr) => {
                for expr in vec_expr {
                    expr.accept(visitor);
                }
            }
            List(_, expr1, expr2) => {
                expr1.accept(visitor);
                expr2.accept(visitor);
            }
            Run(_) => {}
        }
        visitor.visit_statement(self)
    }
}

impl AcceptVisitor for Expression {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        use Expression::*;
        match self {
            Single(..) | Double(..) | Integer(..) | String(..) | Char(..) => {}
            Var(_, ident) => {
                ident.accept(visitor);
            }
            Function(_, ident, vec_expr) => {
                ident.accept(visitor);
                for expr in vec_expr {
                    expr.accept(visitor);
                }
            }
            Negation(_, expr) => expr.accept(visitor),
            Add(_, expr1, expr2)
            | Subtract(_, expr1, expr2)
            | Multiply(_, expr1, expr2)
            | Divide(_, expr1, expr2) => {
                expr1.accept(visitor);
                expr2.accept(visitor);
            }
        }
        visitor.visit_expression(self)
    }
}
