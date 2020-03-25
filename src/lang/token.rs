pub use super::ident::Ident;
use super::{Error, LineNumber, MaxValue};
use crate::error;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
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
        match s {
            "(" => Some(Token::LParen),
            ")" => Some(Token::RParen),
            "," => Some(Token::Comma),
            ":" => Some(Token::Colon),
            ";" => Some(Token::Semicolon),

            "CLEAR" => Some(Token::Word(Word::Clear)),
            "CONT" => Some(Token::Word(Word::Cont)),
            "DIM" => Some(Token::Word(Word::Dim)),
            "END" => Some(Token::Word(Word::End)),
            "ELSE" => Some(Token::Word(Word::Else)),
            "FOR" => Some(Token::Word(Word::For)),
            "GOSUB" => Some(Token::Word(Word::Gosub1)),
            "GO SUB" => Some(Token::Word(Word::Gosub2)),
            "GOTO" => Some(Token::Word(Word::Goto1)),
            "GO TO" => Some(Token::Word(Word::Goto2)),
            "IF" => Some(Token::Word(Word::If)),
            "INPUT" => Some(Token::Word(Word::Input)),
            "LET" => Some(Token::Word(Word::Let)),
            "LIST" => Some(Token::Word(Word::List)),
            "NEW" => Some(Token::Word(Word::New)),
            "NEXT" => Some(Token::Word(Word::Next)),
            "PRINT" => Some(Token::Word(Word::Print1)),
            "?" => Some(Token::Word(Word::Print2)),
            "REM" => Some(Token::Word(Word::Rem1)),
            "'" => Some(Token::Word(Word::Rem2)),
            "RETURN" => Some(Token::Word(Word::Return)),
            "RUN" => Some(Token::Word(Word::Run)),
            "STEP" => Some(Token::Word(Word::Step)),
            "STOP" => Some(Token::Word(Word::Stop)),
            "THEN" => Some(Token::Word(Word::Then)),
            "TO" => Some(Token::Word(Word::To)),

            "^" => Some(Token::Operator(Operator::Caret)),
            "*" => Some(Token::Operator(Operator::Multiply)),
            "/" => Some(Token::Operator(Operator::Divide)),
            "\\" => Some(Token::Operator(Operator::DivideInt)),
            "MOD" => Some(Token::Operator(Operator::Modulus)),
            "+" => Some(Token::Operator(Operator::Plus)),
            "-" => Some(Token::Operator(Operator::Minus)),
            "=" => Some(Token::Operator(Operator::Equal)),
            "<>" => Some(Token::Operator(Operator::NotEqual)),
            "<" => Some(Token::Operator(Operator::Less)),
            "<=" => Some(Token::Operator(Operator::LessEqual)),
            "=<" => Some(Token::Operator(Operator::EqualLess)),
            ">" => Some(Token::Operator(Operator::Greater)),
            ">=" => Some(Token::Operator(Operator::GreaterEqual)),
            "=>" => Some(Token::Operator(Operator::EqualGreater)),
            "NOT" => Some(Token::Operator(Operator::Not)),
            "AND" => Some(Token::Operator(Operator::And)),
            "OR" => Some(Token::Operator(Operator::Or)),
            "XOR" => Some(Token::Operator(Operator::Xor)),
            "IMP" => Some(Token::Operator(Operator::Imp)),
            "EQV" => Some(Token::Operator(Operator::Eqv)),

            _ => None,
        }
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
        Err(error!(UndefinedLine; msg))
    }
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum Word {
    Clear,
    Cont,
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
    List,
    New,
    Next,
    Print1,
    Print2,
    Rem1,
    Rem2,
    Return,
    Step,
    Stop,
    Run,
    Then,
    To,
}

impl std::fmt::Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Word::*;
        match self {
            Clear => write!(f, "CLEAR"),
            Cont => write!(f, "CONT"),
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
            List => write!(f, "LIST"),
            New => write!(f, "NEW"),
            Next => write!(f, "NEXT"),
            Print1 => write!(f, "PRINT"),
            Print2 => write!(f, "?"),
            Rem1 => write!(f, "REM"),
            Rem2 => write!(f, "'"),
            Return => write!(f, "RETURN"),
            Run => write!(f, "RUN"),
            Step => write!(f, "STEP"),
            Stop => write!(f, "STOP"),
            Then => write!(f, "THEN"),
            To => write!(f, "TO"),
        }
    }
}

#[derive(Debug, PartialEq)]
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
    EqualLess,
    Greater,
    GreaterEqual,
    EqualGreater,
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
            | LessEqual | EqualLess | Greater | GreaterEqual | EqualGreater => false,
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
            EqualLess => write!(f, "=<"),
            Greater => write!(f, ">"),
            GreaterEqual => write!(f, ">="),
            EqualGreater => write!(f, "=>"),
            Not => write!(f, "NOT"),
            And => write!(f, "AND"),
            Or => write!(f, "OR"),
            Xor => write!(f, "XOR"),
            Imp => write!(f, "IMP"),
            Eqv => write!(f, "EQV"),
        }
    }
}
