use super::Val;
use crate::error;
use crate::lang::Error;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::rc::Rc;

type Result<T> = std::result::Result<T, Error>;

/// ## Variable memory

#[derive(Debug, Default)]
pub struct Var {
    vars: HashMap<Rc<str>, Val>,
    dims: HashMap<Rc<str>, Vec<i16>>,
}

impl Var {
    pub fn new() -> Var {
        Var::default()
    }

    pub fn clear(&mut self) {
        self.vars.clear();
        self.dims.clear();
    }

    pub fn fetch(&self, var_name: &Rc<str>) -> Val {
        match self.vars.get(var_name) {
            Some(val) => val.clone(),
            None => {
                if var_name.ends_with('$') {
                    Val::String("".into())
                } else if var_name.ends_with('!') {
                    Val::Single(0.0)
                } else if var_name.ends_with('#') {
                    Val::Double(0.0)
                } else if var_name.ends_with('%') {
                    Val::Integer(0)
                } else {
                    Val::Single(0.0)
                }
            }
        }
    }

    pub fn store_array(&mut self, var_name: &Rc<str>, arr: Vec<Val>, value: Val) -> Result<()> {
        let key = self.build_array_key(var_name, arr)?;
        self.store(&key, value)
    }

    pub fn fetch_array(&mut self, var_name: &Rc<str>, arr: Vec<Val>) -> Result<Val> {
        let key = self.build_array_key(var_name, arr)?;
        Ok(self.fetch(&key))
    }

    pub fn dimension_array(&mut self, var_name: &Rc<str>, arr: Vec<Val>) -> Result<()> {
        if self.dims.contains_key(var_name) {
            return Err(error!(RedimensionedArray));
        }
        let vi = self.vec_val_to_vec_i16(arr)?;
        self.dims.insert(var_name.clone(), vi);
        Ok(())
    }

    fn build_array_key(&mut self, var_name: &Rc<str>, arr: Vec<Val>) -> Result<Rc<str>> {
        let requested = self.vec_val_to_vec_i16(arr)?;
        let dimensioned = match self.dims.get(var_name) {
            Some(v) => v,
            None => self
                .dims
                .entry(var_name.clone())
                .or_insert_with(|| vec![10]),
        };
        if dimensioned.len() != requested.len() {
            return Err(error!(SubscriptOutOfRange));
        }
        for (r, d) in requested.iter().zip(dimensioned) {
            if r > d {
                return Err(error!(SubscriptOutOfRange));
            }
        }
        let mut s: String = requested.iter().map(|r| format!(",{}", r)).collect();
        s.push_str(&format!(",{}", var_name));
        Ok(s.into())
    }

    fn vec_val_to_vec_i16(&self, mut arr: Vec<Val>) -> Result<Vec<i16>> {
        let mut vec_i16: Vec<i16> = vec![];
        for v in arr.drain(..) {
            match i16::try_from(v) {
                Ok(i) => {
                    if i < 0 {
                        return Err(error!(SubscriptOutOfRange));
                    }
                    vec_i16.push(i)
                }
                Err(e) => return Err(e),
            }
        }
        Ok(vec_i16)
    }

    pub fn store(&mut self, var_name: &Rc<str>, value: Val) -> Result<()> {
        if self.vars.len() > u16::max_value() as usize {
            return Err(error!(OutOfMemory));
        }
        if var_name.ends_with('$') {
            self.insert_string(var_name, value)
        } else if var_name.ends_with('!') {
            self.insert_single(var_name, value)
        } else if var_name.ends_with('#') {
            self.insert_double(var_name, value)
        } else if var_name.ends_with('%') {
            self.insert_integer(var_name, value)
        } else {
            self.insert_single(var_name, value)
        }
    }

    fn update_val(&mut self, var_name: &Rc<str>, value: Val) {
        if match &value {
            Val::String(s) => s.is_empty(),
            Val::Integer(n) => *n == 0,
            Val::Single(n) => *n == 0.0,
            Val::Double(n) => *n == 0.0,
            Val::Return(_) | Val::Next(_) => false,
        } {
            self.vars.remove(var_name);
        } else {
            match self.vars.get_mut(var_name) {
                Some(var) => *var = value,
                None => {
                    self.vars.insert(var_name.clone(), value);
                }
            };
        }
    }

    fn insert_string(&mut self, var_name: &Rc<str>, value: Val) -> Result<()> {
        match &value {
            Val::String(s) => {
                if s.chars().count() > 255 {
                    return Err(error!(StringTooLong; "MAXIMUM STRING LENGTH IS 255"));
                }
                self.update_val(var_name, value);
                Ok(())
            }
            _ => Err(error!(TypeMismatch)),
        }
    }

    fn insert_integer(&mut self, var_name: &Rc<str>, value: Val) -> Result<()> {
        match value {
            Val::Integer(_) => self.update_val(var_name, value),
            _ => self.update_val(var_name, Val::Integer(i16::try_from(value)?)),
        }
        Ok(())
    }

    fn insert_single(&mut self, var_name: &Rc<str>, value: Val) -> Result<()> {
        match value {
            Val::Single(_) => self.update_val(var_name, value),
            _ => self.update_val(var_name, Val::Single(f32::try_from(value)?)),
        }
        Ok(())
    }

    fn insert_double(&mut self, var_name: &Rc<str>, value: Val) -> Result<()> {
        match value {
            Val::Double(_) => self.update_val(var_name, value),
            _ => self.update_val(var_name, Val::Double(f64::try_from(value)?)),
        }
        Ok(())
    }
}
