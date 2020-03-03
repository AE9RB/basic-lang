use crate::lang::Error;
use std::convert::TryFrom;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Val {
    String(String),
    Integer(i16),
    Single(f32),
    Double(f64),
    Char(char),
    Next(usize),
    Return(usize),
}

impl TryFrom<Val> for u16 {
    type Error = Error;
    // this is limited to valid line numbers
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
                if f >= 0.0 && f <= 65529.0 {
                    Ok(f as u16)
                } else {
                    Err(error!(Overflow))
                }
            }
            Val::Double(d) => {
                if d >= 0.0 && d <= 65529.0 {
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
