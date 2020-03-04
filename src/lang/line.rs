use super::*;

#[derive(Debug)]
pub struct Line {
    number: LineNumber,
    tokens: Vec<token::Token>,
}

impl Line {
    pub fn new(s: &str) -> Line {
        let (line_number, tokens) = lex(s);
        Line {
            tokens: tokens,
            number: line_number,
        }
    }

    pub fn number(&self) -> LineNumber {
        self.number
    }

    pub fn ast(&self) -> Result<Vec<ast::Statement>, Error> {
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
