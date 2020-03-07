use super::{Column, LineNumber};

pub struct Error {
    code: u16,
    line_number: LineNumber,
    column: Column,
    message: &'static str,
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
            message: "",
        }
    }

    pub fn is_direct(&self) -> bool {
        self.line_number.is_none()
    }

    pub fn in_line_number(&self, line: LineNumber) -> Error {
        debug_assert!(self.line_number.is_none());
        Error {
            code: self.code,
            line_number: line,
            column: self.column.clone(),
            message: self.message,
        }
    }

    pub fn in_column(&self, column: &Column) -> Error {
        debug_assert_eq!(self.column, 0..0);
        Error {
            code: self.code,
            line_number: self.line_number,
            column: column.clone(),
            message: self.message,
        }
    }
    pub fn message(&self, message: &'static str) -> Error {
        debug_assert_eq!(message.len(), 0);
        Error {
            code: self.code,
            line_number: self.line_number,
            column: self.column.clone(),
            message: message,
        }
    }
}

pub enum ErrorCode {
    SyntaxError = 2,
    Overflow = 6,
    OutOfMemory = 7,
    UndefinedLine = 8,
    TypeMismatch = 13,
    InternalError = 51,
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error {{ {} }}", self.to_string())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let code_str = match self.code {
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
            21 => "UNPRINTABLE ERROR",
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
            suffix.push_str(&format!(" ({}..{})", self.column.start, self.column.end));
        }
        if !self.message.is_empty() {
            suffix.push_str(&format!("; {}", self.message));
        }
        if code_str.is_empty() {
            if suffix.is_empty() {
                write!(f, "PROGRAM ERROR {}", self.code)
            } else {
                write!(f, "PROGRAM ERROR {} IN{}", self.code, suffix)
            }
        } else {
            if suffix.is_empty() {
                write!(f, "{}", code_str)
            } else {
                write!(f, "{} IN{}", code_str, suffix)
            }
        }
    }
}
