use super::Address;
use crate::error;
use crate::lang::{Error, LineNumber, MaxValue};
use std::convert::TryFrom;

/// ## Stack values

#[derive(Debug, Clone)]
pub enum Val {
    Undefined,
    String(String),
    Integer(i16),
    Single(f32),
    Double(f64),
    Char(char),
    Next(Address),
    Return(Address),
}

impl TryFrom<Val> for LineNumber {
    type Error = Error;
    fn try_from(val: Val) -> Result<Self, Self::Error> {
        match u16::try_from(val) {
            Ok(num) => {
                if num <= LineNumber::max_value() {
                    Ok(Some(num))
                } else {
                    Err(error!(Overflow))
                }
            }
            Err(e) => Err(e),
        }
    }
}

impl TryFrom<Val> for u16 {
    type Error = Error;
    fn try_from(val: Val) -> Result<Self, Self::Error> {
        match val {
            Val::Undefined => Ok(0),
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
            Val::Char(_) | Val::String(_) | Val::Next(_) | Val::Return(_) => {
                Err(error!(SyntaxError))
            }
        }
    }
}
