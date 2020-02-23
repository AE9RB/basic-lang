use super::ast::*;
use super::error::*;
use super::lex::*;
use super::parse::*;
use super::token::*;

#[derive(Debug, PartialEq)]
pub struct Line {
    number: Option<u16>,
    tokens: Vec<Token>,
}

impl Line {
    pub fn new(s: &str) -> Line {
        let (line_number, tokens) = lex(s);
        Line {
            tokens: tokens,
            number: line_number,
        }
    }

    pub fn ast(&mut self) -> Result<Vec<Statement>, Error> {
        parse(self.number, &self.tokens)
    }
}

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s: String = self.tokens.iter().map(|s| s.to_string()).collect();
        if self.number.is_some() {
            write!(f, "{} {}", self.number.unwrap(), s)
        } else {
            write!(f, "{}", s)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foo() {
        let _ = Line::new("100 fancy");
    }
}
