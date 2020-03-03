use super::compile::*;
use super::op::Op;
use crate::lang::Error;
use crate::lang::Line;
use std::collections::HashMap;

pub struct Program {
    ops: Vec<Op>,
    line_number: Option<u16>,
    symbols: HashMap<String, usize>,
    unlinked: HashMap<String, usize>,
}

impl Program {
    pub fn new() -> Program {
        Program {
            ops: vec![],
            line_number: None,
            symbols: HashMap::new(),
            unlinked: HashMap::new(),
        }
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
    pub fn compile<'a, T: IntoIterator<Item = &'a Line>>(
        &mut self,
        lines: T,
    ) -> Result<(), Vec<Error>> {
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
            if let Some(number) = self.line_number {
                //symbols.push line label
            } else {
                // record watermark for direct statement rewind
            }

            compile(self, line)?
        }
        Ok(())
    }
}
