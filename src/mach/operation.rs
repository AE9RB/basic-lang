use super::Val;
use crate::error;
use crate::lang::Error;

type Result<T> = std::result::Result<T, Error>;

pub struct Operation {}

impl Operation {
    pub fn unimplemented(_lhs: Val, _rhs: Val) -> Result<Val> {
        Err(error!(InternalError; "OP NOT IMPLEMENTED; PANIC"))
    }

    pub fn negate(val: Val) -> Result<Val> {
        use Val::*;
        match val {
            Integer(n) => Ok(Integer(-n)),
            Single(n) => Ok(Single(-n)),
            Double(n) => Ok(Double(-n)),
            String(_) | Return(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn multiply(lhs: Val, rhs: Val) -> Result<Val> {
        use Val::*;
        match lhs {
            Integer(l) => match rhs {
                Integer(r) => match l.checked_mul(r) {
                    Some(i) => Ok(Integer(i)),
                    None => Err(error!(Overflow)),
                },
                Single(r) => Ok(Single(l as f32 * r)),
                Double(r) => Ok(Double(l as f64 * r)),
                _ => Err(error!(TypeMismatch)),
            },
            Single(l) => match rhs {
                Integer(r) => Ok(Single(l * r as f32)),
                Single(r) => Ok(Single(l * r)),
                Double(r) => Ok(Double(l as f64 * r)),
                _ => Err(error!(TypeMismatch)),
            },
            Double(l) => match rhs {
                Integer(r) => Ok(Double(l * r as f64)),
                Single(r) => Ok(Double(l * r as f64)),
                Double(r) => Ok(Double(l * r)),
                _ => Err(error!(TypeMismatch)),
            },
            String(_) | Return(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn divide(lhs: Val, rhs: Val) -> Result<Val> {
        use Val::*;
        match lhs {
            Integer(l) => match rhs {
                Integer(r) => match l.checked_div(r) {
                    Some(i) => Ok(Integer(i)),
                    None => {
                        if r == 0 {
                            Err(error!(DivisionByZero))
                        } else {
                            Err(error!(Overflow))
                        }
                    }
                },
                Single(r) => Ok(Single(l as f32 / r)),
                Double(r) => Ok(Double(l as f64 / r)),
                _ => Err(error!(TypeMismatch)),
            },
            Single(l) => match rhs {
                Integer(r) => Ok(Single(l / r as f32)),
                Single(r) => Ok(Single(l / r)),
                Double(r) => Ok(Double(l as f64 / r)),
                _ => Err(error!(TypeMismatch)),
            },
            Double(l) => match rhs {
                Integer(r) => Ok(Double(l / r as f64)),
                Single(r) => Ok(Double(l / r as f64)),
                Double(r) => Ok(Double(l / r)),
                _ => Err(error!(TypeMismatch)),
            },
            String(_) | Return(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn sum(lhs: Val, rhs: Val) -> Result<Val> {
        use Val::*;
        match lhs {
            String(l) => match rhs {
                String(r) => Ok(String((l.to_string() + &r).into())),
                _ => Err(error!(TypeMismatch)),
            },
            Integer(l) => match rhs {
                Integer(r) => match l.checked_add(r) {
                    Some(i) => Ok(Integer(i)),
                    None => Err(error!(Overflow)),
                },
                Single(r) => Ok(Single(l as f32 + r)),
                Double(r) => Ok(Double(l as f64 + r)),
                _ => Err(error!(TypeMismatch)),
            },
            Single(l) => match rhs {
                Integer(r) => Ok(Single(l + r as f32)),
                Single(r) => Ok(Single(l + r)),
                Double(r) => Ok(Double(l as f64 + r)),
                _ => Err(error!(TypeMismatch)),
            },
            Double(l) => match rhs {
                Integer(r) => Ok(Double(l + r as f64)),
                Single(r) => Ok(Double(l + r as f64)),
                Double(r) => Ok(Double(l + r)),
                _ => Err(error!(TypeMismatch)),
            },
            Return(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn subtract(lhs: Val, rhs: Val) -> Result<Val> {
        use Val::*;
        match lhs {
            Integer(l) => match rhs {
                Integer(r) => match l.checked_sub(r) {
                    Some(i) => Ok(Integer(i)),
                    None => Err(error!(Overflow)),
                },
                Single(r) => Ok(Single(l as f32 - r)),
                Double(r) => Ok(Double(l as f64 - r)),
                _ => Err(error!(TypeMismatch)),
            },
            Single(l) => match rhs {
                Integer(r) => Ok(Single(l - r as f32)),
                Single(r) => Ok(Single(l - r)),
                Double(r) => Ok(Double(l as f64 - r)),
                _ => Err(error!(TypeMismatch)),
            },
            Double(l) => match rhs {
                Integer(r) => Ok(Double(l - r as f64)),
                Single(r) => Ok(Double(l - r as f64)),
                Double(r) => Ok(Double(l - r)),
                _ => Err(error!(TypeMismatch)),
            },
            String(_) | Return(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn equal(lhs: Val, rhs: Val) -> Result<Val> {
        if Operation::equal_bool(lhs, rhs)? {
            Ok(Val::Integer(-1))
        } else {
            Ok(Val::Integer(0))
        }
    }

    pub fn not_equal(lhs: Val, rhs: Val) -> Result<Val> {
        if Operation::equal_bool(lhs, rhs)? {
            Ok(Val::Integer(0))
        } else {
            Ok(Val::Integer(-1))
        }
    }

    fn equal_bool(lhs: Val, rhs: Val) -> Result<bool> {
        use Val::*;
        match lhs {
            Integer(l) => match rhs {
                Integer(r) => Ok(l == r),
                Single(r) => Ok((l as f32 - r).abs() < std::f32::EPSILON),
                Double(r) => Ok((l as f64 - r).abs() < std::f64::EPSILON),
                _ => Err(error!(TypeMismatch)),
            },
            Single(l) => match rhs {
                Integer(r) => Ok((l - r as f32).abs() < std::f32::EPSILON),
                Single(r) => Ok((l - r).abs() < std::f32::EPSILON),
                Double(r) => Ok((l as f64 - r).abs() < std::f64::EPSILON),
                _ => Err(error!(TypeMismatch)),
            },
            Double(l) => match rhs {
                Integer(r) => Ok((l - r as f64).abs() < std::f64::EPSILON),
                Single(r) => Ok((l - r as f64).abs() < std::f64::EPSILON),
                Double(r) => Ok((l - r).abs() < std::f64::EPSILON),
                _ => Err(error!(TypeMismatch)),
            },
            String(l) => match rhs {
                String(r) => Ok(l == r),
                _ => Err(error!(TypeMismatch)),
            },
            Return(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn greater(lhs: Val, rhs: Val) -> Result<Val> {
        if Operation::less_bool(rhs, lhs)? {
            Ok(Val::Integer(-1))
        } else {
            Ok(Val::Integer(0))
        }
    }

    pub fn less(lhs: Val, rhs: Val) -> Result<Val> {
        if Operation::less_bool(lhs, rhs)? {
            Ok(Val::Integer(-1))
        } else {
            Ok(Val::Integer(0))
        }
    }

    pub fn less_bool(lhs: Val, rhs: Val) -> Result<bool> {
        use Val::*;
        match lhs {
            Integer(l) => match rhs {
                Integer(r) => Ok(l < r),
                Single(r) => Ok((l as f32) < r),
                Double(r) => Ok((l as f64) < r),
                _ => Err(error!(TypeMismatch)),
            },
            Single(l) => match rhs {
                Integer(r) => Ok(l < r as f32),
                Single(r) => Ok(l < r),
                Double(r) => Ok((l as f64) < r),
                _ => Err(error!(TypeMismatch)),
            },
            Double(l) => match rhs {
                Integer(r) => Ok(l < r as f64),
                Single(r) => Ok(l < r as f64),
                Double(r) => Ok(l < r),
                _ => Err(error!(TypeMismatch)),
            },
            String(_) => Err(error!(InternalError; "TODO; PANIC")),
            Return(_) => Err(error!(TypeMismatch)),
        }
    }

    pub fn greater_equal(lhs: Val, rhs: Val) -> Result<Val> {
        if Operation::less_equal_bool(rhs, lhs)? {
            Ok(Val::Integer(-1))
        } else {
            Ok(Val::Integer(0))
        }
    }

    pub fn less_equal(lhs: Val, rhs: Val) -> Result<Val> {
        if Operation::less_equal_bool(lhs, rhs)? {
            Ok(Val::Integer(-1))
        } else {
            Ok(Val::Integer(0))
        }
    }

    pub fn less_equal_bool(lhs: Val, rhs: Val) -> Result<bool> {
        use Val::*;
        match lhs {
            Integer(l) => match rhs {
                Integer(r) => Ok(l <= r),
                Single(r) => Ok((l as f32) <= r),
                Double(r) => Ok((l as f64) <= r),
                _ => Err(error!(TypeMismatch)),
            },
            Single(l) => match rhs {
                Integer(r) => Ok(l <= r as f32),
                Single(r) => Ok(l <= r),
                Double(r) => Ok((l as f64) <= r),
                _ => Err(error!(TypeMismatch)),
            },
            Double(l) => match rhs {
                Integer(r) => Ok(l <= r as f64),
                Single(r) => Ok(l <= r as f64),
                Double(r) => Ok(l <= r),
                _ => Err(error!(TypeMismatch)),
            },
            String(_) => Err(error!(InternalError; "TODO; PANIC")),
            Return(_) => Err(error!(TypeMismatch)),
        }
    }
}
