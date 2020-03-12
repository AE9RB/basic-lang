extern crate ansi_term;
extern crate ctrlc;
extern crate linefeed;
use crate::mach::{Event, Runtime};
use ansi_term::Style;
use linefeed::complete::{Completer, Completion};
use linefeed::terminal::Terminal;
use linefeed::{Interface, Prompter, ReadResult};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub fn main() {
    let interrupted = Arc::new(AtomicBool::new(false));
    let int_moved = interrupted.clone();
    ctrlc::set_handler(move || {
        int_moved.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
    if let Err(error) = main_loop(interrupted) {
        eprintln!("{}", error);
    }
}

fn main_loop(interrupted: Arc<AtomicBool>) -> std::io::Result<()> {
    let interface = Interface::new("BASIC")?;
    let mut runtime = Arc::new(Runtime::new());
    let mut print_ready = true;

    loop {
        if interrupted.load(Ordering::SeqCst) {
            Arc::get_mut(&mut runtime).unwrap().interrupt();
            interrupted.store(false, Ordering::SeqCst);
        };
        match Arc::get_mut(&mut runtime).unwrap().execute(5000) {
            Event::Stopped => {
                if print_ready {
                    interface.write_fmt(format_args!("READY.\n"))?;
                }
                let saved_completer = interface.completer();
                interface.set_completer(Arc::new(LineCompleter::new(Arc::clone(&runtime))));
                let input = match interface.read_line()? {
                    ReadResult::Input(input) => input,
                    ReadResult::Signal(_) | ReadResult::Eof => break,
                };
                interface.set_completer(saved_completer);
                print_ready = Arc::get_mut(&mut runtime).unwrap().enter(&input);
                if print_ready {
                    interface.add_history_unique(input);
                }
            }
            Event::Errors(errors) => {
                for error in errors.iter() {
                    let error = format!("?{}", error);
                    interface.write_fmt(format_args!("{}\n", Style::new().bold().paint(error)))?;
                }
            }
            Event::Running => {}
            Event::PrintLn(s) => {
                interface.write_fmt(format_args!("{}\n", s))?;
            }
            Event::List((s, _columns)) => {
                interface.write_fmt(format_args!("{}\n", s))?;
            }
        }
    }
    Ok(())
}

struct LineCompleter {
    runtime: Arc<Runtime>,
}

impl LineCompleter {
    fn new(runtime: Arc<Runtime>) -> LineCompleter {
        LineCompleter { runtime }
    }
}

impl<'a, Term: Terminal> Completer<Term> for LineCompleter {
    fn complete(
        &self,
        _word: &str,
        prompter: &Prompter<Term>,
        _start: usize,
        _end: usize,
    ) -> Option<Vec<Completion>> {
        if let Ok(num) = prompter.buffer().parse::<usize>() {
            if let Some((s, _)) = self.runtime.line(num) {
                let mut comp_list = Vec::new();
                let mut comp = Completion::simple(s);
                comp.suffix = linefeed::complete::Suffix::None;
                comp_list.push(comp);
                return Some(comp_list);
            }
        }
        None
    }
}
