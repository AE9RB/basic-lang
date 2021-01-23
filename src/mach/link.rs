use super::{Address, Opcode, Operation, Stack, Symbol, Val};
use crate::error;
use crate::lang::{Column, Error, LineNumber, MaxValue};
use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use std::rc::Rc;

type Result<T> = std::result::Result<T, Error>;

/// ## Linkable object

#[derive(Debug, Clone)]
pub struct Link {
    current_symbol: Symbol,
    ops: Stack<Opcode>,
    data: Stack<Val>,
    data_pos: Address,
    direct_set: bool,
    symbols: BTreeMap<Symbol, (Address, Address)>,
    unlinked: HashMap<Address, (Column, Symbol)>,
    whiles: Vec<(bool, Column, Address, Symbol)>,
}

impl Default for Link {
    fn default() -> Self {
        Link {
            current_symbol: 0,
            ops: Stack::new("PROGRAM SIZE LIMIT EXCEEDED"),
            data: Stack::new("DATA SIZE LIMIT EXCEEDED"),
            data_pos: 0,
            direct_set: false,
            symbols: BTreeMap::default(),
            unlinked: HashMap::default(),
            whiles: Vec::default(),
        }
    }
}

impl Link {
    pub fn append(&mut self, mut link: Link) -> Result<()> {
        if self.direct_set && !link.data.is_empty() {
            return Err(error!(IllegalDirect));
        }
        let ops_addr_offset = self.ops.len();
        let data_addr_offset = self.data.len();
        let sym_offset = self.current_symbol;
        for (symbol, (ops_addr, data_addr)) in link.symbols {
            let mut symbol = symbol;
            if symbol < 0 {
                symbol += sym_offset
            }
            self.symbols.insert(
                symbol,
                (ops_addr + ops_addr_offset, data_addr + data_addr_offset),
            );
        }
        for (address, (col, symbol)) in link.unlinked {
            let mut symbol = symbol;
            if symbol < 0 {
                symbol += sym_offset
            }
            self.unlinked
                .insert(address + ops_addr_offset, (col.clone(), symbol));
        }
        for (kind, col, addr, sym) in link.whiles {
            self.whiles
                .push((kind, col, addr + ops_addr_offset, sym + sym_offset));
        }
        self.current_symbol += link.current_symbol;
        self.ops.append(&mut link.ops)?;
        self.data.append(&mut link.data)
    }

    pub fn push(&mut self, op: Opcode) -> Result<()> {
        self.ops.push(op)
    }

    pub fn transform_to_data(&mut self, col: &Column) -> Result<()> {
        if self.ops.len() == 1 {
            if let Some(Opcode::Literal(val)) = self.ops.drain(..).next() {
                self.data.push(val)?;
                return Ok(());
            }
        } else if self.ops.len() == 2 {
            let mut expr_link = self.ops.drain(..);
            if let Some(Opcode::Literal(val)) = expr_link.next() {
                if let Some(Opcode::Neg) = expr_link.next() {
                    self.data.push(Operation::negate(val)?)?;
                    return Ok(());
                }
            }
        }
        Err(error!(SyntaxError, ..col; "EXPECTED LITERAL"))
    }

    pub fn read_data(&mut self) -> Result<Val> {
        if let Some(val) = self.data.get(self.data_pos) {
            self.data_pos += 1;
            Ok(val.clone())
        } else {
            Err(error!(OutOfData))
        }
    }

    pub fn restore_data(&mut self, addr: Address) {
        self.data_pos = addr;
    }

    pub fn get(&self, addr: Address) -> Option<&Opcode> {
        self.ops.get(addr)
    }

    pub fn last(&self) -> Option<&Opcode> {
        self.ops.last()
    }

    pub fn drain<R>(&mut self, range: R) -> std::vec::Drain<'_, Opcode>
    where
        R: std::ops::RangeBounds<usize>,
    {
        self.ops.drain(range)
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    pub fn len(&self) -> usize {
        self.ops.len()
    }

    pub fn clear(&mut self) {
        self.current_symbol = 0;
        self.direct_set = false;
        self.ops.clear();
        self.data.clear();
        self.symbols.clear();
        self.unlinked.clear();
    }

    pub fn next_symbol(&mut self) -> Symbol {
        self.current_symbol -= 1;
        self.current_symbol
    }

    fn symbol_for_line_number(&mut self, line_number: LineNumber) -> Result<Symbol> {
        match line_number {
            Some(number) => Ok(number as Symbol),
            None => Err(error!(InternalError; "NO SYMBOL FOR LINE NUMBER")),
        }
    }

    pub fn push_def_fn(
        &mut self,
        col: Column,
        ident: Rc<str>,
        vars: Vec<Rc<str>>,
        expr_ops: Link,
    ) -> Result<()> {
        let len = Val::try_from(vars.len())?;
        self.push(Opcode::Literal(len))?;
        self.push(Opcode::Def(ident))?;
        let skip = self.next_symbol();
        self.push_jump(col, skip)?;
        for var in vars {
            self.push(Opcode::Pop(var))?;
        }
        self.append(expr_ops)?;
        self.push(Opcode::Return)?;
        self.push_symbol(skip);
        Ok(())
    }

    pub fn push_for(&mut self, col: Column) -> Result<()> {
        let next = self.next_symbol();
        self.unlinked.insert(self.ops.len(), (col, next));
        self.ops.push(Opcode::Literal(Val::Next(0)))?;
        self.push_symbol(next);
        Ok(())
    }

    pub fn push_gosub(&mut self, col: Column, line_number: LineNumber) -> Result<()> {
        let ret_sym = self.next_symbol();
        self.push_return_val(col.clone(), ret_sym)?;
        let line_number_sym = self.symbol_for_line_number(line_number)?;
        self.unlinked.insert(self.ops.len(), (col, line_number_sym));
        self.ops.push(Opcode::Jump(0))?;
        self.push_symbol(ret_sym);
        Ok(())
    }

    pub fn push_return_val(&mut self, col: Column, symbol: Symbol) -> Result<()> {
        self.unlinked.insert(self.ops.len(), (col, symbol));
        self.ops.push(Opcode::Literal(Val::Return(0)))
    }

    pub fn push_goto(&mut self, col: Column, line_number: LineNumber) -> Result<()> {
        let sym = self.symbol_for_line_number(line_number)?;
        self.unlinked.insert(self.ops.len(), (col, sym));
        self.ops.push(Opcode::Jump(0))
    }

    pub fn push_ifnot(&mut self, col: Column, sym: Symbol) -> Result<()> {
        self.unlinked.insert(self.ops.len(), (col, sym));
        self.push(Opcode::IfNot(0))
    }

    pub fn push_jump(&mut self, col: Column, sym: Symbol) -> Result<()> {
        self.unlinked.insert(self.ops.len(), (col, sym));
        self.push(Opcode::Jump(0))
    }

    pub fn push_restore(&mut self, col: Column, line_number: LineNumber) -> Result<()> {
        if line_number.is_some() {
            let sym = self.symbol_for_line_number(line_number)?;
            self.unlinked.insert(self.ops.len(), (col, sym));
        }
        self.ops.push(Opcode::Restore(0))
    }

    pub fn push_run(&mut self, col: Column, line_number: LineNumber) -> Result<()> {
        self.ops.push(Opcode::Clear)?;
        if line_number.is_some() {
            let sym = self.symbol_for_line_number(line_number)?;
            self.unlinked.insert(self.ops.len(), (col, sym));
        }
        self.ops.push(Opcode::Jump(0))
    }

    pub fn push_symbol(&mut self, sym: Symbol) {
        if self
            .symbols
            .insert(sym, (self.ops.len(), self.data.len()))
            .is_some()
        {
            debug_assert!(false, "Symbol already exists.");
        }
    }

    pub fn push_wend(&mut self, col: Column) -> Result<()> {
        let sym = self.next_symbol();
        let addr = self.ops.len();
        self.whiles.push((false, col, addr, sym));
        self.push(Opcode::Jump(0))?;
        self.push_symbol(sym);
        Ok(())
    }

    pub fn push_while(&mut self, col: Column, expr: Link) -> Result<()> {
        let sym = self.next_symbol();
        self.push_symbol(sym);
        self.append(expr)?;
        self.whiles.push((true, col, self.ops.len(), sym));
        self.push(Opcode::IfNot(0))
    }

    pub fn set_start_of_direct(&mut self, op_addr: Address) {
        self.direct_set = true;
        self.symbols.insert(
            LineNumber::max_value() as Symbol + 1,
            (op_addr, self.data.len()),
        );
    }

    pub fn line_number_for(&self, op_addr: Address) -> LineNumber {
        for (line_number, (symbol_addr, _)) in self.symbols.range(0..).rev() {
            if op_addr >= *symbol_addr {
                if *line_number <= LineNumber::max_value() as isize {
                    return Some(*line_number as u16);
                } else {
                    return None;
                }
            }
        }
        None
    }

    fn link_whiles(&mut self) -> Vec<Error> {
        let mut errors: Vec<Error> = vec![];
        let mut whiles: Vec<(Column, Address, Symbol)> = Vec::default();
        for (kind, col, addr, sym) in std::mem::take(&mut self.whiles).drain(..) {
            if kind {
                whiles.push((col, addr, sym));
                continue;
            }
            match whiles.pop() {
                None => errors.push(error!(WendWithoutWhile, self.line_number_for(addr), ..&col)),
                Some((wh_col, wh_addr, wh_sym)) => {
                    self.unlinked.insert(wh_addr, (wh_col.clone(), sym));
                    self.unlinked.insert(addr, (col, wh_sym));
                }
            }
        }
        while let Some((col, addr, _)) = whiles.pop() {
            errors.push(error!(WhileWithoutWend, self.line_number_for(addr), ..&col));
        }
        errors
    }

    pub fn link(&mut self) -> Vec<Error> {
        let mut errors = self.link_whiles();
        for (op_addr, (col, symbol)) in std::mem::take(&mut self.unlinked) {
            match self.symbols.get(&symbol) {
                None => {
                    if symbol >= 0 {
                        let error = error!(UndefinedLine, self.line_number_for(op_addr), ..&col);
                        errors.push(error);
                        continue;
                    }
                }
                Some((op_dest, data_dest)) => {
                    if let Some(op) = self.ops.get_mut(op_addr) {
                        if let Some(new_op) = match op {
                            Opcode::IfNot(_) => Some(Opcode::IfNot(*op_dest)),
                            Opcode::Jump(_) => Some(Opcode::Jump(*op_dest)),
                            Opcode::Literal(Val::Return(_)) => {
                                Some(Opcode::Literal(Val::Return(*op_dest)))
                            }
                            Opcode::Literal(Val::Next(_)) => {
                                Some(Opcode::Literal(Val::Next(*op_dest)))
                            }
                            Opcode::Restore(_) => Some(Opcode::Restore(*data_dest)),
                            _ => None,
                        } {
                            *op = new_op;
                            continue;
                        }
                    }
                }
            }
            let line_number = self.line_number_for(op_addr);
            errors.push(error!(InternalError, line_number, ..&col; "LINK FAILURE"));
        }
        self.symbols = self.symbols.split_off(&0);
        self.current_symbol = 0;
        errors
    }
}

impl TryFrom<&Link> for LineNumber {
    type Error = Error;

    fn try_from(prog: &Link) -> std::result::Result<Self, Self::Error> {
        if prog.ops.len() == 1 {
            if let Some(Opcode::Literal(val)) = prog.ops.last() {
                return Ok(LineNumber::try_from(val.clone())?);
            }
        }
        Err(error!(UndefinedLine; "INVALID LINE NUMBER"))
    }
}

impl TryFrom<&Link> for Rc<str> {
    type Error = Error;

    fn try_from(prog: &Link) -> std::result::Result<Self, Self::Error> {
        if prog.ops.len() == 1 {
            if let Some(Opcode::Literal(Val::String(s))) = prog.ops.last() {
                return Ok(s.clone());
            }
        }
        Err(error!(SyntaxError; "EXPECTED STRING LITERAL"))
    }
}
