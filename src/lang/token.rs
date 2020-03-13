extern crate macros;
pub use super::ident::Ident;
use super::{Error, LineNumber, MaxValue};
use crate::error;
use macros::EnumFieldLess;
use std::convert::TryFrom;

use std::collections::HashMap;

thread_local!(
    static STRING_TO_TOKEN: HashMap<std::string::String, Token> = Token::field_less()
        .drain(..)
        .chain(Word::field_less().drain(..).map(|x| Token::Word(x)))
        .chain(Operator::field_less().drain(..).map(|x| Token::Operator(x)))
        .map(|d| (d.to_string(), d))
        .collect();
);

#[derive(Debug, PartialEq, EnumFieldLess, Clone)]
pub enum Token {
    Unknown(String),
    Whitespace(usize),
    Literal(Literal),
    Word(Word),
    Operator(Operator),
    Ident(Ident),
    LParen,
    RParen,
    Comma,
    Colon,
    Semicolon,
}

impl Token {
    pub fn from_string(s: &str) -> Option<Token> {
        STRING_TO_TOKEN.with(|stt| match stt.get(s) {
            Some(t) => Some(t.clone()),
            None => None,
        })
    }
    pub fn is_word(&self) -> bool {
        match self {
            Token::Word(_) => true,
            Token::Ident(_) => true,
            Token::Literal(_) => true,
            Token::Operator(op) => op.is_word(),
            _ => false,
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Token::*;
        match self {
            Unknown(s) => write!(f, "{}", s),
            Whitespace(u) => write!(f, "{s:>w$}", s = "", w = u),
            Literal(s) => write!(f, "{}", s),
            Word(s) => write!(f, "{}", s),
            Operator(s) => write!(f, "{}", s),
            Ident(s) => write!(f, "{}", s),
            LParen => write!(f, "("),
            RParen => write!(f, ")"),
            Comma => write!(f, ","),
            Colon => write!(f, ":"),
            Semicolon => write!(f, ";"),
        }
    }
}

impl TryFrom<&Token> for LineNumber {
    type Error = Error;
    fn try_from(token: &Token) -> Result<Self, Self::Error> {
        let msg = "INVALID LINE NUMBER";
        if let Token::Literal(lit) = token {
            let s = match lit {
                Literal::Integer(s) => s,
                Literal::Single(s) => s,
                Literal::Double(s) => s,
                Literal::String(_) => "",
            };
            if s.chars().all(|c| c.is_ascii_digit()) {
                if let Ok(line) = s.parse::<u16>() {
                    if line <= LineNumber::max_value() {
                        return Ok(Some(line));
                    }
                }
                return Err(error!(Overflow; msg));
            }
        }
        Err(error!(SyntaxError; msg))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Single(String),
    Double(String),
    Integer(String),
    String(String),
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Literal::*;
        match self {
            Single(s) => write!(f, "{}", s),
            Double(s) => write!(f, "{}", s),
            Integer(s) => write!(f, "{}", s),
            String(s) => write!(f, "\"{}\"", s),
        }
    }
}

#[derive(Debug, PartialEq, Clone, EnumFieldLess)]
pub enum Word {
    End,
    For,
    Gosub1,
    Gosub2,
    Goto1,
    Goto2,
    Let,
    List,
    Print1,
    Print2,
    Rem1,
    Rem2,
    Run,
    To,
}

impl std::fmt::Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Word::*;
        match self {
            End => write!(f, "END"),
            For => write!(f, "FOR"),
            Gosub1 => write!(f, "GOSUB"),
            Gosub2 => write!(f, "GO SUB"),
            Goto1 => write!(f, "GOTO"),
            Goto2 => write!(f, "GO TO"),
            Let => write!(f, "LET"),
            List => write!(f, "LIST"),
            Print1 => write!(f, "PRINT"),
            Print2 => write!(f, "?"),
            Rem1 => write!(f, "REM"),
            Rem2 => write!(f, "'"),
            Run => write!(f, "RUN"),
            To => write!(f, "TO"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, EnumFieldLess)]
pub enum Operator {
    Caret,
    Multiply,
    Divide,
    DivideInt,
    Modulus,
    Plus,
    Minus,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Not,
    And,
    Or,
    Xor,
    Imp,
    Eqv,
}

impl Operator {
    pub fn is_word(&self) -> bool {
        use Operator::*;
        match self {
            Caret | Multiply | Divide | DivideInt | Plus | Minus | Equal | NotEqual | Less
            | LessEqual | Greater | GreaterEqual => false,
            Modulus | Not | And | Or | Xor | Imp | Eqv => true,
        }
    }
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Operator::*;
        match self {
            Caret => write!(f, "^"),
            Multiply => write!(f, "*"),
            Divide => write!(f, "/"),
            DivideInt => write!(f, "\\"),
            Modulus => write!(f, "MOD"),
            Plus => write!(f, "+"),
            Minus => write!(f, "-"),
            Equal => write!(f, "="),
            NotEqual => write!(f, "<>"),
            Less => write!(f, "<"),
            LessEqual => write!(f, "<="),
            Greater => write!(f, ">"),
            GreaterEqual => write!(f, ">="),
            Not => write!(f, "NOT"),
            And => write!(f, "AND"),
            Or => write!(f, "OR"),
            Xor => write!(f, "XOR"),
            Imp => write!(f, "IMP"),
            Eqv => write!(f, "EQV"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string() {
        let t = Token::from_string("REM");
        assert_eq!(t, Some(Token::Word(Word::Rem1)));
        let t = Token::from_string("PICKLES");
        assert_eq!(t, None);
    }
}
