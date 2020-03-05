use super::{compile, Address, Op, Symbol};
use crate::error;
use crate::lang::{Column, Error, Line, LineNumber};
use std::collections::{BTreeMap, HashMap};

/// ## Program memory

#[derive(Debug)]
pub struct Program {
    ops: Vec<Op>,
    errors: Vec<Error>,
    indirect_errors: Vec<Error>,
    direct_address: Address,
    current_symbol: Symbol,
    symbols: BTreeMap<Symbol, Address>,
    unlinked: HashMap<Address, (Column, Symbol)>,
    line_number: LineNumber,
}

impl Program {
    pub fn new() -> Program {
        Program {
            ops: vec![],
            errors: vec![],
            indirect_errors: vec![],
            direct_address: 0,
            current_symbol: 0,
            symbols: BTreeMap::new(),
            unlinked: HashMap::new(),
            line_number: None,
        }
    }
    pub fn error(&mut self, col: &Column, error: Error) {
        self.errors
            .push(error.in_line_number(self.line_number).in_column(col));
    }
    pub fn append(&mut self, ops: &mut Vec<Op>) {
        self.ops.append(ops)
    }
    pub fn push(&mut self, op: Op) {
        self.ops.push(op)
    }
    pub fn symbol_for_line_number(&mut self, line_number: LineNumber) -> Symbol {
        line_number.unwrap() as Symbol
    }
    pub fn symbol_for_here(&mut self) -> Symbol {
        self.current_symbol -= 1;
        self.symbols.insert(self.current_symbol, self.ops.len());
        self.current_symbol
    }
    pub fn link_next_op_to(&mut self, col: &Column, symbol: Symbol) {
        self.unlinked.insert(self.ops.len(), (col.clone(), symbol));
    }
    pub fn clear(&mut self) {
        self.ops.clear();
        self.errors.clear();
        self.indirect_errors.clear();
        self.direct_address = 0;
        self.current_symbol = 0;
        self.symbols.clear();
        self.unlinked.clear();
        self.line_number = None;
    }
    pub fn compile<'a, T: IntoIterator<Item = &'a Line>>(&mut self, lines: T) {
        let is_out_of_mem = |this: &Self| this.ops.len() > Address::max_value() as usize;
        if is_out_of_mem(self) {
            return;
        }
        let mut direct_seen = false;
        for line in lines {
            if let Some(line_number) = line.number() {
                debug_assert!(
                    self.direct_address == 0,
                    "Can't go back to indirect mode without clear()."
                );
                if let Some(self_line_number) = self.line_number {
                    debug_assert!(line_number > self_line_number, "Lines out of order.");
                }
            } else if self.line_number.is_some() {
                self.link();
                self.ensure_end();
            }
            self.line_number = line.number();
            if let Some(line_number) = self.line_number {
                self.symbols.insert(line_number as Symbol, self.ops.len());
            } else {
                debug_assert!(!direct_seen, "Can't handle multiple direct lines.");
                direct_seen = true;
                self.ops.drain(self.direct_address..);
                self.errors.clear();
            }
            let ast = match line.ast() {
                Ok(ast) => ast,
                Err(e) => {
                    self.errors.push(e);
                    continue;
                }
            };
            compile(self, &ast);
            if is_out_of_mem(self) {
                self.errors.push(error!(OutOfMemory, self.line_number));
                return;
            }
        }
    }

    fn ensure_end(&mut self) {
        fn ender(this: &mut Program) {
            match this.ops.last() {
                Some(Op::End) => {}
                _ => this.ops.push(Op::End),
            };
        }
        if self.direct_address == 0 {
            ender(self);
            self.indirect_errors.append(&mut self.errors);
            self.direct_address = self.ops.len();
        }
        ender(self);
    }

    pub fn link_direct(&mut self) -> Result<(Address, &Vec<Op>), &Vec<Error>> {
        self.link();
        if !self.errors.is_empty() {
            Err(&self.errors)
        } else {
            Ok((self.direct_address, &self.ops))
        }
    }

    pub fn link_indirect(&mut self) -> Result<(Address, &Vec<Op>), &Vec<Error>> {
        self.link();
        if !self.indirect_errors.is_empty() {
            Err(&self.indirect_errors)
        } else {
            Ok((0, &self.ops))
        }
    }

    fn link(&mut self) {
        self.ensure_end();
        for (op_addr, (col, symbol)) in std::mem::take(&mut self.unlinked) {
            let dest = self.symbols.get(&symbol);
            if dest.is_none() && symbol >= 0 {
                let error = error!(UndefinedLine, self.line_number_for(op_addr)).in_column(&col);
                if op_addr < self.direct_address {
                    self.indirect_errors.push(error);
                } else {
                    self.errors.push(error);
                }
                continue;
            }
            let dest = *dest.unwrap();
            if let Some(new_op) = match self.ops[op_addr] {
                Op::If(_) => Some(Op::If(dest)),
                Op::IfNot(_) => Some(Op::IfNot(dest)),
                Op::Jump(_) => Some(Op::Jump(dest)),
                _ => None,
            } {
                self.ops[op_addr] = new_op;
                continue;
            }
            panic!();
        }
        self.symbols = self.symbols.split_off(&0);
        self.current_symbol = 0;
    }

    pub fn line_number_for(&self, op_addr: Address) -> LineNumber {
        if op_addr < self.direct_address {
            for (line_number, symbol_addr) in self.symbols.range(0..).rev() {
                if op_addr >= *symbol_addr {
                    return Some(*line_number as u16);
                }
            }
        }
        None
    }
}
