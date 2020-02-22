pub use super::ident::Ident;

#[derive(Debug, PartialEq)]
pub enum Statement {
    Data(Vec<Expression>),
    Def(Ident, Vec<Ident>),
    Dim(Ident, Vec<i16>),
    Let(Ident, Box<Expression>),
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Single(f32),
    Double(f64),
    Integer(i32),
    String(String),
    Function(Ident, Vec<Expression>),
    Add(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
    Multiply(Box<Expression>, Box<Expression>),
    Divide(Box<Expression>, Box<Expression>),
    Equality(Box<Expression>, Box<Expression>),
    Exponential(Box<Expression>, Box<Expression>),
}
