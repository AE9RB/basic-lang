extern crate macros;
pub use super::ident::Ident;
use macros::EnumIter;

use std::collections::HashMap;

thread_local!(
    static STRING_TO_TOKEN: HashMap<std::string::String, Token> = Token::iter()
        .cloned()
        .chain(Word::iter().map(|x| Token::Word(x.clone())))
        .chain(Operator::iter().map(|x| Token::Operator(x.clone())))
        .map(|d| (d.to_string(), d))
        .collect();
);

#[derive(Debug, PartialEq, Clone, EnumIter)]
pub enum Token {
    Unknown(String),
    Whitespace(usize),
    Literal(Literal),
    Word(Word),
    Operator(Operator),
    Ident(Ident),
    ParenOpen,
    ParenClose,
    Comma,
    Colon,
}

impl Token {
    pub fn from_string(s: &str) -> Option<Token> {
        STRING_TO_TOKEN.with(|stt| match stt.get(s) {
            Some(t) => Some(t.clone()),
            None => None,
        })
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
            ParenOpen => write!(f, "("),
            ParenClose => write!(f, ")"),
            Comma => write!(f, ","),
            Colon => write!(f, ":"),
        }
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

#[derive(Debug, PartialEq, Clone, EnumIter)]
pub enum Word {
    Data,
    Def,
    Dim,
    Else,
    End,
    For,
    Gosub1,
    Gosub2,
    Goto1,
    Goto2,
    If,
    Input,
    Let,
    Next,
    On,
    Print1,
    Print2,
    Read,
    Rem1,
    Rem2,
    Restore,
    Return,
    Stop,
    Then,
    To,
}

impl std::fmt::Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Word::*;
        match self {
            Data => write!(f, "DATA"),
            Def => write!(f, "DEF"),
            Dim => write!(f, "DIM"),
            Else => write!(f, "ELSE"),
            End => write!(f, "END"),
            For => write!(f, "FOR"),
            Gosub1 => write!(f, "GOSUB"),
            Gosub2 => write!(f, "GO SUB"),
            Goto1 => write!(f, "GOTO"),
            Goto2 => write!(f, "GO TO"),
            If => write!(f, "IF"),
            Input => write!(f, "INPUT"),
            Let => write!(f, "LET"),
            Next => write!(f, "NEXT"),
            On => write!(f, "ON"),
            Print1 => write!(f, "PRINT"),
            Print2 => write!(f, "?"),
            Read => write!(f, "READ"),
            Rem1 => write!(f, "REM"),
            Rem2 => write!(f, "'"),
            Restore => write!(f, "RESTORE"),
            Return => write!(f, "RETURN"),
            Stop => write!(f, "STOP"),
            Then => write!(f, "THEN"),
            To => write!(f, "TO"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, EnumIter)]
pub enum Operator {
    Equals,
    Plus,
    Minus,
    Multiply,
    Divide,
    DivideInt,
    Caret,
    Modulus,
    Not,
    And,
    Or,
    Xor,
    Eqv,
    Imp,
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Operator::*;
        match self {
            Equals => write!(f, "="),
            Plus => write!(f, "+"),
            Minus => write!(f, "-"),
            Multiply => write!(f, "*"),
            Divide => write!(f, "/"),
            DivideInt => write!(f, "\\"),
            Caret => write!(f, "^"),
            Modulus => write!(f, "MOD"),
            Not => write!(f, "NOT"),
            And => write!(f, "AND"),
            Or => write!(f, "OR"),
            Xor => write!(f, "XOR"),
            Eqv => write!(f, "EQV"),
            Imp => write!(f, "IMP"),
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
