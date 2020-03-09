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
                Some(Token::Colon) => {
                    parse.next();
                    continue;
                }
                Some(_) => {
                    r.push(parse.expect_statement()?);
                }
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

    fn expect_statement(&mut self) -> Result<Statement> {
        Statement::expect(self)
    }

    fn expect_expression(&mut self) -> Result<Expression> {
        Expression::expect(self)
    }

    fn expect_expression_list(&mut self) -> Result<Vec<Expression>> {
        self.expect(Token::LParen)?;
        let mut v: Vec<Expression> = vec![];
        loop {
            v.push(self.expect_expression()?);
            match self.next() {
                Some(Token::RParen) => return Ok(v),
                Some(Token::Comma) => continue,
                _ => {
                    return Err(error!(SyntaxError, ..&self.column(); "EXPECTED END OR SEPARATOR"))
                }
            }
        }
    }

    fn expect_print_list(&mut self) -> Result<Vec<Expression>> {
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
                    v.push(self.expect_expression()?);
                }
            };
        }
    }

    fn expect_ident(&mut self) -> Result<(Column, Ident)> {
        let ident = match self.next() {
            Some(Token::Ident(i)) => i.clone(),
            _ => return Err(error!(SyntaxError, ..&self.column(); "EXPECTED IDENT")),
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
        Err(error!(SyntaxError, ..&self.column();
            match token {
                Unknown(_) | Whitespace(_) => {"PANIC"}
                Literal(_) => {"EXPECTED LITERAL"}
                Word(_) => {"EXPECTED STATEMENT WORD"}
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
    fn expect(parse: &mut Parser) -> Result<Expression> {
        fn descend(parse: &mut Parser, precedence: usize) -> Result<Expression> {
            let mut lhs = match parse.next() {
                Some(Token::LParen) => {
                    let expr = descend(parse, 0)?;
                    parse.expect(Token::RParen)?;
                    expr
                }
                Some(Token::Ident(i)) => {
                    let column = parse.column();
                    match parse.peek() {
                        Some(&&Token::LParen) => {
                            Expression::Function(column, i.clone(), parse.expect_expression_list()?)
                        }
                        _ => Expression::Var(column, i.clone()),
                    }
                }
                Some(Token::Operator(Operator::Plus)) => {
                    let op_prec = Expression::binary_op_prec(&Operator::Caret) - 1;
                    descend(parse, op_prec)?
                }
                Some(Token::Operator(Operator::Minus)) => {
                    let column = parse.column();
                    let op_prec = Expression::binary_op_prec(&Operator::Caret) - 1;
                    let expr = descend(parse, op_prec)?;
                    Expression::Negation(column, Box::new(expr))
                }
                Some(Token::Literal(lit)) => Expression::for_literal(parse.column(), lit)?,
                _ => return Err(error!(SyntaxError, ..&parse.column(); "EXPECTED EXPRESSION")),
            };
            let mut rhs;
            loop {
                match parse.peek() {
                    Some(Token::Operator(op)) => {
                        let op_prec = Expression::binary_op_prec(op);
                        if op_prec < precedence {
                            break;
                        }
                        parse.next();
                        let column = parse.column();
                        rhs = descend(parse, op_prec)?;
                        lhs = Expression::for_binary_op(column, op, lhs, rhs)?;
                    }
                    _ => break,
                }
            }
            Ok(lhs)
        };
        descend(parse, 0)
    }

    fn for_binary_op(
        col: Column,
        op: &Operator,
        lhs: Expression,
        rhs: Expression,
    ) -> Result<Expression> {
        use Operator::*;
        Ok(match op {
            Plus => Expression::Add(col, Box::new(lhs), Box::new(rhs)),
            Minus => Expression::Subtract(col, Box::new(lhs), Box::new(rhs)),
            Multiply => Expression::Multiply(col, Box::new(lhs), Box::new(rhs)),
            Divide => Expression::Divide(col, Box::new(lhs), Box::new(rhs)),
            _ => {
                dbg!(&op);
                return Err(error!(InternalError, ..&col; "OPERATOR NOT YET PARSING; PANIC"));
            }
        })
    }

    fn binary_op_prec(op: &Operator) -> usize {
        use Operator::*;
        match op {
            Caret => 13,
            // Unary identity and negation = Caret - 1
            Multiply | Divide => 11,
            DivideInt => 10,
            Modulus => 9,
            Plus | Minus => 8,
            Equal | NotEqual | Less | LessEqual | Greater | GreaterEqual => 7,
            Not => 6,
            And => 5,
            Or => 4,
            Xor => 3,
            Imp => 2,
            Eqv => 1,
        }
    }

    fn for_literal(col: Column, lit: &Literal) -> Result<Expression> {
        fn parse<T: std::str::FromStr>(col: Column, s: &str) -> Result<T> {
            let mut s = String::from(s).replace("D", "E");
            match s.chars().last() {
                Some('!') | Some('#') | Some('%') => {
                    s.pop();
                }
                _ => {}
            };
            match s.parse() {
                Ok(num) => Ok(num),
                Err(_) => return Err(error!(TypeMismatch, ..&col)),
            }
        }
        match lit {
            Literal::Single(s) => Ok(Expression::Single(col.clone(), parse(col, s)?)),
            Literal::Double(s) => Ok(Expression::Double(col.clone(), parse(col, s)?)),
            Literal::Integer(s) => Ok(Expression::Integer(col.clone(), parse(col, s)?)),
            Literal::String(s) => Ok(Expression::String(col, s.to_string())),
        }
    }
}

impl Statement {
    fn expect(parse: &mut Parser) -> Result<Statement> {
        parse.peek();
        match parse.peek() {
            Some(Token::Ident(_)) => return Self::r#let(parse),
            Some(Token::Word(word)) => {
                parse.next();
                use Word::*;
                match word {
                    Goto1 | Goto2 => return Self::r#goto(parse),
                    Let => return Self::r#let(parse),
                    Print1 | Print2 => return Self::r#print(parse),
                    Run => return Self::r#run(parse),
                    Data | Def | Dim | End | For | Gosub1 | Gosub2 | If | Input | Next | On
                    | Read | Restore | Return | Stop => {
                        dbg!(&word);
                        return Err(
                            error!(InternalError, ..&parse.column(); "STATEMENT NOT YET PARSING; PANIC"),
                        );
                    }
                    Else | Rem1 | Rem2 | To | Then => {}
                }
            }
            _ => {}
        }
        Err(error!(SyntaxError, ..&parse.column(); "EXPECTED STATEMENT"))
    }

    fn r#let(parse: &mut Parser) -> Result<Statement> {
        let column = parse.column();
        let ident = parse.expect_ident()?;
        parse.expect(Token::Operator(Operator::Equal))?;
        let expr = parse.expect_expression()?;
        Ok(Statement::Let(column, ident, expr))
    }

    fn r#print(parse: &mut Parser) -> Result<Statement> {
        Ok(Statement::Print(parse.column(), parse.expect_print_list()?))
    }

    fn r#goto(parse: &mut Parser) -> Result<Statement> {
        Ok(Statement::Goto(parse.column(), parse.expect_expression()?))
    }

    fn r#run(parse: &mut Parser) -> Result<Statement> {
        Ok(Statement::Run(parse.column()))
    }
}

#[cfg(test)]
mod tests {
    use super::super::lex::*;
    use super::*;

    fn parse_str(s: &str) -> Option<Statement> {
        let (lin, tokens) = lex(s);
        match parse(lin, &tokens) {
            Ok(mut v) => v.pop(),
            Err(_) => None,
        }
    }

    #[test]
    fn test_let_foo_eq_bar() {
        let answer = Statement::Let(
            0..3,
            (3..6, Ident::Plain("TER".to_string())),
            Expression::Var(7..10, Ident::Plain("BAR".to_string())),
        );
        assert_eq!(parse_str("letter=bar:"), Some(answer));
        let answer = Statement::Let(
            0..3,
            (0..3, Ident::Plain("TER".to_string())),
            Expression::Var(4..7, Ident::Plain("BAR".to_string())),
        );
        assert_eq!(parse_str("ter=bar:"), Some(answer));
    }

    #[test]
    fn test_literals() {
        let answer = Statement::Let(
            0..1,
            (0..1, Ident::Plain("A".to_string())),
            Expression::Integer(2..4, 12),
        );
        assert_eq!(parse_str("A=12"), Some(answer));
        let answer = Statement::Let(
            0..1,
            (0..1, Ident::Plain("A".to_string())),
            Expression::Single(2..5, 12.0),
        );
        assert_eq!(parse_str("A=12!"), Some(answer));
        let answer = Statement::Let(
            0..1,
            (0..1, Ident::Plain("A".to_string())),
            Expression::Double(2..6, 12e4),
        );
        assert_eq!(parse_str("A=12d4"), Some(answer));
        let answer = Statement::Let(
            0..1,
            (0..1, Ident::Plain("A".to_string())),
            Expression::String(2..8, "food".to_string()),
        );
        assert_eq!(parse_str("A=\"food\""), Some(answer));
        let answer = Statement::Let(
            0..1,
            (0..1, Ident::Plain("A".to_string())),
            Expression::Double(2..58, 0.0),
        );
        assert_eq!(
            parse_str("A=798347598234765983475983248592d-234721398742391847982344"),
            Some(answer)
        );
    }

    #[test]
    fn test_unary() {
        let answer = Statement::Let(
            0..1,
            (0..1, Ident::Plain("A".to_string())),
            Expression::Negation(
                2..3,
                Box::new(Expression::Add(
                    5..6,
                    Box::new(Expression::Integer(4..5, 1)),
                    Box::new(Expression::Integer(7..8, 1)),
                )),
            ),
        );
        assert_eq!(parse_str("A=-(1++1)"), Some(answer));
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
        assert_eq!(parse_str("A=cos(3.14)"), Some(answer));
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
        assert_eq!(parse_str("let A=(2-(3+cos(3.14))*4)"), Some(answer));
    }

    #[test]
    fn test_printer_list() {
        let (lin, tokens) = lex("? 1 2,3;:?");
        assert_eq!(
            parse(lin, &tokens).ok(),
            Some(vec!(
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
            ))
        );
    }
}
