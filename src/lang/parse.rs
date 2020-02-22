use super::ast::*;
use super::error::*;
use super::token::*;

type Result = std::result::Result<Vec<Statement>, Error>;

pub fn parse<'a, T: Iterator<Item = &'a Token>>(iter: T) -> Result {
    Parse {
        token_stream: iter
            .filter({
                |&_| {
                    //todo filter whitespace
                    true
                }
            } as fn(&&Token) -> bool)
            .peekable(),
    }
    .start()
}

struct Parse<'a, T: Iterator<Item = &'a Token>> {
    token_stream: std::iter::Peekable<T>,
}

impl<'a, T: Iterator<Item = &'a Token>> Parse<'a, T> {
    fn next(&mut self) -> Option<&Token> {
        self.token_stream.next()
    }

    fn peek(&mut self) -> Option<&&Token> {
        self.token_stream.peek()
    }

    fn expect(&mut self, _: &Token) -> Result {
        match self.next() {
            Some(_) => Ok(vec![Statement::Data(vec![])]),
            None => error!(SyntaxError),
        }
    }

    fn start(&mut self) -> Result {
        self.peek();
        self.expect(&Token::Comma)?;
        Ok(vec![Statement::Data(vec![])])
    }
}

#[cfg(test)]
mod tests {
    use super::super::lex::*;
    use super::*;

    fn parse_str(s: &str) -> Result {
        let (_, tokens) = lex(s);
        parse(tokens.iter())
    }

    #[test]
    fn test_foo1() {
        let x = parse_str("for i%=1to30-10").unwrap();
        assert_eq!(x, vec![Statement::Data(vec![])]);
    }

    #[test]
    fn test_foo2() {
        let x = parse_str("for i%=1to30-10").unwrap();
        assert_eq!(x, vec![Statement::Data(vec![])]);
    }
}
