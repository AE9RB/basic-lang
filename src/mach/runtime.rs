use super::*;
use crate::error;
use crate::lang::{Error, Line, LineNumber};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

type Result<T> = std::result::Result<T, Error>;

const INTRO: &str = "64K BASIC";
const PROMPT: &str = "READY.";

/// ## Virtual machine

pub struct Runtime {
    prompt: String,
    source: Listing,
    dirty: bool,
    program: Program,
    pc: Address,
    entry_address: Address,
    stack: RuntimeStack,
    vars: Var,
    state: State,
    cont: State,
    cont_pc: Address,
    print_col: usize,
    rand: (u32, u32, u32),
    functions: HashMap<Rc<str>, (usize, Address)>,
}

/// ## Events for the user interface

#[derive(Debug)]
pub enum Event {
    Errors(Arc<Vec<Error>>),
    Input(String, bool),
    Print(String),
    List((String, Vec<Range<usize>>)),
    Running,
    Stopped,
    Load(String),
    Save(String),
    Cls,
    Inkey,
}

#[derive(Debug)]
enum State {
    Intro,
    Stopped,
    Listing(Range<LineNumber>),
    RuntimeError(Error),
    Running,
    Input,
    InputRedo,
    InputRunning,
    Interrupt,
    Inkey,
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime {
            prompt: PROMPT.into(),
            source: Listing::default(),
            dirty: false,
            program: Program::default(),
            pc: 0,
            entry_address: 1,
            stack: Stack::new("STACK OVERFLOW"),
            vars: Var::new(),
            state: State::Intro,
            cont: State::Stopped,
            cont_pc: 0,
            print_col: 0,
            rand: (1, 1, 1),
            functions: HashMap::default(),
        }
    }
}

impl Runtime {
    /// Enters a line of BASIC or INPUT.
    /// Returns true if good candidate for history.
    pub fn enter(&mut self, string: &str) -> bool {
        if let State::Input = self.state {
            self.enter_input(string);
            self.print_col = 0;
            return true;
        }
        if let State::Inkey = self.state {
            self.enter_inkey(string);
            return false;
        }
        debug_assert!(matches!(self.state, State::Stopped | State::Intro));
        if string.len() > MAX_LINE_LEN {
            self.state = State::RuntimeError(error!(LineBufferOverflow));
            return false;
        }
        let line = Line::new(string);
        if line.is_direct() {
            if line.is_empty() {
                false
            } else {
                self.enter_direct(line);
                true
            }
        } else {
            self.enter_indirect(line);
            false
        }
    }

    fn enter_direct(&mut self, line: Line) {
        if self.dirty {
            self.program.clear();
            self.program.compile(self.source.lines());
            self.dirty = false;
        }
        self.program.compile(&line);
        let (pc, indirect_errors, direct_errors) = self.program.link();
        self.pc = pc;
        self.entry_address = pc;
        self.source.indirect_errors = indirect_errors;
        self.source.direct_errors = direct_errors;
        self.state = State::Running;
    }

    fn enter_indirect(&mut self, line: Line) {
        self.cont = State::Stopped;
        if line.is_empty() {
            self.dirty = self.source.remove(line.number()).is_some();
        } else {
            self.source.insert(line);
            self.dirty = true;
        }
    }

    fn enter_inkey(&mut self, mut string: &str) {
        if string.len() > MAX_LINE_LEN {
            string = "";
        }
        if let Err(error) = self.stack.push(Val::String(string.into())) {
            self.clear();
            self.state = State::RuntimeError(error);
        }
        self.state = State::Running;
    }

    fn enter_input(&mut self, string: &str) {
        if string.len() > MAX_LINE_LEN {
            self.state = State::InputRedo;
            return;
        }
        if let Err(error) = self.do_input(string) {
            self.clear();
            self.state = State::RuntimeError(error);
            debug_assert!(false, "BAD INPUT STACK");
        }
    }

    fn do_input(&mut self, string: &str) -> Result<()> {
        let len = match self.stack.last() {
            Some(Val::Integer(n)) => *n as usize,
            _ => return Err(error!(InternalError)),
        };
        let mut vec_val: Vec<Val> = vec![];
        if len <= 1 {
            vec_val.push(Val::String(string.into()));
        } else {
            let mut start: usize = 0;
            let mut in_quote = false;
            for (index, ch) in string.char_indices() {
                match ch {
                    '"' => {
                        in_quote = !in_quote;
                    }
                    ',' if !in_quote => {
                        vec_val.push(Val::String(string[start..index].into()));
                        start = index + 1;
                    }
                    _ => {}
                }
            }
            vec_val.push(Val::String(string[start..].into()));
            if len as usize != vec_val.len() {
                self.state = State::InputRedo;
                return Ok(());
            }
        }
        self.stack.push(Val::Return(self.pc))?;
        while let Some(v) = vec_val.pop() {
            self.stack.push(v)?;
        }
        self.state = State::InputRunning;
        Ok(())
    }

    fn ready_prompt(&mut self) -> Option<Event> {
        if self.entry_address != 0 {
            self.entry_address = 0;
            let mut s = String::new();
            if self.print_col > 0 {
                s.push('\n');
                self.print_col = 0;
            }
            if !self.prompt.is_empty() {
                s.push_str(&self.prompt);
                s.push('\n');
            }
            return Some(Event::Print(s));
        };
        None
    }

    /// Obtain a thread-safe Listing for saving and line completion.
    pub fn get_listing(&self) -> Listing {
        self.source.clone()
    }

    /// Set a new listing. Used to load a program.
    pub fn set_listing(&mut self, listing: Listing) {
        self.r#new_();
        self.source = listing;
    }

    /// Set a prompt instead of the default "READY."
    pub fn set_prompt(&mut self, prompt: &str) {
        self.prompt = prompt.into();
    }

    /// Interrupt the program. Displays `BREAK` error.
    pub fn interrupt(&mut self) {
        self.cont = State::Interrupt;
        std::mem::swap(&mut self.state, &mut self.cont);
        self.cont_pc = self.pc;
        if self.pc >= self.entry_address {
            self.cont = State::Stopped;
            self.stack.clear();
        }
    }

    /// Use a large number for iterations but not so much
    /// that interrupts aren't responsive.
    pub fn execute(&mut self, iterations: usize) -> Event {
        fn line_number(this: &Runtime) -> LineNumber {
            let mut pc = this.pc;
            if pc > 0 {
                pc -= 1
            }
            this.program.line_number_for(pc)
        }
        match &self.state {
            State::Intro => {
                self.state = State::Stopped;
                let mut s = INTRO.to_string();
                if let Some(version) = option_env!("CARGO_PKG_VERSION") {
                    s.push(' ');
                    s.push_str(version);
                }
                #[cfg(debug_assertions)]
                s.push_str("+debug");
                s.push('\n');
                return Event::Print(s);
            }
            State::Stopped => match self.ready_prompt() {
                Some(e) => return e,
                None => return Event::Stopped,
            },
            State::Interrupt => {
                self.state = State::RuntimeError(error!(Break, line_number(&self)));
            }
            State::Listing(range) => {
                let mut range = range.clone();
                if let Some((string, columns)) = self.source.list_line(&mut range) {
                    self.state = State::Listing(range);
                    return Event::List((string, columns));
                } else {
                    self.state = State::Running;
                }
            }
            State::Input => match self.execute_input() {
                Ok(event) => return event,
                Err(error) => {
                    self.state = State::RuntimeError(error.in_line_number(line_number(&self)))
                }
            },
            State::InputRedo => {
                self.state = State::Input;
                return Event::Errors(Arc::new(vec![error!(RedoFromStart)]));
            }
            State::InputRunning | State::Running => {
                if !self.source.direct_errors.is_empty() {
                    self.state = State::Stopped;
                    return Event::Errors(Arc::clone(&self.source.direct_errors));
                }
            }
            State::Inkey | State::RuntimeError(_) => {}
        }
        if let State::RuntimeError(_) = self.state {
            if self.print_col > 0 {
                self.print_col = 0;
                return Event::Print('\n'.to_string());
            }
            let mut state = State::Stopped;
            std::mem::swap(&mut self.state, &mut state);
            if let State::RuntimeError(error) = state {
                return Event::Errors(Arc::new(vec![error]));
            }
        }
        debug_assert!(matches!(self.state, State::Running | State::InputRunning));
        match self.execute_loop(iterations) {
            Ok(event) => {
                if let State::Stopped = self.state {
                    if let Event::Stopped = event {
                        if let Some(event) = self.ready_prompt() {
                            return event;
                        }
                    }
                }
                event
            }
            Err(error) => {
                if let State::InputRunning = self.state {
                    loop {
                        match self.stack.pop() {
                            Err(_) => break,
                            Ok(Val::Return(addr)) => {
                                self.pc = addr;
                                break;
                            }
                            _ => continue,
                        }
                    }
                    self.state = State::InputRedo;
                } else {
                    self.cont = State::RuntimeError(error.in_line_number(line_number(&self)));
                    std::mem::swap(&mut self.cont, &mut self.state);
                    self.cont_pc = self.pc;
                    if self.pc >= self.entry_address || self.stack.is_full() {
                        self.stack.clear();
                        self.cont = State::Stopped;
                    }
                }
                Event::Running
            }
        }
    }

    fn execute_input(&mut self) -> Result<Event> {
        let len = self.stack.pop()?;
        let caps = self.stack.pop()?;
        let mut prompt = match self.stack.last() {
            Some(Val::String(s)) => s.to_string(),
            _ => return Err(error!(InternalError)),
        };
        prompt.push('?');
        prompt.push(' ');
        let is_caps = match caps {
            Val::Integer(i) if i == 0 => false,
            _ => true,
        };
        self.stack.push(caps)?;
        self.stack.push(len)?;
        self.print_col = 0;
        Ok(Event::Input(prompt, is_caps))
    }

    #[allow(clippy::cognitive_complexity)]
    fn execute_loop(&mut self, iterations: usize) -> Result<Event> {
        let has_indirect_errors = !self.source.indirect_errors.is_empty();
        for _ in 0..iterations {
            let op = match self.program.get(self.pc) {
                Some(v) => v,
                None => return Err(error!(InternalError; "INVALID PC ADDRESS")),
            };
            self.pc += 1;
            match op {
                Opcode::Literal(val) => self.stack.push(val.clone())?,
                Opcode::Pop(var_name) => self.vars.store(&var_name, self.stack.pop()?)?,
                Opcode::Push(var_name) => self.stack.push(self.vars.fetch(&var_name))?,
                Opcode::PopArr(var_name) => {
                    let vec = self.stack.pop_vec()?;
                    let val = self.stack.pop()?;
                    self.vars.store_array(&var_name, vec, val)?;
                }
                Opcode::PushArr(var_name) => {
                    let vec = self.stack.pop_vec()?;
                    let val = self.vars.fetch_array(&var_name, vec)?;
                    self.stack.push(val)?;
                }
                Opcode::DimArr(var_name) => {
                    let vec = self.stack.pop_vec()?;
                    self.vars.dimension_array(&var_name, vec)?;
                }
                Opcode::IfNot(addr) => {
                    if match self.stack.pop()? {
                        Val::Return(_) | Val::String(_) | Val::Next(_) => {
                            return Err(error!(TypeMismatch))
                        }
                        Val::Integer(n) => n == 0,
                        Val::Single(n) => n == 0.0,
                        Val::Double(n) => n == 0.0,
                    } {
                        self.pc = addr;
                    }
                }
                Opcode::Jump(addr) => {
                    self.pc = addr;
                    if has_indirect_errors && self.pc < self.entry_address {
                        self.state = State::Stopped;
                        self.cont = State::Stopped;
                        return Ok(Event::Errors(Arc::clone(&self.source.indirect_errors)));
                    }
                }
                Opcode::Clear => self.r#clear(),
                Opcode::Cls => return self.r#cls(),
                Opcode::Cont => {
                    if let Some(event) = self.r#cont()? {
                        return Ok(event);
                    }
                }
                Opcode::Def(var_name) => self.r#def(var_name)?,
                Opcode::End => return Ok(self.r#end()),
                Opcode::Fn(var_name) => self.r#fn(var_name)?,
                Opcode::Input(var_name) => {
                    if let Some(event) = self.r#input(var_name)? {
                        return Ok(event);
                    }
                }
                Opcode::List => return self.r#list(),
                Opcode::Load => return self.r#load(),
                Opcode::New => return Ok(self.r#new_()),
                Opcode::On => self.r#on()?,
                Opcode::Next(var_name) => self.r#next(var_name)?,
                Opcode::Print => return self.r#print(),
                Opcode::Read => self.r#read()?,
                Opcode::Restore(addr) => self.r#restore(addr)?,
                Opcode::Return => self.r#return()?,
                Opcode::Save => return self.r#save(),
                Opcode::Stop => return Err(error!(Break)),

                Opcode::Neg => self.stack.pop_1_push(&Operation::negate)?,
                Opcode::Pow => self.stack.pop_2_push(&Operation::power)?,
                Opcode::Mul => self.stack.pop_2_push(&Operation::multiply)?,
                Opcode::Div => self.stack.pop_2_push(&Operation::divide)?,
                Opcode::DivInt => self.stack.pop_2_push(&Operation::divint)?,
                Opcode::Mod => self.stack.pop_2_push(&Operation::remainder)?,
                Opcode::Add => self.stack.pop_2_push(&Operation::sum)?,
                Opcode::Sub => self.stack.pop_2_push(&Operation::subtract)?,
                Opcode::Eq => self.stack.pop_2_push(&Operation::equal)?,
                Opcode::NotEq => self.stack.pop_2_push(&Operation::not_equal)?,
                Opcode::Lt => self.stack.pop_2_push(&Operation::less)?,
                Opcode::LtEq => self.stack.pop_2_push(&Operation::less_equal)?,
                Opcode::Gt => self.stack.pop_2_push(&Operation::greater)?,
                Opcode::GtEq => self.stack.pop_2_push(&Operation::greater_equal)?,
                Opcode::Not => self.stack.pop_1_push(&Operation::not)?,
                Opcode::And => self.stack.pop_2_push(&Operation::and)?,
                Opcode::Or => self.stack.pop_2_push(&Operation::or)?,
                Opcode::Xor => self.stack.pop_2_push(&Operation::xor)?,
                Opcode::Imp => self.stack.pop_2_push(&Operation::imp)?,
                Opcode::Eqv => self.stack.pop_2_push(&Operation::eqv)?,

                Opcode::Abs => self.stack.pop_1_push(&Function::abs)?,
                Opcode::Asc => self.stack.pop_1_push(&Function::asc)?,
                Opcode::Chr => self.stack.pop_1_push(&Function::chr)?,
                Opcode::Cos => self.stack.pop_1_push(&Function::cos)?,
                Opcode::Date => self.stack.push(Function::date()?)?,
                Opcode::Exp => self.stack.pop_1_push(&Function::exp)?,
                Opcode::Inkey => {
                    self.state = State::Inkey;
                    return Ok(Event::Inkey);
                }
                Opcode::Int => self.stack.pop_1_push(&Function::int)?,
                Opcode::Left => self.stack.pop_2_push(&Function::left)?,
                Opcode::Len => self.stack.pop_1_push(&Function::len)?,
                Opcode::Mid => {
                    let vec = self.stack.pop_vec()?;
                    self.stack.push(Function::mid(vec)?)?;
                }
                Opcode::Right => self.stack.pop_2_push(&Function::right)?,
                Opcode::Rnd => {
                    let vec = self.stack.pop_vec()?;
                    self.stack.push(Function::rnd(&mut self.rand, vec)?)?;
                }
                Opcode::Sgn => self.stack.pop_1_push(&Function::sgn)?,
                Opcode::Sin => self.stack.pop_1_push(&Function::sin)?,
                Opcode::Sqr => self.stack.pop_1_push(&Function::sqr)?,
                Opcode::Str => self.stack.pop_1_push(&Function::str)?,
                Opcode::Tab => {
                    let val = self.stack.pop()?;
                    self.stack.push(Function::tab(self.print_col, val)?)?;
                }
                Opcode::Time => self.stack.push(Function::time()?)?,
                Opcode::Val => self.stack.pop_1_push(&Function::val)?,
            }
        }
        Ok(Event::Running)
    }

    fn r#clear(&mut self) {
        self.program.restore_data(0);
        self.stack.clear();
        self.vars.clear();
        self.functions.clear();
        self.cont = State::Stopped;
    }

    fn r#cls(&mut self) -> Result<Event> {
        Ok(Event::Cls)
    }

    fn r#cont(&mut self) -> Result<Option<Event>> {
        if let State::Stopped = self.cont {
            return Err(error!(CantContinue));
        }
        if let State::Running = self.state {
            self.state = State::Stopped;
            std::mem::swap(&mut self.cont, &mut self.state);
            self.pc = self.cont_pc;
        } else {
            return Err(error!(CantContinue));
        }
        if let State::Running = self.state {
            Ok(None)
        } else {
            Ok(Some(Event::Running))
        }
    }

    fn r#def(&mut self, fn_name: Rc<str>) -> Result<()> {
        if self.pc >= self.entry_address {
            Err(error!(IllegalDirect))
        } else if let Val::Integer(len) = self.stack.pop()? {
            self.functions.insert(fn_name, (len as usize, self.pc + 1));
            Ok(())
        } else {
            Err(error!(InternalError))
        }
    }

    fn r#end(&mut self) -> Event {
        self.cont = State::Stopped;
        std::mem::swap(&mut self.cont, &mut self.state);
        self.cont_pc = self.pc;
        if self.pc >= self.entry_address {
            self.cont = State::Stopped;
            self.stack.clear();
        }
        Event::Stopped
    }

    fn r#fn(&mut self, fn_name: Rc<str>) -> Result<()> {
        let mut args = self.stack.pop_vec()?;
        if let Some((arity, addr)) = self.functions.get(&fn_name) {
            if *arity == args.len() {
                self.stack.push(Val::Return(self.pc))?;
                for arg in args.drain(..).rev() {
                    self.stack.push(arg)?;
                }
                self.pc = *addr;
                Ok(())
            } else {
                Err(error!(IllegalFunctionCall; "WRONG NUMBER OF ARGUMENTS"))
            }
        } else {
            Err(error!(UndefinedUserFunction))
        }
    }

    fn r#input(&mut self, var_name: Rc<str>) -> Result<Option<Event>> {
        if let State::Running = self.state {
            self.state = State::Input;
            self.pc -= 1;
            return Ok(Some(Event::Running));
        } else if let State::InputRunning = self.state {
            if var_name.is_empty() {
                self.state = State::Running;
                self.stack.pop()?;
                self.stack.pop()?;
                self.stack.pop()?;
                self.stack.pop()?;
                return Ok(None);
            } else if let Val::String(field) = self.stack.pop()? {
                let mut field = field.trim();
                if var_name.ends_with('$') {
                    if field.len() >= 2 && field.starts_with('"') && field.ends_with('"') {
                        field = &field[1..field.len() - 1];
                    }
                    self.stack.push(Val::String(field.into()))?;
                } else if field.is_empty() {
                    self.stack.push(Val::Integer(0))?;
                } else {
                    self.stack.push(Val::from(field))?;
                }
                return Ok(None);
            }
        }
        debug_assert!(false, "input stack corrupt");
        Err(error!(InternalError))
    }

    fn r#list(&mut self) -> Result<Event> {
        let (from, to) = self.stack.pop_2()?;
        let from = LineNumber::try_from(from)?;
        let to = LineNumber::try_from(to)?;
        if to < from {
            return Err(error!(UndefinedLine; "INVALID RANGE"));
        }
        self.state = State::Listing(from..to);
        Ok(Event::Running)
    }

    fn r#load(&mut self) -> Result<Event> {
        match self.stack.pop()? {
            Val::String(s) => {
                self.r#end();
                if self.pc < self.entry_address {
                    Err(error!(IllegalDirect))
                } else {
                    Ok(Event::Load(s.to_string()))
                }
            }
            _ => Err(error!(TypeMismatch)),
        }
    }

    fn r#new_(&mut self) -> Event {
        self.r#clear();
        self.source.clear();
        self.dirty = true;
        self.state = State::Stopped;
        Event::Stopped
    }

    fn r#next(&mut self, next_name: Rc<str>) -> Result<()> {
        loop {
            let next = match self.stack.pop() {
                Ok(Val::Next(addr)) => addr,
                Ok(_) | Err(_) => return Err(error!(NextWithoutFor)),
            };
            let var_name_val = self.stack.pop()?;
            let step_val = self.stack.pop()?;
            let to_val = self.stack.pop()?;
            if let Val::String(var_name) = var_name_val {
                if !next_name.is_empty() && var_name != next_name {
                    continue;
                }
                let mut current = self.vars.fetch(&var_name);
                current = Operation::sum(current, step_val.clone())?;
                self.vars.store(&var_name, current.clone())?;
                if let Ok(step) = f64::try_from(step_val.clone()) {
                    let done = Val::Integer(-1)
                        == if step < 0.0 {
                            Operation::less(current, to_val.clone())?
                        } else {
                            Operation::less(to_val.clone(), current)?
                        };
                    if !done {
                        self.stack.push(to_val)?;
                        self.stack.push(step_val)?;
                        self.stack.push(Val::String(var_name))?;
                        self.stack.push(Val::Next(next))?;
                        self.pc = next;
                    }
                    return Ok(());
                }
            }
        }
    }

    fn r#on(&mut self) -> Result<()> {
        let select = i16::try_from(self.stack.pop()?)?;
        let len = i16::try_from(self.stack.pop()?)?;
        if select < 0 || len < 0 {
            return Err(error!(IllegalFunctionCall));
        }
        if select == 0 || select > len {
            self.pc += len as usize;
        } else {
            self.pc += select as usize - 1;
        }
        Ok(())
    }

    fn r#print(&mut self) -> Result<Event> {
        let item = self.stack.pop()?;
        let val_str = match item {
            Val::String(s) => s,
            _ => format!("{} ", item).into(),
        };
        for ch in val_str.chars() {
            match ch {
                '\n' => self.print_col = 0,
                _ => self.print_col += 1,
            }
        }
        Ok(Event::Print(val_str.to_string()))
    }

    fn r#read(&mut self) -> Result<()> {
        let val = self.program.read_data()?;
        self.stack.push(val)
    }

    fn r#restore(&mut self, addr: Address) -> Result<()> {
        self.program.restore_data(addr);
        Ok(())
    }

    fn r#return(&mut self) -> Result<()> {
        let mut ret_val: Option<Val> = None;
        let mut first = true;
        loop {
            match self.stack.pop() {
                Ok(Val::Return(addr)) => {
                    if let Some(val) = ret_val {
                        self.stack.push(val)?;
                    }
                    self.pc = addr;
                    return Ok(());
                }
                Ok(val) => {
                    if first
                        && matches!(
                            val,
                            Val::String(..) | Val::Single(..) | Val::Double(..) | Val::Integer(..)
                        )
                    {
                        ret_val = Some(val);
                    }
                    first = false;
                    continue;
                }
                Err(_) => return Err(error!(ReturnWithoutGosub)),
            }
        }
    }

    fn r#save(&mut self) -> Result<Event> {
        match self.stack.pop()? {
            Val::String(s) => {
                self.r#end();
                if self.pc < self.entry_address {
                    Err(error!(IllegalDirect))
                } else {
                    Ok(Event::Save(s.to_string()))
                }
            }
            _ => Err(error!(TypeMismatch)),
        }
    }
}

type RuntimeStack = Stack<Val>;

trait RuntimeStackTrait<T> {
    fn pop_1_push<F: Fn(Val) -> Result<Val>>(&mut self, func: &F) -> Result<()>;
    fn pop_2_push<F: Fn(Val, Val) -> Result<Val>>(&mut self, func: &F) -> Result<()>;
    fn pop_vec(&mut self) -> Result<Stack<Val>>;
}

impl RuntimeStackTrait<Val> for RuntimeStack {
    fn pop_1_push<F: Fn(Val) -> Result<Val>>(&mut self, func: &F) -> Result<()> {
        let val = self.pop()?;
        self.push(func(val)?)?;
        Ok(())
    }
    fn pop_2_push<F: Fn(Val, Val) -> Result<Val>>(&mut self, func: &F) -> Result<()> {
        let (val1, val2) = self.pop_2()?;
        self.push(func(val1, val2)?)?;
        Ok(())
    }
    fn pop_vec(&mut self) -> Result<Stack<Val>> {
        if let Val::Integer(n) = self.pop()? {
            self.pop_n(n as usize)
        } else {
            Err(error!(InternalError; "NO VECTOR ON STACK"))
        }
    }
}
