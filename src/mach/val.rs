use super::Address;
use crate::error;
use crate::lang::{Error, LineNumber, MaxValue};
use std::convert::TryFrom;

/// ## Stack values

#[derive(Debug, Clone)]
pub enum Val {
    String(String),
    Integer(i16),
    Single(f32),
    Double(f64),
    Char(char),
    Return(Address),
}

impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Val::*;
        match self {
            String(s) => write!(f, "{}", s),
            Integer(n) => write!(f, "{}", n),
            Single(n) => write!(f, "{}", n),
            Double(n) => write!(f, "{}", n),
            Char(c) => write!(f, "{}", c),
            Return(..) => write!(f, "PANIC"),
        }
    }
}

impl TryFrom<Val> for LineNumber {
    type Error = Error;
    fn try_from(val: Val) -> Result<Self, Self::Error> {
        let num = u16::try_from(val)?;
        if num <= LineNumber::max_value() {
            Ok(Some(num))
        } else {
            Err(error!(Overflow))
        }
    }
}

impl TryFrom<Val> for u16 {
    type Error = Error;
    fn try_from(val: Val) -> Result<Self, Self::Error> {
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
            Val::Char(_) | Val::String(_) | Val::Return(_) => Err(error!(TypeMismatch)),
        }
    }
}
