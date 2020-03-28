use super::{Error, LineNumber, MaxValue};
use crate::error;
use std::collections::VecDeque;
use std::convert::TryFrom;

#[derive(Debug, Clone, PartialEq)]
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
    pub fn scan_alphabetic(v: &mut VecDeque<Token>, mut s: &str) -> String {
        while let Some((idx, len, token)) = [
            ("RETURN", Token::Word(Word::Return)),
            ("CLEAR", Token::Word(Word::Clear)),
            ("INPUT", Token::Word(Word::Input)),
            ("PRINT", Token::Word(Word::Print1)),
            ("GOSUB", Token::Word(Word::Gosub1)),
            ("CONT", Token::Word(Word::Cont)),
            ("ELSE", Token::Word(Word::Else)),
            ("GOTO", Token::Word(Word::Goto1)),
            ("NEXT", Token::Word(Word::Next)),
            ("LIST", Token::Word(Word::List)),
            ("STEP", Token::Word(Word::Step)),
            ("STOP", Token::Word(Word::Stop)),
            ("THEN", Token::Word(Word::Then)),
            ("DEF", Token::Word(Word::Def)),
            ("DIM", Token::Word(Word::Dim)),
            ("END", Token::Word(Word::End)),
            ("FOR", Token::Word(Word::For)),
            ("XOR", Token::Operator(Operator::Xor)),
            ("IMP", Token::Operator(Operator::Imp)),
            ("EQV", Token::Operator(Operator::Eqv)),
            ("MOD", Token::Operator(Operator::Modulus)),
            ("NOT", Token::Operator(Operator::Not)),
            ("AND", Token::Operator(Operator::And)),
            ("LET", Token::Word(Word::Let)),
            ("NEW", Token::Word(Word::New)),
            ("REM", Token::Word(Word::Rem1)),
            ("RUN", Token::Word(Word::Run)),
            ("ON", Token::Word(Word::On)),
            ("TO", Token::Word(Word::To)),
            ("IF", Token::Word(Word::If)),
            ("OR", Token::Operator(Operator::Or)),
        ]
        .iter()
        .filter_map(|(ts, tk)| {
            if let Some(idx) = s.find(ts) {
                Some((idx, ts.len(), tk.clone()))
            } else {
                None
            }
        })
        .min_by_key(|(i, _, _)| *i)
        {
            if idx == 0 {
                v.push_back(token);
                s = &s[len..];
            } else {
                v.push_back(Token::Ident(Ident::Plain(s[..idx].into())));
                v.push_back(token);
                s = &s[(idx + len)..];
            }
        }
        s.to_string()
    }

    pub fn scan_minutia(s: &str) -> Option<Token> {
        match s {
            "(" => Some(Token::LParen),
            ")" => Some(Token::RParen),
            "," => Some(Token::Comma),
            ":" => Some(Token::Colon),
            ";" => Some(Token::Semicolon),
            "?" => Some(Token::Word(Word::Print2)),
            "'" => Some(Token::Word(Word::Rem2)),
            "^" => Some(Token::Operator(Operator::Caret)),
            "*" => Some(Token::Operator(Operator::Multiply)),
            "/" => Some(Token::Operator(Operator::Divide)),
            "\\" => Some(Token::Operator(Operator::DivideInt)),
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum Word {
    Clear,
    Cont,
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
    List,
    New,
    Next,
    On,
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
            List => write!(f, "LIST"),
            New => write!(f, "NEW"),
            Next => write!(f, "NEXT"),
            On => write!(f, "ON"),
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
