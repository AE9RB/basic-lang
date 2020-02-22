// Used in both Token and Ast

#[derive(Debug, PartialEq, Hash, Clone)]
pub enum Ident {
    Plain(String),
    String(String),
    Single(String),
    Double(String),
    Integer(String),
}

impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Ident::*;
        match self {
            Plain(s) => write!(f, "{}", s),
            String(s) => write!(f, "{}", s),
            Single(s) => write!(f, "{}", s),
            Double(s) => write!(f, "{}", s),
            Integer(s) => write!(f, "{}", s),
        }
    }
}
