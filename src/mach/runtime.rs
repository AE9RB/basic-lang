use super::{Op, Program, Val};
use crate::lang::{Line, LineNumber};
use std::collections::{BTreeMap, HashMap};

pub struct Runtime {
    source: BTreeMap<LineNumber, Line>,
    dirty: bool,
    program: Program,
    stack: Vec<Val>,
    vars: HashMap<String, Val>,
}

impl Runtime {
    pub fn new() -> Runtime {
        Runtime {
            source: BTreeMap::new(),
            dirty: false,
            program: Program::new(),
            stack: Vec::new(),
            vars: HashMap::new(),
        }
    }
    pub fn enter(&mut self, line: Line) {
        let direct = line.is_direct();
        self.source.insert(line.number(), line);
        if direct {
            if self.dirty {
                self.program.clear();
                let indirect_lines = self.source.range(Some(0)..).map(|(_, line)| line);
                self.program.compile(indirect_lines);
                self.dirty = false;
            }
            let direct_line = self.source.get(&None).unwrap();
            self.program.compile(direct_line);
        //self.execute();
        } else {
            self.dirty = true;
        }
    }

    fn execute(&mut self) {
        if let Ok((mut pc, ops)) = self.program.link_indirect() {
            loop {
                let op = &ops[pc];
                pc += 1;
                match op {
                    Op::Literal(val) => self.stack.push(val.clone()),
                    Op::Push(str) => {
                        self.stack.push(match self.vars.get(str) {
                            Some(val) => val.clone(),
                            None => Val::Undefined,
                        });
                    }
                    Op::Jump(addr) => pc = *addr,
                    _ => unimplemented!("{:?}", op),
                }
            }
        } else {
            // print errors
        }
    }
}
