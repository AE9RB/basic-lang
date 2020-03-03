use super::compile::*;
use super::op::Op;
use crate::lang::Error;
use crate::lang::Line;
use std::collections::BTreeMap;
use std::collections::HashMap;

pub type Symbol = isize;

#[derive(Debug)]
pub struct Program {
    ops: Vec<Op>,
    error: Vec<Error>,
    line_number: Option<u16>,
    current_symbol: Symbol,
    symbols: BTreeMap<Symbol, usize>,
    unlinked: HashMap<usize, Symbol>,
}

impl Program {
    pub fn new() -> Program {
        Program {
            ops: vec![],
            error: vec![],
            line_number: None,
            current_symbol: 0,
            symbols: BTreeMap::new(),
            unlinked: HashMap::new(),
        }
    }
    pub fn error_push(&mut self, error: Error) {
        self.error.push(error);
    }
    pub fn append(&mut self, ops: &mut Vec<Op>) {
        self.ops.append(ops)
    }
    pub fn push(&mut self, op: Op) {
        self.ops.push(op)
    }
    pub fn len(&self) -> usize {
        self.ops.len()
    }
    pub fn ops(&self) -> &Vec<Op> {
        &self.ops
    }
    pub fn symbol_for_line_number(&mut self, line_number: u16) -> Symbol {
        line_number as Symbol
    }
    pub fn symbol_here(&mut self) -> Symbol {
        self.current_symbol -= 1;
        self.symbols.insert(self.current_symbol, self.ops.len());
        self.current_symbol
    }
    pub fn link_next_op_to(&mut self, symbol: Symbol) {
        self.unlinked.insert(self.ops.len(), symbol);
    }
    pub fn compile<'a, T: IntoIterator<Item = &'a Line>>(&mut self, lines: T) {
        for line in lines {
            if !self.ops.is_empty() {
                if let Some(line_number) = line.number() {
                    match self.line_number {
                        None => panic!("TODO need to rewind direct statement here"),
                        Some(current_number) => {
                            if line_number <= current_number {
                                panic!("TODO need to push error, lines out of order");
                            }
                        }
                    }
                }
            }
            self.line_number = line.number();
            if let Some(line_number) = self.line_number {
                self.symbols.insert(line_number as Symbol, self.ops.len());
            } else {
                // record watermark for direct statement rewind
            }

            compile(self, line)
        }
    }
    pub fn link(&mut self) {
        for (op_addr, symbol) in std::mem::take(&mut self.unlinked) {
            let dest = self.symbols.get(&symbol);
            if dest.is_none() && symbol >= 0 {
                self.error_push(error!(UndefinedLine).in_line_number(self.line_number(op_addr)));
                continue;
            }
            let dest = *dest.unwrap();
            if let Some(new_op) = match self.ops[op_addr] {
                Op::If(_) => Some(Op::If(dest)),
                Op::IfNot(_) => Some(Op::IfNot(dest)),
                Op::Goto(_) => Some(Op::Goto(dest)),
                _ => None,
            } {
                self.ops[op_addr] = new_op;
                continue;
            }
            panic!();
        }
        self.symbols = self.symbols.split_off(&0);
    }
    pub fn line_number(&self, op_addr: usize) -> Option<u16> {
        for (line_number, symbol_addr) in self.symbols.range(0..).rev() {
            if op_addr >= *symbol_addr {
                return Some(*line_number as u16);
            }
        }
        None
    }
}
