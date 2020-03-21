use basic::mach::{Event, Runtime};

pub fn exec(runtime: &mut Runtime) -> String {
    exec_n(runtime, 5000)
}

pub fn exec_n(runtime: &mut Runtime, cycles: usize) -> String {
    let mut s = String::new();
    let mut prev_running = false;
    loop {
        let event = runtime.execute(cycles);
        match &event {
            Event::Stopped => {
                break;
            }
            Event::Errors(errors) => {
                for error in errors.iter() {
                    s.push_str(&format!("{}\n", error));
                }
            }
            Event::Running => {
                if prev_running {
                    s.push_str(&format!("\n{} Execution cycles exceeded.\n", cycles));
                    break;
                }
            }
            Event::Print(ps) => {
                s.push_str(&ps);
            }
            Event::Input(ps, _) => {
                s.push_str(&ps);
                break;
            }
            Event::List((ls, _columns)) => {
                s.push_str(&format!("{}\n", ls));
            }
        }
        match event {
            Event::Running => prev_running = true,
            _ => prev_running = false,
        }
    }
    s.trim_end_matches("READY.\n").to_string()
}
