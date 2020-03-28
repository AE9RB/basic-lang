use super::Address;
use crate::error;
use crate::lang::{Error, LineNumber, MaxValue};
use std::convert::TryFrom;
use std::rc::Rc;

/// ## Runtime values for stack and variables

#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    String(Rc<str>),
    Single(f32),
    Double(f64),
    Integer(i16),
    Return(Address),
    Next(Address),
}

impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Val::*;
        match self {
            String(s) => write!(f, "{}", s),
            Integer(n) => {
                if *n < 0 {
                    write!(f, "{}", n)
                } else {
                    write!(f, " {}", n)
                }
            }
            Single(n) => {
                if *n < 0.0 {
                    write!(f, "{}", n)
                } else {
                    write!(f, " {}", n)
                }
            }
            Double(n) => {
                if *n < 0.0 {
                    write!(f, "{}", n)
                } else {
                    write!(f, " {}", n)
                }
            }
            Return(..) | Next(..) => {
                debug_assert!(false);
                write!(f, "")
            }
        }
    }
}

impl TryFrom<LineNumber> for Val {
    type Error = Error;
    fn try_from(line_number: LineNumber) -> std::result::Result<Self, Self::Error> {
        match line_number {
            Some(number) => Ok(Val::Single(number as f32)),
            None => Err(error!(UndefinedLine)),
        }
    }
}

impl TryFrom<Val> for LineNumber {
    type Error = Error;
    fn try_from(val: Val) -> std::result::Result<Self, Self::Error> {
        let num = u16::try_from(val)?;
        if num <= LineNumber::max_value() {
            Ok(Some(num))
        } else {
            Err(error!(UndefinedLine))
        }
    }
}

impl TryFrom<Val> for u16 {
    type Error = Error;
    fn try_from(val: Val) -> std::result::Result<Self, Self::Error> {
        match val {
            Val::Integer(i) => {
                if i >= 0 {
                    Ok(i as u16)
                } else {
                    Err(error!(Overflow))
                }
            }
            Val::Single(f) => {
                if f >= 0.0 && f <= u16::max_value() as f32 {
                    Ok(f as u16)
                } else {
                    Err(error!(Overflow))
                }
            }
            Val::Double(d) => {
                if d >= 0.0 && d <= u16::max_value() as f64 {
                    Ok(d as u16)
                } else {
                    Err(error!(Overflow))
                }
            }
            Val::String(_) | Val::Return(_) | Val::Next(..) => Err(error!(TypeMismatch)),
        }
    }
}

impl TryFrom<Val> for i16 {
    type Error = Error;
    fn try_from(val: Val) -> std::result::Result<Self, Self::Error> {
        match val {
            Val::Integer(i) => Ok(i),
            Val::Single(f) => {
                if f >= i16::min_value() as f32 && f <= i16::max_value() as f32 {
                    Ok(f as i16)
                } else {
                    Err(error!(Overflow))
                }
            }
            Val::Double(d) => {
                if d >= i16::min_value() as f64 && d <= i16::max_value() as f64 {
                    Ok(d as i16)
                } else {
                    Err(error!(Overflow))
                }
            }
            Val::String(_) | Val::Return(_) | Val::Next(..) => Err(error!(TypeMismatch)),
        }
    }
}

impl TryFrom<Val> for u32 {
    type Error = Error;
    fn try_from(val: Val) -> std::result::Result<Self, Self::Error> {
        match val {
            Val::Integer(i) => {
                if i >= 0 {
                    Ok(i as u32)
                } else {
                    Err(error!(Overflow))
                }
            }
            Val::Single(f) => {
                if f >= 0.0 && f <= u32::max_value() as f32 {
                    Ok(f as u32)
                } else {
                    Err(error!(Overflow))
                }
            }
            Val::Double(d) => {
                if d >= 0.0 && d <= u32::max_value() as f64 {
                    Ok(d as u32)
                } else {
                    Err(error!(Overflow))
                }
            }
            Val::String(_) | Val::Return(_) | Val::Next(..) => Err(error!(TypeMismatch)),
        }
    }
}

impl TryFrom<Val> for usize {
    type Error = Error;
    fn try_from(val: Val) -> std::result::Result<Self, Self::Error> {
        match val {
            Val::Integer(i) => {
                if i >= 0 {
                    Ok(i as usize)
                } else {
                    Err(error!(Overflow))
                }
            }
            Val::Single(f) => {
                if f >= 0.0 && f <= usize::max_value() as f32 {
                    Ok(f as usize)
                } else {
                    Err(error!(Overflow))
                }
            }
            Val::Double(d) => {
                if d >= 0.0 && d <= usize::max_value() as f64 {
                    Ok(d as usize)
                } else {
                    Err(error!(Overflow))
                }
            }
            Val::String(_) | Val::Return(_) | Val::Next(..) => Err(error!(TypeMismatch)),
        }
    }
}

impl TryFrom<Val> for f32 {
    type Error = Error;
    fn try_from(val: Val) -> std::result::Result<Self, Self::Error> {
        match val {
            Val::Integer(i) => Ok(i as f32),
            Val::Single(f) => Ok(f),
            Val::Double(d) => Ok(d as f32),
            Val::String(_) | Val::Return(_) | Val::Next(..) => Err(error!(TypeMismatch)),
        }
    }
}

impl TryFrom<Val> for f64 {
    type Error = Error;
    fn try_from(val: Val) -> std::result::Result<Self, Self::Error> {
        match val {
            Val::Integer(i) => Ok(i as f64),
            Val::Single(f) => Ok(f as f64),
            Val::Double(d) => Ok(d),
            Val::String(_) | Val::Return(_) | Val::Next(..) => Err(error!(TypeMismatch)),
        }
    }
}

impl TryFrom<Val> for Rc<str> {
    type Error = Error;
    fn try_from(val: Val) -> std::result::Result<Self, Self::Error> {
        match val {
            Val::String(s) => Ok(s),
            _ => Err(error!(TypeMismatch)),
        }
    }
}

impl From<&str> for Val {
    fn from(string: &str) -> Self {
        let mut s = String::from(string).replace("D", "E").replace("d", "e");
        match s.chars().last() {
            Some('!') | Some('#') | Some('%') => {
                s.pop();
            }
            _ => {}
        };
        if let Ok(num) = s.parse::<f64>() {
            Val::Double(num)
        } else {
            Val::String(string.into())
        }
    }
}

impl TryFrom<usize> for Val {
    type Error = Error;
    fn try_from(num: usize) -> std::result::Result<Self, Self::Error> {
        match i16::try_from(num) {
            Ok(len) => Ok(Val::Integer(len)),
            Err(_) => {
                debug_assert!(false, "LEN VAL TOO BIG");
                Err(error!(Overflow))
            }
        }
    }
}
