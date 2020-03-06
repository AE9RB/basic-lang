use super::{ast::*, token::*, Column, Error, LineNumber};
use crate::error;

type Result<T> = std::result::Result<T, Error>;

pub fn parse(line_number: LineNumber, tokens: &[Token]) -> Result<Vec<Statement>> {
    match Parser::parse(tokens) {
        Err(e) => Err(e.in_line_number(line_number)),
        Ok(r) => Ok(r),
    }
}

struct Parser<'a> {
    token_stream: std::slice::Iter<'a, Token>,
    peeked: Option<&'a Token>,
    rem2: bool,
    col: Column,
}

impl<'a> Parser<'a> {
    fn parse(tokens: &'a [Token]) -> Result<Vec<Statement>> {
        let mut parse = Parser {
            token_stream: tokens.iter(),
            peeked: None,
            rem2: false,
            col: 0..0,
        };
        let mut r: Vec<Statement> = vec![];
        loop {
            match parse.peek() {
                None | Some(Token::Word(Word::Rem1)) => return Ok(r),
                Some(t) => {
                    if *t == &Token::Colon {
                        parse.next();
                        continue;
                    }
                }
            }
            match parse.statement() {
                Ok(s) => r.push(s),
                Err(e) => return Err(e.in_column(&parse.col)),
            }
        }
    }

    fn column(&self) -> Column {
        self.col.clone()
    }

    fn next(&mut self) -> Option<&'a Token> {
        if self.peeked.is_some() {
            return self.peeked.take();
        }
        loop {
            self.col.start = self.col.end;
            let t = self.token_stream.next()?;
            self.col.end += t.to_string().chars().count();
            if self.rem2 {
                continue;
            }
            match t {
                Token::Word(Word::Rem2) => {
                    self.rem2 = true;
                    continue;
                }
                Token::Whitespace(_) => continue,
                _ => return Some(t),
            }
        }
    }

    fn peek(&mut self) -> Option<&&'a Token> {
        if self.peeked.is_none() {
            self.peeked = self.next();
        }
        self.peeked.as_ref()
    }

    fn statement(&mut self) -> Result<Statement> {
        match self.peek() {
            Some(Token::Ident(_)) => Statement::for_word(self, &Word::Let),
            Some(Token::Word(word)) => {
                self.next();
                Statement::for_word(self, word)
            }
            _ => {
                 let error = Statement::for_word(self, &Word::Rem1);
                 debug_assert!(error.is_err());
                 error
            },
        }
    }

    fn expression(&mut self) -> Result<Expression> {
        fn parse(this: &mut Parser, precedence: usize) -> Result<Expression> {
            let mut lhs = match this.next() {
                Some(Token::LParen) => {
                    let expr = this.expression()?;
                    this.expect(Token::RParen)?;
                    expr
                }
                Some(Token::Ident(i)) => {
                    let column = this.column();
                    match this.peek() {
                        Some(&&Token::LParen) => {
                            Expression::Function(column, i.clone(), this.expression_list()?)
                        }
                        _ => Expression::Var(column, i.clone()),
                    }
                }
                Some(Token::Literal(l)) => Expression::for_literal(this.column(), l),
                _ => return Err(error!(SyntaxError; "EXPECTED EXPRESSION")),
            };
            let mut rhs;
            loop {
                match this.peek() {
                    Some(Token::Operator(op)) => {
                        let op_precedence = Expression::op_precedence(op);
                        if op_precedence < precedence {
                            break;
                        }
                        this.next();
                        let column = this.column();
                        rhs = parse(this, op_precedence)?;
                        lhs = Expression::for_binary_op(column, op, lhs, rhs);
                    }
                    _ => break,
                }
            }
            Ok(lhs)
        };
        parse(self, 0)
    }

    fn expression_list(&mut self) -> Result<Vec<Expression>> {
        self.expect(Token::LParen)?;
        let mut v: Vec<Expression> = vec![];
        loop {
            v.push(self.expression()?);
            match self.next() {
                Some(Token::RParen) => return Ok(v),
                Some(Token::Comma) => continue,
                _ => return Err(error!(SyntaxError; "EXPECTED END OR SEPARATOR")),
            }
        }
    }

    fn printer_list(&mut self) -> Result<Vec<Expression>> {
        let mut v: Vec<Expression> = vec![];
        let mut linefeed = true;
        loop {
            match self.peek() {
                None | Some(Token::Colon) => {
                    if linefeed {
                        let mut column = self.column();
                        column.end = column.start;
                        v.push(Expression::Char(column, '\n'));
                    }
                    return Ok(v);
                }
                Some(Token::Semicolon) => {
                    linefeed = false;
                    self.next();
                }
                Some(Token::Comma) => {
                    linefeed = false;
                    self.next();
                    v.push(Expression::Char(self.column(), '\t'));
                }
                _ => {
                    linefeed = true;
                    v.push(self.expression()?);
                }
            };
        }
    }

    fn ident(&mut self) -> Result<(Column, Ident)> {
        let ident = match self.next() {
            Some(Token::Ident(i)) => i.clone(),
            _ => return Err(error!(SyntaxError)),
        };
        Ok((self.column(), ident))
    }

    fn expect(&mut self, token: Token) -> Result<()> {
        if let Some(t) = self.next() {
            if *t == token {
                return Ok(());
            }
        }
        use Token::*;
        Err(error!(SyntaxError;
            match token {
                Unknown(_) | Whitespace(_) => {"UNEXPECTED TOKEN"}
                Literal(_) => {"EXPECTED LITERAL"}
                Word(_) => {"EXPECTED RESERVED WORD"}
                Operator(_) => {"EXPECTED OPERATOR"}
                Ident(_) => {"EXPECTED IDENTIFIER"}
                LParen => {"EXPECTED LEFT PARENTHESIS"}
                RParen => {"EXPECTED RIGHT PARENTHESIS"}
                Comma => {"EXPECTED COMMA"}
                Colon => {"EXPECTED COLON"}
                Semicolon => {"EXPECTED SEMICOLON"}
            }
        ))
    }
}

impl Expression {
    fn for_binary_op(col: Column, op: &Operator, lhs: Expression, rhs: Expression) -> Expression {
        use Operator::*;
        match op {
            Plus => Expression::Add(col, Box::new(lhs), Box::new(rhs)),
            Minus => Expression::Subtract(col, Box::new(lhs), Box::new(rhs)),
            Multiply => Expression::Multiply(col, Box::new(lhs), Box::new(rhs)),
            Divide => Expression::Divide(col, Box::new(lhs), Box::new(rhs)),
            _ => unimplemented!(),
        }
    }

    fn op_precedence(op: &Operator) -> usize {
        use Operator::*;
        match op {
            Equals => 0,
            Plus | Minus => 10,
            Multiply | Divide => 20,
            DivideInt => 0,
            Caret => 0,
            Modulus => 0,
            Not => 0,
            And => 0,
            Or => 0,
            Xor => 0,
            Eqv => 0,
            Imp => 0,
        }
    }

    fn for_literal(col: Column, lit: &Literal) -> Expression {
        fn clean(s: &str) -> String {
            let mut s = String::from(s).replace("D", "E");
            match s.chars().last() {
                Some('!') | Some('#') | Some('%') => {
                    s.pop();
                }
                _ => {}
            };
            s
        }
        match lit {
            Literal::Single(s) => Expression::Single(col, clean(s).parse().unwrap()),
            Literal::Double(s) => Expression::Double(col, clean(s).parse().unwrap()),
            Literal::Integer(s) => Expression::Integer(col, clean(s).parse().unwrap()),
            Literal::String(s) => Expression::String(col, s.to_string()),
        }
    }
}

impl Statement {
    fn for_word(parse: &mut Parser, word: &Word) -> Result<Statement> {
        let column = parse.column();
        use Word::*;
        match word {
            Goto1 | Goto2 => Self::r#goto(parse, column),
            Let => Self::r#let(parse, column),
            Print1 | Print2 => Self::r#print(parse, column),
            Run => Self::r#run(parse, column),
            Data | Def | Dim  | End | For | Gosub1 | Gosub2 | If | Input | Next | On
            | Read | Restore | Return | Stop => {
                Err(error!(SyntaxError; "STATEMENT NOT YET PARSING"))
            }
            Else | Rem1 | Rem2 | To | Then => Err(error!(SyntaxError; "EXPECTED STATEMENT")),
        }
    }

    fn r#let(parse: &mut Parser, column: Column) -> Result<Statement> {
        let ident = parse.ident()?;
        parse.expect(Token::Operator(Operator::Equals))?;
        let expr = parse.expression()?;
        Ok(Statement::Let(column, ident, expr))
    }

    fn r#print(parse: &mut Parser, column: Column) -> Result<Statement> {
        Ok(Statement::Print(column, parse.printer_list()?))
    }

    fn r#goto(parse: &mut Parser, column: Column) -> Result<Statement> {
        Ok(Statement::Goto(column, parse.expression()?))
    }

    fn r#run(_parse: &mut Parser, column: Column) -> Result<Statement> {
        Ok(Statement::Run(column))
    }
}

#[cfg(test)]
mod tests {
    use super::super::lex::*;
    use super::*;

    fn parse_str(s: &str) -> Statement {
        let (lin, tokens) = lex(s);
        match parse(lin, &tokens) {
            Ok(mut v) => {
                if v.len() != 1 {
                    panic!();
                }
                v.pop().unwrap()
            }
            Err(e) => panic!("{} : {:?}", e, e),
        }
    }

    #[test]
    fn test_let_foo_eq_bar() {
        let answer = Statement::Let(
            0..3,
            (3..6, Ident::Plain("TER".to_string())),
            Expression::Var(7..10, Ident::Plain("BAR".to_string())),
        );
        assert_eq!(parse_str("letter=bar:"), answer);
        let answer = Statement::Let(
            0..3,
            (0..3, Ident::Plain("TER".to_string())),
            Expression::Var(4..7, Ident::Plain("BAR".to_string())),
        );
        assert_eq!(parse_str("ter=bar:"), answer);
    }

    #[test]
    fn test_literals() {
        let answer = Statement::Let(
            0..1,
            (0..1, Ident::Plain("A".to_string())),
            Expression::Integer(2..4, 12),
        );
        assert_eq!(parse_str("A=12"), answer);
        let answer = Statement::Let(
            0..1,
            (0..1, Ident::Plain("A".to_string())),
            Expression::Single(2..5, 12.0),
        );
        assert_eq!(parse_str("A=12!"), answer);
        let answer = Statement::Let(
            0..1,
            (0..1, Ident::Plain("A".to_string())),
            Expression::Double(2..6, 12e4),
        );
        assert_eq!(parse_str("A=12d4"), answer);
        let answer = Statement::Let(
            0..1,
            (0..1, Ident::Plain("A".to_string())),
            Expression::String(2..8, "food".to_string()),
        );
        assert_eq!(parse_str("A=\"food\""), answer);
        let answer = Statement::Let(
            0..1,
            (0..1, Ident::Plain("A".to_string())),
            Expression::Double(2..58, 0.0),
        );
        assert_eq!(
            parse_str("A=798347598234765983475983248592d-234721398742391847982344"),
            answer
        );
    }

    #[test]
    fn test_functions() {
        let answer = Statement::Let(
            0..1,
            (0..1, Ident::Plain("A".to_string())),
            Expression::Function(
                2..5,
                Ident::Plain("COS".to_string()),
                vec![Expression::Single(6..10, 3.14)],
            ),
        );
        assert_eq!(parse_str("A=cos(3.14)"), answer);
    }

    #[test]
    fn test_precedence_and_paren() {
        let answer = Statement::Let(
            0..3,
            (4..5, Ident::Plain("A".to_string())),
            Expression::Subtract(
                8..9,
                Box::new(Expression::Integer(7..8, 2)),
                Box::new(Expression::Multiply(
                    22..23,
                    Box::new(Expression::Add(
                        11..12,
                        Box::new(Expression::Integer(10..11, 3)),
                        Box::new(Expression::Function(
                            12..15,
                            Ident::Plain("COS".to_string()),
                            vec![Expression::Single(16..20, 3.14)],
                        )),
                    )),
                    Box::new(Expression::Integer(23..24, 4)),
                )),
            ),
        );
        assert_eq!(parse_str("let A=(2-(3+cos(3.14))*4)"), answer);
    }

    #[test]
    fn test_printer_list() {
        let (lin, tokens) = lex("? 1 2,3;:?");
        assert_eq!(
            parse(lin, &tokens).unwrap(),
            vec!(
                Statement::Print(
                    0..1,
                    vec!(
                        Expression::Integer(2..3, 1),
                        Expression::Integer(4..5, 2),
                        Expression::Char(5..6, '\t'),
                        Expression::Integer(6..7, 3),
                    )
                ),
                Statement::Print(9..10, vec!(Expression::Char(10..10, '\n'),)),
            )
        );
    }
}
