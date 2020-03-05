use super::{Op, Program, Val};
use crate::lang::{Error, Line, LineNumber};
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
            dbg!(self.execute());
        } else {
            self.dirty = true;
        }
    }

    fn execute(&mut self) -> Result<(), &Vec<Error>> {
        self.stack.clear();
        let mut pc = self.program.link();
        let has_indirect_errors = self.program.indirect_errors().len() > 0;
        let watermark = pc;
        if self.program.direct_errors().len() > 0 {
            return Err(self.program.direct_errors());
        }
        loop {
            let op = self.program.op(pc);
            pc += 1;
            match op {
                Op::Literal(val) => self.stack.push(val.clone()),
                Op::Push(var_name) => {
                    self.stack.push(match self.vars.get(var_name) {
                        Some(val) => val.clone(),
                        None => Val::Undefined,
                    });
                }
                Op::Run => {
                    if has_indirect_errors && pc < watermark {
                        return Err(self.program.indirect_errors());
                    }
                    self.stack.clear();
                    self.vars.clear();
                    pc = 0;
                }
                Op::Jump(addr) => {
                    if has_indirect_errors && pc < watermark {
                        return Err(self.program.indirect_errors());
                    }
                    pc = *addr;
                }
                _ => unimplemented!("{:?}", op),
            }
        }
    }
}
