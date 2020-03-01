#[allow(dead_code)]
#[derive(Debug)]
pub enum Val {
    String(String),
    Integer(i16),
    Single(f32),
    Double(f64),
    Char(char),
    Next(usize),
    Return(usize),
}
