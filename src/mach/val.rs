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
        let mut s = match self {
            String(s) => return write!(f, "{}", s),
            Integer(num) => format!("{}", num),
            Single(num) => {
                let s = format!("{}", num);
                if s.chars().filter(char::is_ascii_digit).count() > 9 {
                    format!("{:E}", num)
                } else {
                    format!("{}", num)
                }
            }
            Double(num) => {
                let s = format!("{}", num);
                if s.chars().filter(char::is_ascii_digit).count() > 17 {
                    format!("{:E}", num)
                } else {
                    format!("{}", num)
                }
            }
            Return(..) | Next(..) => {
                debug_assert!(false);
                return write!(f, "");
            }
        };
        if !s.starts_with('-') {
            s.insert(0, ' ');
        }
        write!(f, "{}", s)
    }
}

impl TryFrom<LineNumber> for Val {
    type Error = Error;
    fn try_from(line_number: LineNumber) -> std::result::Result<Self, Self::Error> {
        match line_number {
            Some(num) => Ok(Val::Single(num as f32)),
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
            Val::Integer(num) => {
                if num >= 0 {
                    Ok(num as u16)
                } else {
                    Err(error!(Overflow))
                }
            }
            Val::Single(num) => {
                if num >= 0.0 && num <= u16::max_value() as f32 {
                    Ok(num as u16)
                } else {
                    Err(error!(Overflow))
                }
            }
            Val::Double(num) => {
                if num >= 0.0 && num <= u16::max_value() as f64 {
                    Ok(num as u16)
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
            Val::Integer(num) => Ok(num),
            Val::Single(num) => {
                if num >= i16::min_value() as f32 && num <= i16::max_value() as f32 {
                    Ok(num as i16)
                } else {
                    Err(error!(Overflow))
                }
            }
            Val::Double(num) => {
                if num >= i16::min_value() as f64 && num <= i16::max_value() as f64 {
                    Ok(num as i16)
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
            Val::Integer(num) => {
                if num >= 0 {
                    Ok(num as u32)
                } else {
                    Err(error!(Overflow))
                }
            }
            Val::Single(num) => {
                if num >= 0.0 && num <= u32::max_value() as f32 {
                    Ok(num as u32)
                } else {
                    Err(error!(Overflow))
                }
            }
            Val::Double(num) => {
                if num >= 0.0 && num <= u32::max_value() as f64 {
                    Ok(num as u32)
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
            Val::Integer(num) => {
                if num >= 0 {
                    Ok(num as usize)
                } else {
                    Err(error!(Overflow))
                }
            }
            Val::Single(num) => {
                if num >= 0.0 && num <= usize::max_value() as f32 {
                    Ok(num as usize)
                } else {
                    Err(error!(Overflow))
                }
            }
            Val::Double(num) => {
                if num >= 0.0 && num <= usize::max_value() as f64 {
                    Ok(num as usize)
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
            Val::Integer(num) => Ok(num as f32),
            Val::Single(num) => Ok(num),
            Val::Double(num) => Ok(num as f32),
            Val::String(_) | Val::Return(_) | Val::Next(..) => Err(error!(TypeMismatch)),
        }
    }
}

impl TryFrom<Val> for f64 {
    type Error = Error;
    fn try_from(val: Val) -> std::result::Result<Self, Self::Error> {
        match val {
            Val::Integer(num) => Ok(num as f64),
            Val::Single(num) => Ok(num as f64),
            Val::Double(num) => Ok(num),
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
            if num.fract() < std::f64::EPSILON
                && num <= i16::max_value() as f64
                && num >= i16::min_value() as f64
            {
                Val::Integer(num as i16)
            } else {
                Val::Double(num)
            }
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
