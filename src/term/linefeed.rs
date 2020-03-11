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
    interface.set_completer(Arc::new(LineCompleter));
    let mut runtime = Runtime::new();
    let mut print_ready = true;

    loop {
        if interrupted.load(Ordering::SeqCst) {
            runtime.interrupt();
            interrupted.store(false, Ordering::SeqCst);
        };
        match runtime.execute(5000) {
            Event::Stopped => {
                if print_ready {
                    print_ready = false;
                    interface.write_fmt(format_args!("READY.\n"))?;
                }
                match interface.read_line()? {
                    ReadResult::Input(input) => {
                        runtime.enter(&input);
                        if !runtime.is_stopped() {
                            print_ready = true;
                        }
                    }
                    ReadResult::Signal(_) | ReadResult::Eof => break,
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

struct LineCompleter;

impl<Term: Terminal> Completer<Term> for LineCompleter {
    fn complete(
        &self,
        _word: &str,
        prompter: &Prompter<Term>,
        _start: usize,
        _end: usize,
    ) -> Option<Vec<Completion>> {
        let line = prompter.buffer();
        if line == "10" {
            let mut compls = Vec::new();
            let mut c = Completion::simple("10 PRINT \"HELLO WORLD\"".to_owned());
            c.suffix = linefeed::complete::Suffix::None;
            compls.push(c);
            return Some(compls);
        }
        None
    }
}
