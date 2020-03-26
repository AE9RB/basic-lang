use super::token::{Literal, Operator, Token, Word};
use super::{ast::*, Column, Error, LineNumber, MaxValue};
use crate::error;

type Result<T> = std::result::Result<T, Error>;

pub fn parse(line_number: LineNumber, tokens: &[Token]) -> Result<Vec<Statement>> {
    match BasicParser::parse(tokens) {
        Err(e) => Err(e.in_line_number(line_number)),
        Ok(r) => Ok(r),
    }
}

struct BasicParser<'a> {
    token_stream: std::slice::Iter<'a, Token>,
    peeked: Option<&'a Token>,
    rem2: bool,
    col: Column,
}

impl From<super::token::Ident> for Ident {
    fn from(ident: super::token::Ident) -> Self {
        use super::token::Ident::*;
        match ident {
            Plain(s) => Ident::Plain(s.into()),
            String(s) => Ident::Plain(s.into()),
            Single(s) => Ident::Plain(s.into()),
            Double(s) => Ident::Plain(s.into()),
            Integer(s) => Ident::Plain(s.into()),
        }
    }
}

impl<'a> BasicParser<'a> {
    fn parse(tokens: &'a [Token]) -> Result<Vec<Statement>> {
        let mut parse = BasicParser {
            token_stream: tokens.iter(),
            peeked: None,
            rem2: false,
            col: 0..0,
        };
        match parse.peek() {
            Some(Token::Literal(Literal::Integer(_)))
            | Some(Token::Literal(Literal::Single(_)))
            | Some(Token::Literal(Literal::Double(_))) => {
                return Err(error!(UndefinedLine, ..&parse.col; "INVALID LINE NUMBER"));
            }
            _ => {}
        }
        let mut statements: Vec<Statement> = vec![];
        loop {
            match parse.peek() {
                None | Some(Token::Word(Word::Rem1)) => return Ok(statements),
                Some(_) => {
                    statements.append(&mut parse.expect_statements()?);
                }
            }
        }
    }

    fn next(&mut self) -> Option<&'a Token> {
        if self.peeked.is_some() {
            return self.peeked.take();
        }
        loop {
            self.col.start = self.col.end;
            let token = self.token_stream.next()?;
            self.col.end += token.to_string().chars().count();
            if self.rem2 {
                continue;
            }
            match token {
                Token::Word(Word::Rem2) => {
                    self.rem2 = true;
                    continue;
                }
                Token::Whitespace(_) => continue,
                _ => return Some(token),
            }
        }
    }

    fn peek(&mut self) -> Option<&&'a Token> {
        if self.peeked.is_none() {
            self.peeked = self.next();
        }
        self.peeked.as_ref()
    }

    fn expect_statements(&mut self) -> Result<Vec<Statement>> {
        let mut statements: Vec<Statement> = vec![];
        let mut expect_colon = false;
        loop {
            match self.peek() {
                None | Some(Token::Word(Word::Rem1)) | Some(Token::Word(Word::Else)) => {
                    return Ok(statements)
                }
                Some(Token::Colon) => {
                    expect_colon = false;
                    self.next();
                    continue;
                }
                Some(_) => {
                    if expect_colon {
                        return Err(error!(SyntaxError, ..&self.col; "UNEXPECTED TOKEN"));
                    }
                    statements.append(&mut Statement::expect(self)?);
                    expect_colon = true;
                }
            }
        }
    }

    fn expect_expression(&mut self) -> Result<Expression> {
        Expression::expect(self)
    }

    fn expect_expression_list(&mut self) -> Result<Vec<Expression>> {
        self.expect(Token::LParen)?;
        let mut expressions: Vec<Expression> = vec![];
        if let Some(Token::RParen) = self.peek() {
            self.next();
            return Ok(expressions);
        }
        loop {
            expressions.push(self.expect_expression()?);
            match self.next() {
                Some(Token::RParen) => return Ok(expressions),
                Some(Token::Comma) => continue,
                _ => {
                    return Err(
                        error!(SyntaxError, ..&self.col; "EXPECTED RIGHT PARENTHESIS OR COMMA"),
                    )
                }
            }
        }
    }

    fn expect_print_list(&mut self) -> Result<Vec<Expression>> {
        let mut expressions: Vec<Expression> = vec![];
        let mut linefeed = true;
        loop {
            match self.peek() {
                None | Some(Token::Colon) | Some(Token::Word(Word::Else)) => {
                    let mut column = self.col.clone();
                    column.end = column.start;
                    if linefeed {
                        expressions.push(Expression::String(column, "\n".into()));
                    }
                    return Ok(expressions);
                }
                Some(Token::Semicolon) => {
                    linefeed = false;
                    self.next();
                }
                Some(Token::Comma) => {
                    linefeed = false;
                    self.next();
                    expressions.push(Expression::Function(
                        self.col.clone(),
                        Ident::String("TAB".into()),
                        vec![Expression::Integer(self.col.clone(), -14)],
                    ));
                }
                _ => {
                    linefeed = true;
                    expressions.push(self.expect_expression()?);
                }
            };
        }
    }

    fn expect_var(&mut self) -> Result<Variable> {
        let ident = if let Some(Token::Ident(ident)) = self.next() {
            ident.clone()
        } else {
            return Err(error!(SyntaxError, ..&self.col; "EXPECTED VARIABLE"));
        };
        let col = self.col.clone();
        if format!("{}", ident).starts_with("FN") {
            return Err(error!(SyntaxError, ..&col; "FN RESERVED FOR FUNCTIONS"));
        }
        match self.peek() {
            Some(Token::LParen) => {
                let vec_expr = self.expect_expression_list()?;
                Ok(Variable::Array(
                    col.start..self.col.end,
                    ident.into(),
                    vec_expr,
                ))
            }
            _ => Ok(Variable::Unary(col, ident.into())),
        }
    }

    fn expect_var_list(&mut self) -> Result<Vec<Variable>> {
        let mut vars: Vec<Variable> = vec![];
        let mut expecting = false;
        loop {
            match self.peek() {
                None | Some(Token::Colon) | Some(Token::Word(Word::Else)) if !expecting => break,
                _ => vars.push(self.expect_var()?),
            };
            if self.maybe(Token::Comma) {
                expecting = true;
            } else {
                break;
            }
        }
        Ok(vars)
    }

    fn expect_unary_var(&mut self) -> Result<Ident> {
        match self.expect_var()? {
            Variable::Array(col, ..) => Err(error!(SyntaxError, ..&col; "ARRAY NOT ALLOWED HERE")),
            Variable::Unary(_col, ident) => Ok(ident),
        }
    }

    fn expect_unary_var_list(&mut self) -> Result<Vec<Ident>> {
        let vars = self.expect_var_list()?;
        let mut vec_ident: Vec<Ident> = vec![];
        for var in vars {
            match var {
                Variable::Array(col, ..) => {
                    return Err(error!(SyntaxError, ..&col; "ARRAY NOT ALLOWED HERE"));
                }
                Variable::Unary(_col, ident) => vec_ident.push(ident),
            }
        }
        Ok(vec_ident)
    }

    fn maybe_line_number(&mut self) -> Result<LineNumber> {
        if let Some(str) = match self.peek() {
            Some(Token::Literal(Literal::Integer(s))) => Some(s),
            Some(Token::Literal(Literal::Single(s))) => Some(s),
            Some(Token::Literal(Literal::Double(s))) => Some(s),
            _ => None,
        } {
            self.next();
            if let Ok(num) = str.parse::<u16>() {
                if num <= LineNumber::max_value() {
                    return Ok(Some(num));
                }
            }
            return Err(error!(UndefinedLine, ..&self.col; "INVALID LINE NUMBER"));
        }
        Ok(None)
    }

    fn expect_line_number(&mut self) -> Result<Expression> {
        match self.maybe_line_number()? {
            Some(num) => Ok(Expression::Single(self.col.clone(), num as f32)),
            None => Err(error!(SyntaxError, ..&self.col; "EXPECTED LINE NUMBER")),
        }
    }

    fn expect_line_number_list(&mut self) -> Result<Vec<Expression>> {
        let mut vars: Vec<Expression> = vec![];
        let mut expecting = false;
        loop {
            match self.peek() {
                None | Some(Token::Colon) | Some(Token::Word(Word::Else)) if !expecting => break,
                _ => vars.push(self.expect_line_number()?),
            };
            if self.maybe(Token::Comma) {
                expecting = true;
            } else {
                break;
            }
        }
        Ok(vars)
    }

    fn expect_line_number_range(&mut self) -> Result<(Expression, Expression)> {
        let from;
        let from_num;
        let to;
        if let Some(num) = self.maybe_line_number()? {
            from_num = num as f32;
            from = Expression::Single(self.col.clone(), num as f32);
        } else {
            from_num = LineNumber::max_value() as f32;
            let col = self.col.start..self.col.start;
            from = Expression::Single(col, 0.0);
        };
        if let Some(&&Token::Operator(Operator::Minus)) = self.peek() {
            self.next();
            if let Some(ln) = self.maybe_line_number()? {
                to = Expression::Single(self.col.clone(), ln as f32);
            } else {
                let col = self.col.start..self.col.start;
                to = Expression::Single(col, LineNumber::max_value() as f32);
            };
        } else {
            let col = self.col.start..self.col.start;
            to = Expression::Single(col, from_num);
        }
        Ok((from, to))
    }

    fn maybe(&mut self, token: Token) -> bool {
        if let Some(t) = self.peek() {
            if **t == token {
                self.next();
                return true;
            }
        }
        false
    }

    fn expect(&mut self, token: Token) -> Result<()> {
        if let Some(t) = self.next() {
            if *t == token {
                return Ok(());
            }
        }
        Err(error!(SyntaxError, ..&self.col;
            match token {
                Token::Unknown(_) | Token::Whitespace(_) => {"PANIC"}
                Token::Literal(_) => {"EXPECTED LITERAL"}
                Token::Word(Word::To) => {"EXPECTED TO"}
                Token::Word(_) => {"EXPECTED STATEMENT WORD"}
                Token::Operator(Operator::Equal) => {"EXPECTED EQUALS SIGN"}
                Token::Operator(_) => {"EXPECTED OPERATOR"}
                Token::Ident(_) => {"EXPECTED IDENTIFIER"}
                Token::LParen => {"EXPECTED LEFT PARENTHESIS"}
                Token::RParen => {"EXPECTED RIGHT PARENTHESIS"}
                Token::Comma => {"EXPECTED COMMA"}
                Token::Colon => {"EXPECTED COLON"}
                Token::Semicolon => {"EXPECTED SEMICOLON"}
            }
        ))
    }
}

impl Expression {
    fn expect(parse: &mut BasicParser) -> Result<Expression> {
        fn descend(parse: &mut BasicParser, precedence: usize) -> Result<Expression> {
            let mut lhs = match parse.next() {
                Some(Token::LParen) => {
                    let expr = descend(parse, 0)?;
                    parse.expect(Token::RParen)?;
                    expr
                }
                Some(Token::Ident(tok_ident)) => {
                    let ident = tok_ident.clone();
                    let col = parse.col.clone();
                    match parse.peek() {
                        Some(&&Token::LParen) => {
                            let vec_expr = parse.expect_expression_list()?;
                            let col = col.start..parse.col.end;
                            Expression::Function(col, ident.into(), vec_expr)
                        }
                        _ => Expression::UnaryVar(col, ident.into()),
                    }
                }
                Some(Token::Operator(Operator::Plus)) => {
                    let op_prec = Expression::unary_op_prec(&Operator::Plus);
                    descend(parse, op_prec)?
                }
                Some(Token::Operator(Operator::Minus)) => {
                    let col = parse.col.clone();
                    let op_prec = Expression::unary_op_prec(&Operator::Minus);
                    let expr = descend(parse, op_prec)?;
                    Expression::Negation(col, Box::new(expr))
                }
                Some(Token::Literal(lit)) => Expression::for_literal(parse.col.clone(), lit)?,
                _ => return Err(error!(SyntaxError, ..&parse.col; "EXPECTED EXPRESSION")),
            };
            let mut rhs;
            while let Some(Token::Operator(op)) = parse.peek() {
                let op_prec = Expression::binary_op_prec(op);
                if op_prec <= precedence {
                    break;
                }
                parse.next();
                let column = parse.col.clone();
                rhs = descend(parse, op_prec)?;
                lhs = Expression::for_binary_op(column, op, lhs, rhs)?;
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
            Caret => Expression::Exponentiation(col, Box::new(lhs), Box::new(rhs)),
            Multiply => Expression::Multiply(col, Box::new(lhs), Box::new(rhs)),
            Divide => Expression::Divide(col, Box::new(lhs), Box::new(rhs)),
            DivideInt => Expression::DivideInt(col, Box::new(lhs), Box::new(rhs)),
            Modulus => Expression::Modulus(col, Box::new(lhs), Box::new(rhs)),
            Plus => Expression::Add(col, Box::new(lhs), Box::new(rhs)),
            Minus => Expression::Subtract(col, Box::new(lhs), Box::new(rhs)),
            Equal => Expression::Equal(col, Box::new(lhs), Box::new(rhs)),
            NotEqual => Expression::NotEqual(col, Box::new(lhs), Box::new(rhs)),
            Less => Expression::Less(col, Box::new(lhs), Box::new(rhs)),
            LessEqual => Expression::LessEqual(col, Box::new(lhs), Box::new(rhs)),
            EqualLess => Expression::LessEqual(col, Box::new(lhs), Box::new(rhs)),
            Greater => Expression::Greater(col, Box::new(lhs), Box::new(rhs)),
            GreaterEqual => Expression::GreaterEqual(col, Box::new(lhs), Box::new(rhs)),
            EqualGreater => Expression::GreaterEqual(col, Box::new(lhs), Box::new(rhs)),
            Not => Expression::Not(col, Box::new(lhs), Box::new(rhs)),
            And => Expression::And(col, Box::new(lhs), Box::new(rhs)),
            Or => Expression::Or(col, Box::new(lhs), Box::new(rhs)),
            Xor => Expression::Xor(col, Box::new(lhs), Box::new(rhs)),
            Imp => Expression::Imp(col, Box::new(lhs), Box::new(rhs)),
            Eqv => Expression::Eqv(col, Box::new(lhs), Box::new(rhs)),
        })
    }

    fn unary_op_prec(op: &Operator) -> usize {
        use Operator::*;
        match op {
            Plus | Minus => 12,
            _ => {
                debug_assert!(false, "NOT A UNARY OP");
                0
            }
        }
    }

    fn binary_op_prec(op: &Operator) -> usize {
        use Operator::*;
        match op {
            Caret => 13,
            // Unary identity and negation => 12
            Multiply | Divide => 11,
            DivideInt => 10,
            Modulus => 9,
            Plus | Minus => 8,
            Equal | NotEqual | Less | LessEqual | EqualLess | Greater | GreaterEqual
            | EqualGreater => 7,
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
                Err(_) => Err(error!(TypeMismatch, ..&col)),
            }
        }
        match lit {
            Literal::Single(s) => Ok(Expression::Single(col.clone(), parse(col, s)?)),
            Literal::Double(s) => Ok(Expression::Double(col.clone(), parse(col, s)?)),
            Literal::Integer(s) => Ok(Expression::Integer(col.clone(), parse(col, s)?)),
            Literal::String(s) => {
                if s.chars().count() > 255 {
                    Err(error!(StringTooLong, ..&col; "MAXIMUM LITERAL LENGTH IS 255"))
                } else {
                    Ok(Expression::String(col, s.clone().into()))
                }
            }
        }
    }
}

impl Statement {
    fn expect(parse: &mut BasicParser) -> Result<Vec<Statement>> {
        match parse.peek() {
            Some(Token::Ident(_)) => return Ok(vec![Self::r#let(parse)?]),
            Some(Token::Word(word)) => {
                parse.next();
                use Word::*;
                match word {
                    Clear => return Ok(vec![Self::r#clear(parse)?]),
                    Cont => return Ok(vec![Self::r#cont(parse)?]),
                    Dim => return Self::r#dim(parse),
                    End => return Ok(vec![Self::r#end(parse)?]),
                    For => return Ok(vec![Self::r#for(parse)?]),
                    Gosub1 | Gosub2 => return Ok(vec![Self::r#gosub(parse)?]),
                    Goto1 | Goto2 => return Ok(vec![Self::r#goto(parse)?]),
                    If => return Ok(vec![Self::r#if(parse)?]),
                    Input => return Ok(vec![Self::r#input(parse)?]),
                    Let => return Ok(vec![Self::r#let(parse)?]),
                    List => return Ok(vec![Self::r#list(parse)?]),
                    New => return Ok(vec![Self::r#new(parse)?]),
                    Next => return Self::r#next(parse),
                    On => return Ok(vec![Self::r#on(parse)?]),
                    Print1 | Print2 => return Self::r#print(parse),
                    Return => return Ok(vec![Self::r#return(parse)?]),
                    Run => return Ok(vec![Self::r#run(parse)?]),
                    Stop => return Ok(vec![Self::r#stop(parse)?]),
                    Else | Rem1 | Rem2 | Step | Then | To => {}
                }
            }
            _ => {}
        }
        Err(error!(SyntaxError, ..&parse.col; "EXPECTED STATEMENT"))
    }

    fn r#clear(parse: &mut BasicParser) -> Result<Statement> {
        let result = Ok(Statement::Clear(parse.col.clone()));
        while match parse.peek() {
            None | Some(Token::Colon) | Some(Token::Word(Word::Else)) => false,
            _ => true,
        } {
            parse.next();
        }
        result
    }

    fn r#cont(parse: &mut BasicParser) -> Result<Statement> {
        Ok(Statement::Cont(parse.col.clone()))
    }

    fn r#dim(parse: &mut BasicParser) -> Result<Vec<Statement>> {
        let column = parse.col.clone();
        let mut v: Vec<Statement> = vec![];
        let var_list = parse.expect_var_list()?;
        if var_list.is_empty() {
            return Err(error!(SyntaxError, ..&column; "EXPECTED ARRAY LIST"));
        }
        for var in var_list {
            v.push(Statement::Dim(column.clone(), var));
        }
        Ok(v)
    }

    fn r#end(parse: &mut BasicParser) -> Result<Statement> {
        Ok(Statement::End(parse.col.clone()))
    }

    fn r#for(parse: &mut BasicParser) -> Result<Statement> {
        let column = parse.col.clone();
        let ident = parse.expect_unary_var()?;
        parse.expect(Token::Operator(Operator::Equal))?;
        let expr_from = parse.expect_expression()?;
        parse.expect(Token::Word(Word::To))?;
        let expr_to = parse.expect_expression()?;
        let mut expr_step = Expression::Integer(parse.col.end..parse.col.end, 1);
        if parse.maybe(Token::Word(Word::Step)) {
            expr_step = parse.expect_expression()?
        }
        Ok(Statement::For(column, ident, expr_from, expr_to, expr_step))
    }

    fn r#gosub(parse: &mut BasicParser) -> Result<Statement> {
        Ok(Statement::Gosub(
            parse.col.clone(),
            parse.expect_line_number()?,
        ))
    }

    fn r#goto(parse: &mut BasicParser) -> Result<Statement> {
        Ok(Statement::Goto(
            parse.col.clone(),
            parse.expect_line_number()?,
        ))
    }

    fn r#if(parse: &mut BasicParser) -> Result<Statement> {
        let column = parse.col.clone();
        let predicate = parse.expect_expression()?;
        if parse.maybe(Token::Word(Word::Goto1)) || parse.maybe(Token::Word(Word::Goto2)) {
            let stmt = Statement::Goto(parse.col.clone(), parse.expect_line_number()?);
            return Ok(Statement::If(column, predicate, vec![stmt], vec![]));
        }
        parse.expect(Token::Word(Word::Then))?;
        let then_stmt = match parse.maybe_line_number()? {
            Some(n) => vec![Statement::Goto(
                column.clone(),
                Expression::Single(parse.col.clone(), n as f32),
            )],
            None => parse.expect_statements()?,
        };
        let else_stmt = if parse.maybe(Token::Word(Word::Else)) {
            match parse.maybe_line_number()? {
                Some(n) => vec![Statement::Goto(
                    column.clone(),
                    Expression::Single(parse.col.clone(), n as f32),
                )],
                None => parse.expect_statements()?,
            }
        } else {
            vec![]
        };
        Ok(Statement::If(column, predicate, then_stmt, else_stmt))
    }

    fn r#input(parse: &mut BasicParser) -> Result<Statement> {
        let column = parse.col.clone();
        let mut prompt_col = column.end..column.end;
        let caps = if let Some(Token::Comma) = parse.peek() {
            parse.next();
            Expression::Integer(parse.col.clone(), 0)
        } else {
            Expression::Integer(parse.col.start..parse.col.start, -1)
        };
        let prompt = match parse.peek() {
            Some(Token::Literal(Literal::String(s))) => {
                parse.next();
                prompt_col = parse.col.clone();
                match parse.peek() {
                    None | Some(Token::Colon) | Some(Token::Word(Word::Else)) => {}
                    Some(Token::Semicolon) => {
                        parse.next();
                    }
                    _ => {
                        return Err(error!(SyntaxError, ..&parse.col.clone(); "UNEXPECTED TOKEN"));
                    }
                }
                s.clone()
            }
            _ => String::new(),
        };
        let idents = parse.expect_var_list()?;
        if idents.is_empty() {
            return Err(error!(SyntaxError, ..&parse.col.clone(); "MISSING VARIABLE LIST"));
        }
        Ok(Statement::Input(
            column,
            caps,
            Expression::String(prompt_col, prompt.into()),
            idents,
        ))
    }

    fn r#let(parse: &mut BasicParser) -> Result<Statement> {
        let column = parse.col.clone();
        let var = parse.expect_var()?;
        parse.expect(Token::Operator(Operator::Equal))?;
        let expr = parse.expect_expression()?;
        Ok(Statement::Let(column, var, expr))
    }

    fn r#list(parse: &mut BasicParser) -> Result<Statement> {
        let column = parse.col.clone();
        let (from, to) = parse.expect_line_number_range()?;
        Ok(Statement::List(column, from, to))
    }

    fn r#new(parse: &mut BasicParser) -> Result<Statement> {
        Ok(Statement::New(parse.col.clone()))
    }

    fn r#next(parse: &mut BasicParser) -> Result<Vec<Statement>> {
        let column = parse.col.clone();
        let mut idents = parse.expect_unary_var_list()?;
        if idents.is_empty() {
            return Ok(vec![Statement::Next(column, Ident::Plain("".into()))]);
        }
        Ok(idents
            .drain(..)
            .map(|i| Statement::Next(column.clone(), i))
            .collect::<Vec<Statement>>())
    }

    fn r#on(parse: &mut BasicParser) -> Result<Statement> {
        let column = parse.col.clone();
        let var = parse.expect_var()?;
        match parse.next() {
            Some(Token::Word(Word::Goto1)) => Ok(Statement::OnGoto(
                column,
                var,
                parse.expect_line_number_list()?,
            )),
            Some(Token::Word(Word::Gosub1)) => Ok(Statement::OnGosub(
                column,
                var,
                parse.expect_line_number_list()?,
            )),
            _ => Err(error!(SyntaxError, ..&parse.col; "EXPECTED GOTO OR GOSUB")),
        }
    }

    fn r#print(parse: &mut BasicParser) -> Result<Vec<Statement>> {
        let column = parse.col.clone();
        let mut vec_expr = parse.expect_print_list()?;
        Ok(vec_expr
            .drain(..)
            .map(|e| Statement::Print(column.clone(), e))
            .collect())
    }

    fn r#return(parse: &mut BasicParser) -> Result<Statement> {
        Ok(Statement::Return(parse.col.clone()))
    }

    fn r#run(parse: &mut BasicParser) -> Result<Statement> {
        let column = parse.col.clone();
        match parse.maybe_line_number()? {
            Some(num) => Ok(Statement::Run(
                column,
                Expression::Single(parse.col.clone(), num as f32),
            )),
            None => {
                let empty = parse.col.clone();
                let empty = empty.start..empty.start;
                Ok(Statement::Run(column, Expression::Single(empty, -1.0)))
            }
        }
    }

    fn r#stop(parse: &mut BasicParser) -> Result<Statement> {
        Ok(Statement::Stop(parse.col.clone()))
    }
}
