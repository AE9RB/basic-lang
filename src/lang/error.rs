#[derive(Debug, PartialEq)]
pub struct Error {
    code: u16,
    line: Option<u16>,
    column: std::ops::Range<usize>,
}

macro_rules! error {
    ($err:ident) => {
        Err($crate::lang::error::Error::new(
            $crate::lang::error::ErrorCode::$err,
        ))
    };
}

impl Error {
    pub fn new(code: ErrorCode) -> Error {
        Error {
            code: code as u16,
            line: None,
            column: 0..0,
        }
    }

    pub fn in_line_number(mut self, line: Option<u16>) -> Error {
        debug_assert!(self.line.is_none());
        self.line = line;
        self
    }

    pub fn in_column(mut self, column: &std::ops::Range<usize>) -> Error {
        debug_assert_eq!(self.column, 0..0);
        self.column = column.clone();
        self
    }

    pub fn column(&self) -> &std::ops::Range<usize> {
        &self.column
    }
}

#[repr(u16)]
pub enum ErrorCode {
    SyntaxError = 2,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self.code {
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
        let suffix = match self.line {
            None => format!(""),
            Some(_) => {
                if (0..0) == self.column {
                    format!(" IN {}", self.line.unwrap())
                } else {
                    format!(" IN {}:{}", self.line.unwrap(), self.column.start)
                }
            }
        };
        if s.len() > 0 {
            write!(f, "{}{}", s, suffix)
        } else {
            write!(f, "PROGRAM ERROR {}{}", self.code, suffix)
        }
    }
}
