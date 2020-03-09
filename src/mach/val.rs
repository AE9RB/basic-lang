use super::Address;
use crate::error;
use crate::lang::{Error, LineNumber, MaxValue};
use std::convert::TryFrom;

type Result<T> = std::result::Result<T, Error>;

/// ## Runtime stack values

#[derive(Debug, Clone)]
pub enum Val {
    String(String),
    Integer(i16),
    Single(f32),
    Double(f64),
    Char(char),
    Return(Address),
}

impl Val {
    pub fn neg(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(l) => Ok(Integer(-l)),
            Single(l) => Ok(Single(-l)),
            Double(l) => Ok(Double(-l)),
            String(_) | Char(_) | Return(_) => Err(error!(TypeMismatch)),
        }
    }
    pub fn add(lhs: Val, rhs: Val) -> Result<Val> {
        use Val::*;
        loop {
            return match lhs {
                String(mut l) => match rhs {
                    String(r) => Ok(String(l + &r)),
                    Char(r) => Ok(String({
                        l.push(r);
                        l
                    })),
                    _ => break,
                },
                Char(l) => match rhs {
                    String(r) => Ok(String(l.to_string() + &r)),
                    Char(r) => Ok(String({
                        let mut l = l.to_string();
                        l.push(r);
                        l
                    })),
                    _ => break,
                },
                Integer(l) => match rhs {
                    Integer(r) => Ok(Integer(l + r)),
                    Single(r) => Ok(Single(l as f32 + r)),
                    Double(r) => Ok(Double(l as f64 + r)),
                    _ => break,
                },
                Single(l) => match rhs {
                    Integer(r) => Ok(Single(l + r as f32)),
                    Single(r) => Ok(Single(l + r)),
                    Double(r) => Ok(Double(l as f64 + r)),
                    _ => break,
                },
                Double(l) => match rhs {
                    Integer(r) => Ok(Double(l + r as f64)),
                    Single(r) => Ok(Double(l + r as f64)),
                    Double(r) => Ok(Double(l + r)),
                    _ => break,
                },
                Return(_) => break,
            };
        }
        Err(error!(TypeMismatch))
    }
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
            Return(..) => {
                debug_assert!(false);
                write!(f, "")
            }
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
            Err(error!(Overflow))
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
            Val::Char(_) | Val::String(_) | Val::Return(_) => Err(error!(TypeMismatch)),
        }
    }
}
