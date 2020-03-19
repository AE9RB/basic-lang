use super::Column;

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub enum Statement {
    Clear(Column),
    Cont(Column),
    End(Column),
    For(Column, Ident, Expression, Expression, Expression),
    Goto(Column, Expression),
    Input(Column, Expression, Expression, Vec<Ident>),
    Let(Column, Ident, Expression),
    List(Column, Expression, Expression),
    Next(Column, Vec<Ident>),
    Print(Column, Vec<Expression>),
    Run(Column, Expression),
    Stop(Column),
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub enum Ident {
    Plain(Column, String),
    String(Column, String),
    Single(Column, String),
    Double(Column, String),
    Integer(Column, String),
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub enum Expression {
    Single(Column, f32),
    Double(Column, f64),
    Integer(Column, i16),
    String(Column, String),
    Char(Column, char),
    Var(Column, Ident),
    Function(Column, Ident, Vec<Expression>),
    Negation(Column, Box<Expression>),
    Exponentiation(Column, Box<Expression>, Box<Expression>),
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
            Clear(_) | Cont(_) | End(_) | Stop(_) => {}
            For(_, ident, expr1, expr2, expr3) => {
                ident.accept(visitor);
                expr1.accept(visitor);
                expr2.accept(visitor);
                expr3.accept(visitor);
            }
            Goto(_, expr) => {
                expr.accept(visitor);
            }
            Let(_, ident, expr) => {
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
            Input(_, expr1, expr2, vec_ident) => {
                expr1.accept(visitor);
                expr2.accept(visitor);
                for var in vec_ident {
                    var.accept(visitor);
                }
            }
            Next(_, vec_ident) => {
                for var in vec_ident {
                    var.accept(visitor);
                }
            }
            Run(_, expr) => {
                expr.accept(visitor);
            }
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
            Exponentiation(_, expr1, expr2)
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
