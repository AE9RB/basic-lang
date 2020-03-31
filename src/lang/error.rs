use super::{Column, LineNumber};
use std::sync::Arc;

#[derive(Clone)]
pub struct Error {
    code: u16,
    line_number: LineNumber,
    column: Column,
    message: Arc<str>,
}

#[doc(hidden)]
#[macro_export]
macro_rules! error {
    ($err:ident) => {
        $crate::lang::Error::new($crate::lang::ErrorCode::$err)
    };
    ($err:ident, ..$col:expr) => {
        $crate::lang::Error::new($crate::lang::ErrorCode::$err).in_column($col)
    };
    ($err:ident, $line:expr) => {
        $crate::lang::Error::new($crate::lang::ErrorCode::$err).in_line_number($line)
    };
    ($err:ident; $msg:expr) => {
        $crate::lang::Error::new($crate::lang::ErrorCode::$err).message($msg)
    };
    ($err:ident, ..$col:expr;  $msg:expr) => {
        $crate::lang::Error::new($crate::lang::ErrorCode::$err)
            .in_column($col)
            .message($msg)
    };
    ($err:ident, $line:expr, ..$col:expr) => {
        $crate::lang::Error::new($crate::lang::ErrorCode::$err)
            .in_line_number($line)
            .in_column($col)
    };
    ($err:ident, $line:expr; $msg:expr) => {
        $crate::lang::Error::new($crate::lang::ErrorCode::$err)
            .in_line_number($line)
            .message($msg)
    };
    ($err:ident, $line:expr, ..$col:expr;  $msg:expr) => {
        $crate::lang::Error::new($crate::lang::ErrorCode::$err)
            .in_line_number($line)
            .in_column($col)
            .message($msg)
    };
}

impl Error {
    pub fn new(code: ErrorCode) -> Error {
        Error {
            code: code as u16,
            line_number: None,
            column: 0..0,
            message: "".into(),
        }
    }

    pub fn is_direct(&self) -> bool {
        self.line_number.is_none()
    }

    pub fn line_number(&self) -> LineNumber {
        self.line_number
    }

    pub fn in_line_number(&self, line: LineNumber) -> Error {
        debug_assert!(self.line_number.is_none());
        Error {
            code: self.code,
            line_number: line,
            column: self.column.clone(),
            message: self.message.clone(),
        }
    }

    pub fn column(&self) -> Column {
        match self.line_number {
            Some(num) => {
                let offset = num.to_string().len() + 1;
                (self.column.start + offset)..(self.column.end + offset)
            }
            None => self.column.clone(),
        }
    }

    pub fn in_column(&self, column: &Column) -> Error {
        debug_assert_eq!(self.column, 0..0);
        Error {
            code: self.code,
            line_number: self.line_number,
            column: column.clone(),
            message: self.message.clone(),
        }
    }

    pub fn message(&self, message: &str) -> Error {
        debug_assert_eq!(self.message.len(), 0);
        Error {
            code: self.code,
            line_number: self.line_number,
            column: self.column.clone(),
            message: message.into(),
        }
    }
}

pub enum ErrorCode {
    Break = 0,
    NextWithoutFor = 1,
    SyntaxError = 2,
    ReturnWithoutGosub = 3,
    OutOfData = 4,
    IllegalFunctionCall = 5,
    Overflow = 6,
    OutOfMemory = 7,
    UndefinedLine = 8,
    SubscriptOutOfRange = 9,
    RedimensionedArray = 10,
    DivisionByZero = 11,
    IllegalDirect = 12,
    TypeMismatch = 13,
    OutOfStringSpace = 14,
    StringTooLong = 15,
    CantContinue = 17,
    UndefinedUserFunction = 18,
    RedoFromStart = 21,
    LineBufferOverflow = 23,
    ForWithoutNext = 26,
    InternalError = 51,
    FileNotFound = 53,
    DirectStatementInFile = 66,
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error {{ {} }}", self.to_string())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let code_str = match self.code {
            0 => "BREAK",
            1 => "NEXT WITHOUT FOR",
            2 => "SYNTAX ERROR",
            3 => "RETURN WITHOUT GOSUB",
            4 => "OUT OF DATA",
            5 => "ILLEGAL FUNCTION CALL",
            6 => "OVERFLOW",
            7 => "OUT OF MEMORY",
            8 => "UNDEFINED LINE",
            9 => "SUBSCRIPT OUT OF RANGE",
            10 => "REDIMENSIONED ARRAY",
            11 => "DIVISION BY ZERO",
            12 => "ILLEGAL DIRECT",
            13 => "TYPE MISMATCH",
            14 => "OUT OF STRING SPACE",
            15 => "STRING TOO LONG",
            16 => "STRING FORMULA TOO COMPLEX",
            17 => "CAN'T CONTINUE",
            18 => "UNDEFINED USER FUNCTION",
            19 => "NO RESUME",
            20 => "RESUME WITHOUT ERROR",
            21 => "REDO FROM START",
            22 => "MISSING OPERAND",
            23 => "LINE BUFFER OVERFLOW",
            26 => "FOR WITHOUT NEXT",
            29 => "WHILE WITHOUT WEND",
            30 => "WEND WITHOUT WHILE",
            50 => "FIELD OVERFLOW",
            51 => "INTERNAL ERROR",
            52 => "BAD FILE NUMBER",
            53 => "FILE NOT FOUND",
            54 => "BAD FILE MODE",
            55 => "FILE ALREADY OPEN",
            56 => "DISK NOT MOUNTED",
            57 => "DISK I/O ERROR",
            58 => "FILE ALREADY EXISTS",
            59 => "SET TO NON-DISK STRING",
            60 => "DISK ALREADY MOUNTED",
            61 => "DISK FULL",
            62 => "INPUT PAST END",
            63 => "BAD RECORD NUMBER",
            64 => "BAD FILE NAME",
            65 => "MODE-MISMATCH",
            66 => "DIRECT STATEMENT IN FILE",
            67 => "TOO MANY FILES",
            68 => "OUT OF RANDOM BLOCKS",
            _ => "",
        };
        let mut suffix = String::new();
        if let Some(line_number) = self.line_number {
            suffix.push_str(&format!(" {}", line_number));
        }
        if (0..0) != self.column {
            if suffix.is_empty() {
                suffix.push(' ');
            }
            suffix.push_str(&format!(":{}", self.column().start + 1));
        }
        if !suffix.is_empty() {
            suffix.insert_str(0, " IN");
        }
        if !self.message.is_empty() {
            suffix.push_str(&format!("; {}", self.message));
        }
        if code_str.is_empty() {
            write!(f, "?PROGRAM ERROR {}{}", self.code, suffix)
        } else {
            write!(f, "?{}{}", code_str, suffix)
        }
    }
}
