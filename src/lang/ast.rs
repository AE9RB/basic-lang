pub use super::ident::Ident;

pub type Column = std::ops::Range<usize>;

#[derive(Debug, PartialEq)]
pub enum Statement {
    Let(Column, (Column, Ident), Expression),
    Print(Column, Vec<Expression>),
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Single(Column, f32),
    Double(Column, f64),
    Integer(Column, i16),
    String(Column, String),
    Char(Column, char),
    Ident(Column, Ident),
    Function(Column, Ident, Vec<Expression>),
    Add(Column, Box<Expression>, Box<Expression>),
    Subtract(Column, Box<Expression>, Box<Expression>),
    Multiply(Column, Box<Expression>, Box<Expression>),
    Divide(Column, Box<Expression>, Box<Expression>),
}
