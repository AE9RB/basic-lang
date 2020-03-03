use super::Line;

#[derive(Debug)]
pub struct Error {
    code: u16,
    line_number: Option<u16>,
    column: std::ops::Range<usize>,
}

macro_rules! error {
    ($err:ident) => {
        $crate::lang::Error::new($crate::lang::ErrorCode::$err)
    };
}

impl Error {
    pub fn new(code: ErrorCode) -> Error {
        Error {
            code: code as u16,
            line_number: None,
            column: 0..0,
        }
    }

    pub fn in_line(&self, line: &Line) -> Error {
        debug_assert!(self.line_number.is_none());
        Error {
            code: self.code,
            line_number: line.number(),
            column: self.column.clone(),
        }
    }

    pub fn in_line_number(&self, line: Option<u16>) -> Error {
        debug_assert!(self.line_number.is_none());
        Error {
            code: self.code,
            line_number: line,
            column: self.column.clone(),
        }
    }

    pub fn in_column(&self, column: &std::ops::Range<usize>) -> Error {
        debug_assert_eq!(self.column, 0..0);
        Error {
            code: self.code,
            line_number: self.line_number,
            column: column.clone(),
        }
    }

    pub fn column(&self) -> &std::ops::Range<usize> {
        &self.column
    }
}

pub enum ErrorCode {
    SyntaxError = 2,
    Overflow = 6,
    OutOfMemory = 7,
    UndefinedLine = 8,
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
        let suffix = match self.line_number {
            None => {
                if (0..0) == *self.column() {
                    format!("")
                } else {
                    format!(" IN {}..{}", self.column().start, self.column().end)
                }
            }
            Some(line_number) => {
                if (0..0) == *self.column() {
                    format!(" IN {}", line_number)
                } else {
                    format!(
                        " IN {}:{}..{}",
                        line_number,
                        self.column().start,
                        self.column().end
                    )
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
