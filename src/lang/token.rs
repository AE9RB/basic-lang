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
            ("RESTORE", Token::Word(Word::Restore)),
            ("DEFDBL", Token::Word(Word::Defdbl)),
            ("DEFINT", Token::Word(Word::Defint)),
            ("DEFSNG", Token::Word(Word::Defsng)),
            ("DEFSTR", Token::Word(Word::Defstr)),
            ("DELETE", Token::Word(Word::Delete)),
            ("RETURN", Token::Word(Word::Return)),
            ("CLEAR", Token::Word(Word::Clear)),
            ("ERASE", Token::Word(Word::Erase)),
            ("GOSUB", Token::Word(Word::Gosub)),
            ("INPUT", Token::Word(Word::Input)),
            ("PRINT", Token::Word(Word::Print)),
            ("RENUM", Token::Word(Word::Renum)),
            ("TROFF", Token::Word(Word::Troff)),
            ("WHILE", Token::Word(Word::While)),
            ("CONT", Token::Word(Word::Cont)),
            ("DATA", Token::Word(Word::Data)),
            ("ELSE", Token::Word(Word::Else)),
            ("GOTO", Token::Word(Word::Goto)),
            ("NEXT", Token::Word(Word::Next)),
            ("LIST", Token::Word(Word::List)),
            ("LOAD", Token::Word(Word::Load)),
            ("READ", Token::Word(Word::Read)),
            ("SAVE", Token::Word(Word::Save)),
            ("STEP", Token::Word(Word::Step)),
            ("STOP", Token::Word(Word::Stop)),
            ("SWAP", Token::Word(Word::Swap)),
            ("THEN", Token::Word(Word::Then)),
            ("TRON", Token::Word(Word::Tron)),
            ("WEND", Token::Word(Word::Wend)),
            ("AND", Token::Operator(Operator::And)),
            ("CLS", Token::Word(Word::Cls)),
            ("DEF", Token::Word(Word::Def)),
            ("DIM", Token::Word(Word::Dim)),
            ("END", Token::Word(Word::End)),
            ("EQV", Token::Operator(Operator::Eqv)),
            ("FOR", Token::Word(Word::For)),
            ("IMP", Token::Operator(Operator::Imp)),
            ("LET", Token::Word(Word::Let)),
            ("MOD", Token::Operator(Operator::Modulo)),
            ("NEW", Token::Word(Word::New)),
            ("NOT", Token::Operator(Operator::Not)),
            ("REM", Token::Word(Word::Rem1)),
            ("RUN", Token::Word(Word::Run)),
            ("XOR", Token::Operator(Operator::Xor)),
            ("IF", Token::Word(Word::If)),
            ("ON", Token::Word(Word::On)),
            ("OR", Token::Operator(Operator::Or)),
            ("TO", Token::Word(Word::To)),
        ]
        .iter()
        .filter_map(|(ts, tk)| {
            s.find(ts).map(|idx| (idx, ts.len(), tk.clone()))
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

    pub fn match_minutia(s: &str) -> Option<Token> {
        match s {
            "(" => Some(Token::LParen),
            ")" => Some(Token::RParen),
            "," => Some(Token::Comma),
            ":" => Some(Token::Colon),
            ";" => Some(Token::Semicolon),
            "?" => Some(Token::Word(Word::Print)),
            "'" => Some(Token::Word(Word::Rem2)),
            "^" => Some(Token::Operator(Operator::Caret)),
            "*" => Some(Token::Operator(Operator::Multiply)),
            "/" => Some(Token::Operator(Operator::Divide)),
            "\\" => Some(Token::Operator(Operator::DivideInt)),
            "+" => Some(Token::Operator(Operator::Plus)),
            "-" => Some(Token::Operator(Operator::Minus)),
            "=" => Some(Token::Operator(Operator::Equal)),
            "<" => Some(Token::Operator(Operator::Less)),
            ">" => Some(Token::Operator(Operator::Greater)),
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
                Literal::Hex(_) | Literal::Octal(_) | Literal::String(_) => "",
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
    Hex(String),
    Octal(String),
    String(String),
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Literal::*;
        match self {
            Single(s) => write!(f, "{}", s),
            Double(s) => write!(f, "{}", s),
            Integer(s) => write!(f, "{}", s),
            Hex(s) => write!(f, "&H{}", s),
            Octal(s) => write!(f, "&{}", s),
            String(s) => write!(f, "\"{}\"", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Word {
    Clear,
    Cls,
    Cont,
    Data,
    Def,
    Defdbl,
    Defint,
    Defsng,
    Defstr,
    Delete,
    Dim,
    Else,
    End,
    Erase,
    For,
    Gosub,
    Goto,
    If,
    Input,
    Let,
    List,
    Load,
    New,
    Next,
    On,
    Print,
    Read,
    Rem1,
    Rem2,
    Renum,
    Restore,
    Return,
    Save,
    Step,
    Stop,
    Swap,
    Run,
    Then,
    To,
    Troff,
    Tron,
    Wend,
    While,
}

impl std::fmt::Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Word::*;
        match self {
            Clear => write!(f, "CLEAR"),
            Cls => write!(f, "CLS"),
            Cont => write!(f, "CONT"),
            Data => write!(f, "DATA"),
            Def => write!(f, "DEF"),
            Defdbl => write!(f, "DEFDBL"),
            Defint => write!(f, "DEFINT"),
            Defsng => write!(f, "DEFSNG"),
            Defstr => write!(f, "DEFSTR"),
            Delete => write!(f, "DELETE"),
            Dim => write!(f, "DIM"),
            Else => write!(f, "ELSE"),
            End => write!(f, "END"),
            Erase => write!(f, "ERASE"),
            For => write!(f, "FOR"),
            Gosub => write!(f, "GOSUB"),
            Goto => write!(f, "GOTO"),
            If => write!(f, "IF"),
            Input => write!(f, "INPUT"),
            Let => write!(f, "LET"),
            List => write!(f, "LIST"),
            Load => write!(f, "LOAD"),
            New => write!(f, "NEW"),
            Next => write!(f, "NEXT"),
            On => write!(f, "ON"),
            Print => write!(f, "PRINT"),
            Read => write!(f, "READ"),
            Rem1 => write!(f, "REM"),
            Rem2 => write!(f, "'"),
            Renum => write!(f, "RENUM"),
            Restore => write!(f, "RESTORE"),
            Return => write!(f, "RETURN"),
            Run => write!(f, "RUN"),
            Save => write!(f, "SAVE"),
            Step => write!(f, "STEP"),
            Stop => write!(f, "STOP"),
            Swap => write!(f, "SWAP"),
            Then => write!(f, "THEN"),
            To => write!(f, "TO"),
            Troff => write!(f, "TROFF"),
            Tron => write!(f, "TRON"),
            Wend => write!(f, "WEND"),
            While => write!(f, "WHILE"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Caret,
    Multiply,
    Divide,
    DivideInt,
    Modulo,
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
            Modulo | Not | And | Or | Xor | Imp | Eqv => true,
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
            Modulo => write!(f, "MOD"),
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
