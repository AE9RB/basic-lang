use super::token::{self, Literal, Operator, Token, Word};
use super::{ast::*, Column, Error, LineNumber, MaxValue};
use crate::error;
use std::collections::HashMap;

type Result<T> = std::result::Result<T, Error>;

const FN_RESERVED: &str = "FN RESERVED FOR FUNCTIONS";
const ARRAY_NOT_ALLOWED: &str = "ARRAY NOT ALLOWED";
const EXPECTED_VARIABLE: &str = "EXPECTED VARIABLE";

pub fn parse(line_number: LineNumber, tokens: &[Token]) -> Result<Vec<Statement>> {
    match BasicParser::parse(tokens) {
        Err(e) => Err(e.in_line_number(line_number)),
        Ok(r) => Ok(r),
    }
}

struct BasicParser<'a> {
    token_stream: std::slice::Iter<'a, Token>,
    peeked: Option<&'a Token>,
    rem: bool,
    col: Column,
}

impl<'a> BasicParser<'a> {
    fn parse(tokens: &'a [Token]) -> Result<Vec<Statement>> {
        let mut parse = BasicParser {
            token_stream: tokens.iter(),
            peeked: None,
            rem: false,
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
        parse.expect_statements()
    }

    fn next(&mut self) -> Option<&'a Token> {
        if self.peeked.is_some() {
            return self.peeked.take();
        }
        loop {
            self.col.start = self.col.end;
            let token = self.token_stream.next()?;
            if matches!(token, Token::Word(Word::Rem1) | Token::Word(Word::Rem2)) {
                self.rem = true;
            }
            if self.rem {
                continue;
            }
            self.col.end += token.to_string().chars().count();
            match token {
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
                None | Some(Token::Word(Word::Else)) => return Ok(statements),
                Some(Token::Colon) => {
                    expect_colon = false;
                    self.next();
                    continue;
                }
                Some(_) => {
                    if expect_colon {
                        return Err(error!(SyntaxError, ..&self.col; "UNEXPECTED TOKEN"));
                    }
                    statements.push(Statement::expect(self)?);
                    expect_colon = true;
                }
            }
        }
    }

    fn expect_expression(&mut self) -> Result<Expression> {
        self.expect_fn_expression(&HashMap::default())
    }

    fn expect_expression_list(&mut self) -> Result<Vec<Expression>> {
        self.expect_fn_expression_list(&HashMap::default())
    }

    fn expect_fn_expression(
        &mut self,
        var_map: &HashMap<token::Ident, Variable>,
    ) -> Result<Expression> {
        Expression::expect(self, var_map)
    }

    fn expect_fn_expression_list(
        &mut self,
        var_map: &HashMap<token::Ident, Variable>,
    ) -> Result<Vec<Expression>> {
        let mut expressions: Vec<Expression> = vec![];
        loop {
            expressions.push(self.expect_fn_expression(var_map)?);
            if self.maybe(Token::Comma) {
                continue;
            }
            return Ok(expressions);
        }
    }

    fn expect_print_list(&mut self) -> Result<Vec<Expression>> {
        let mut expressions: Vec<Expression> = vec![];
        let mut linefeed = true;
        loop {
            match self.peek() {
                None | Some(Token::Colon) | Some(Token::Word(Word::Else)) => {
                    let mut column = self.col.clone();
                    column.start = column.end;
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
                    expressions.push(Expression::Variable(Variable::Array(
                        self.col.clone(),
                        Ident::String("TAB".into()),
                        vec![Expression::Integer(self.col.clone(), -14)],
                    )));
                }
                _ => {
                    linefeed = true;
                    expressions.push(self.expect_expression()?);
                }
            };
        }
    }

    fn expect_ident(&mut self) -> Result<(Column, token::Ident)> {
        let ident = if let Some(Token::Ident(ident)) = self.next() {
            ident.clone()
        } else {
            return Err(error!(SyntaxError, ..&self.col; EXPECTED_VARIABLE));
        };
        let col = self.col.clone();
        if ident.is_user_function() {
            return Err(error!(SyntaxError, ..&col; FN_RESERVED));
        }
        if let Some(Token::LParen) = self.peek() {
            return Err(error!(SyntaxError, ..&(col); ARRAY_NOT_ALLOWED));
        }
        Ok((col, ident))
    }

    fn expect_ident_list(&mut self) -> Result<Vec<(Column, token::Ident)>> {
        let mut idents: Vec<(Column, token::Ident)> = vec![];
        let mut expecting = false;
        loop {
            match self.peek() {
                None | Some(Token::Colon) | Some(Token::Word(Word::Else)) if !expecting => break,
                _ => idents.push(self.expect_ident()?),
            };
            if self.maybe(Token::Comma) {
                expecting = true;
            } else {
                break;
            }
        }
        Ok(idents)
    }

    fn expect_var(&mut self) -> Result<Variable> {
        let ident = if let Some(Token::Ident(ident)) = self.next() {
            ident.clone()
        } else {
            return Err(error!(SyntaxError, ..&self.col; EXPECTED_VARIABLE));
        };
        let col = self.col.clone();
        if ident.is_user_function() {
            return Err(error!(SyntaxError, ..&col; FN_RESERVED));
        }
        match self.peek() {
            Some(Token::LParen) => {
                self.expect(Token::LParen)?;
                let vec_expr = self.expect_expression_list()?;
                self.expect(Token::RParen)?;
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
        let mut vec_var: Vec<Variable> = vec![];
        loop {
            vec_var.push(self.expect_var()?);
            if self.maybe(Token::Comma) {
                continue;
            }
            break;
        }
        Ok(vec_var)
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
        let mut to_num;
        let col = self.col.clone();
        if let Some(num) = self.maybe_line_number()? {
            from_num = num as f32;
            to_num = from_num;
            from = Expression::Single(self.col.clone(), from_num);
        } else {
            from_num = 0.0;
            to_num = LineNumber::max_value() as f32;
            from = Expression::Single(self.col.start..self.col.start, from_num);
        };
        if self.maybe(Token::Operator(Operator::Minus)) {
            if let Some(ln) = self.maybe_line_number()? {
                to_num = ln as f32;
                to = Expression::Single(self.col.clone(), to_num);
            } else {
                to_num = LineNumber::max_value() as f32;
                to = Expression::Single(self.col.start..self.col.start, to_num);
            }
        } else {
            to = Expression::Single(self.col.start..self.col.start, to_num);
        }
        if from_num > to_num {
            return Err(error!(UndefinedLine, ..&(col.start..self.col.end); "INVALID RANGE"));
        }
        Ok((from, to))
    }

    fn expect_var_range(&mut self) -> Result<(Variable, Variable)> {
        let (from_col, from_ident) = self.expect_ident()?;
        let (to_col, to_ident) = if self.maybe(Token::Operator(Operator::Minus)) {
            self.expect_ident()?
        } else {
            (from_col.clone(), from_ident.clone())
        };
        let from_char = match &from_ident {
            token::Ident::Plain(s) if s.len() == 1 => s,
            _ => return Err(error!(SyntaxError, ..&from_col)),
        };
        let to_char = match &to_ident {
            token::Ident::Plain(s) if s.len() == 1 => s,
            _ => return Err(error!(SyntaxError, ..&to_col)),
        };
        if from_char > to_char {
            return Err(error!(SyntaxError, ..&(from_col.start..to_col.end)));
        }
        let from = Variable::Unary(from_col, from_ident.into());
        let to = Variable::Unary(to_col, to_ident.into());
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
                Token::Unknown(_) | Token::Whitespace(_) => {"EXPECTED THE IMPOSSIBLE"}
                Token::Literal(_) => {"EXPECTED LITERAL"}
                Token::Word(Word::Then) => {"EXPECTED THEN"}
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
    fn expect(
        parse: &mut BasicParser,
        var_map: &HashMap<token::Ident, Variable>,
    ) -> Result<Expression> {
        fn descend(
            parse: &mut BasicParser,
            var_map: &HashMap<token::Ident, Variable>,
            precedence: usize,
        ) -> Result<Expression> {
            let mut lhs = match parse.next() {
                Some(Token::LParen) => {
                    let expr = descend(parse, var_map, 0)?;
                    parse.expect(Token::RParen)?;
                    expr
                }
                Some(Token::Ident(tok_ident)) => {
                    let ident = tok_ident.clone();
                    let col = parse.col.clone();
                    match parse.peek() {
                        Some(&&Token::LParen) => {
                            parse.expect(Token::LParen)?;
                            let mut vec_expr = vec![];
                            if !parse.maybe(Token::RParen) {
                                vec_expr = parse.expect_fn_expression_list(var_map)?;
                                parse.expect(Token::RParen)?;
                            }
                            let col = col.start..parse.col.end;
                            Expression::Variable(Variable::Array(col, ident.into(), vec_expr))
                        }
                        _ => {
                            if ident.is_user_function() {
                                return Err(error!(SyntaxError, ..&col; FN_RESERVED));
                            }
                            match var_map.get(&ident) {
                                Some(var) => Expression::Variable(var.clone()),
                                None => Expression::Variable(Variable::Unary(col, ident.into())),
                            }
                        }
                    }
                }
                Some(Token::Operator(Operator::Plus)) => {
                    let op_prec = Expression::unary_op_precedence(&Operator::Plus)?;
                    descend(parse, var_map, op_prec)?
                }
                Some(Token::Operator(Operator::Minus)) => {
                    let col = parse.col.clone();
                    let op_prec = Expression::unary_op_precedence(&Operator::Minus)?;
                    let expr = descend(parse, var_map, op_prec)?;
                    Expression::Negation(col, Box::new(expr))
                }
                Some(Token::Operator(Operator::Not)) => {
                    let col = parse.col.clone();
                    let op_prec = Expression::unary_op_precedence(&Operator::Not)?;
                    let expr = descend(parse, var_map, op_prec)?;
                    Expression::Not(col, Box::new(expr))
                }
                Some(Token::Literal(lit)) => Expression::literal(parse.col.clone(), lit)?,
                _ => return Err(error!(SyntaxError, ..&parse.col; "EXPECTED EXPRESSION")),
            };
            let mut rhs;
            while let Some(Token::Operator(op)) = parse.peek() {
                let op_prec = Expression::binary_op_precedence(op)?;
                if op_prec <= precedence {
                    break;
                }
                parse.next();
                let column = parse.col.clone();
                rhs = descend(parse, var_map, op_prec)?;
                lhs = Expression::binary_op(column, op, lhs, rhs)?;
            }
            Ok(lhs)
        }
        descend(parse, var_map, 0)
    }

    fn binary_op(
        col: Column,
        op: &Operator,
        lhs: Expression,
        rhs: Expression,
    ) -> Result<Expression> {
        use Operator::*;
        Ok(match op {
            Caret => Expression::Power(col, Box::new(lhs), Box::new(rhs)),
            Multiply => Expression::Multiply(col, Box::new(lhs), Box::new(rhs)),
            Divide => Expression::Divide(col, Box::new(lhs), Box::new(rhs)),
            DivideInt => Expression::DivideInt(col, Box::new(lhs), Box::new(rhs)),
            Modulo => Expression::Modulo(col, Box::new(lhs), Box::new(rhs)),
            Plus => Expression::Add(col, Box::new(lhs), Box::new(rhs)),
            Minus => Expression::Subtract(col, Box::new(lhs), Box::new(rhs)),
            Equal => Expression::Equal(col, Box::new(lhs), Box::new(rhs)),
            NotEqual => Expression::NotEqual(col, Box::new(lhs), Box::new(rhs)),
            Less => Expression::Less(col, Box::new(lhs), Box::new(rhs)),
            LessEqual => Expression::LessEqual(col, Box::new(lhs), Box::new(rhs)),
            Greater => Expression::Greater(col, Box::new(lhs), Box::new(rhs)),
            GreaterEqual => Expression::GreaterEqual(col, Box::new(lhs), Box::new(rhs)),
            Not => return Err(error!(InternalError)),
            And => Expression::And(col, Box::new(lhs), Box::new(rhs)),
            Or => Expression::Or(col, Box::new(lhs), Box::new(rhs)),
            Xor => Expression::Xor(col, Box::new(lhs), Box::new(rhs)),
            Imp => Expression::Imp(col, Box::new(lhs), Box::new(rhs)),
            Eqv => Expression::Eqv(col, Box::new(lhs), Box::new(rhs)),
        })
    }

    fn unary_op_precedence(op: &Operator) -> Result<usize> {
        use Operator::*;
        Ok(match op {
            Plus | Minus => 12,
            Not => 6,
            _ => 0,
        })
    }

    fn binary_op_precedence(op: &Operator) -> Result<usize> {
        use Operator::*;
        Ok(match op {
            Caret => 13,
            // Unary identity and negation => 12
            Multiply | Divide => 11,
            DivideInt => 10,
            Modulo => 9,
            Plus | Minus => 8,
            Equal | NotEqual | Less | LessEqual | Greater | GreaterEqual => 7,
            // Unary not => 6
            And => 5,
            Or => 4,
            Xor => 3,
            Imp => 2,
            Eqv => 1,
            _ => 0,
        })
    }

    fn literal(col: Column, lit: &Literal) -> Result<Expression> {
        fn parse<T: std::str::FromStr>(col: Column, s: &str) -> Result<T> {
            let mut s = String::from(s).replace('D', "E");
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
        fn parse_radix(col: Column, src: &str, radix: u32) -> Result<Expression> {
            match i16::from_str_radix(src, radix) {
                Ok(num) => Ok(Expression::Integer(col, num)),
                Err(_) => Err(error!(Overflow, ..&col)),
            }
        }
        match lit {
            Literal::Hex(s) => parse_radix(col, s, 16),
            Literal::Octal(s) => parse_radix(col, s, 8),
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
    fn expect(parse: &mut BasicParser) -> Result<Statement> {
        match parse.peek() {
            Some(Token::Ident(_)) => return Self::r#let(parse, true),
            Some(Token::Word(word)) => {
                parse.next();
                use Word::*;
                match word {
                    Clear => return Self::r#clear(parse),
                    Cls => return Self::r#cls(parse),
                    Cont => return Self::r#cont(parse),
                    Data => return Self::r#data(parse),
                    Def => return Self::r#def(parse),
                    Defdbl => return Self::r#defdbl(parse),
                    Defint => return Self::r#defint(parse),
                    Defsng => return Self::r#defsng(parse),
                    Defstr => return Self::r#defstr(parse),
                    Delete => return Self::r#delete(parse),
                    Dim => return Self::r#dim(parse),
                    End => return Self::r#end(parse),
                    Erase => return Self::r#erase(parse),
                    For => return Self::r#for(parse),
                    Gosub => return Self::r#gosub(parse),
                    Goto => return Self::r#goto(parse),
                    If => return Self::r#if(parse),
                    Input => return Self::r#input(parse),
                    Let => return Self::r#let(parse, false),
                    List => return Self::r#list(parse),
                    Load => return Self::r#load(parse),
                    New => return Self::r#new(parse),
                    Next => return Self::r#next(parse),
                    On => return Self::r#on(parse),
                    Print => return Self::r#print(parse),
                    Read => return Self::r#read(parse),
                    Renum => return Self::r#renum(parse),
                    Restore => return Self::r#restore(parse),
                    Return => return Self::r#return(parse),
                    Run => return Self::r#run(parse),
                    Save => return Self::r#save(parse),
                    Stop => return Self::r#stop(parse),
                    Swap => return Self::r#swap(parse),
                    Troff => return Self::r#troff(parse),
                    Tron => return Self::r#tron(parse),
                    Wend => return Self::r#wend(parse),
                    While => return Self::r#while(parse),
                    Else | Rem1 | Rem2 | Step | Then | To => {}
                }
            }
            _ => {}
        }
        Err(error!(SyntaxError, ..&parse.col; "EXPECTED STATEMENT"))
    }

    fn r#clear(parse: &mut BasicParser) -> Result<Statement> {
        let result = Ok(Statement::Clear(parse.col.clone()));
        while !matches!(
            parse.peek(),
            None | Some(Token::Colon) | Some(Token::Word(Word::Else))
        ) {
            parse.next();
        }
        result
    }

    fn r#cls(parse: &mut BasicParser) -> Result<Statement> {
        Ok(Statement::Cls(parse.col.clone()))
    }

    fn r#cont(parse: &mut BasicParser) -> Result<Statement> {
        Ok(Statement::Cont(parse.col.clone()))
    }

    fn r#data(parse: &mut BasicParser) -> Result<Statement> {
        let vec_expr = parse.expect_expression_list()?;
        Ok(Statement::Data(parse.col.clone(), vec_expr))
    }

    fn r#def(parse: &mut BasicParser) -> Result<Statement> {
        let column = parse.col.clone();
        let fn_ident = if let Some(Token::Ident(ident)) = parse.next() {
            ident.clone()
        } else {
            return Err(error!(SyntaxError, ..&parse.col; EXPECTED_VARIABLE));
        };
        if !fn_ident.is_user_function() {
            return Err(error!(SyntaxError, ..&parse.col; "MUST START WITH FN"));
        }
        let fn_ident_col = parse.col.clone();
        parse.expect(Token::LParen)?;
        let mut ident_list = parse.expect_ident_list()?;
        parse.expect(Token::RParen)?;
        parse.expect(Token::Operator(Operator::Equal))?;
        let mut var_map: HashMap<token::Ident, Variable> = HashMap::default();
        let var_ident: Vec<Variable> = ident_list
            .drain(..)
            .map(|(tok_col, tok_ident)| {
                let ast_ident = Ident::from((&fn_ident, &tok_ident));
                var_map.insert(
                    tok_ident,
                    Variable::Unary(tok_col.clone(), ast_ident.clone()),
                );
                Variable::Unary(tok_col, ast_ident)
            })
            .collect();
        let expr = parse.expect_fn_expression(&var_map)?;
        let var = Variable::Unary(fn_ident_col, fn_ident.into());
        Ok(Statement::Def(column, var, var_ident, expr))
    }

    fn r#defdbl(parse: &mut BasicParser) -> Result<Statement> {
        let (from, to) = parse.expect_var_range()?;
        Ok(Statement::Defdbl(parse.col.clone(), from, to))
    }

    fn r#defint(parse: &mut BasicParser) -> Result<Statement> {
        let (from, to) = parse.expect_var_range()?;
        Ok(Statement::Defint(parse.col.clone(), from, to))
    }

    fn r#defsng(parse: &mut BasicParser) -> Result<Statement> {
        let (from, to) = parse.expect_var_range()?;
        Ok(Statement::Defsng(parse.col.clone(), from, to))
    }

    fn r#defstr(parse: &mut BasicParser) -> Result<Statement> {
        let (from, to) = parse.expect_var_range()?;
        Ok(Statement::Defstr(parse.col.clone(), from, to))
    }

    fn r#delete(parse: &mut BasicParser) -> Result<Statement> {
        let column = parse.col.clone();
        let (from, to) = parse.expect_line_number_range()?;
        Ok(Statement::Delete(column, from, to))
    }

    fn r#dim(parse: &mut BasicParser) -> Result<Statement> {
        let column = parse.col.clone();
        let var_list = parse.expect_var_list()?;
        Ok(Statement::Dim(column, var_list))
    }

    fn r#end(parse: &mut BasicParser) -> Result<Statement> {
        Ok(Statement::End(parse.col.clone()))
    }

    fn r#erase(parse: &mut BasicParser) -> Result<Statement> {
        let column = parse.col.clone();
        let mut idents = parse.expect_ident_list()?;
        if idents.is_empty() {
            return Err(error!(SyntaxError, ..&(column.start..column.start); "EXPECTED VARIABLE"));
        }
        let vec_var = idents
            .drain(..)
            .map(|(col, i)| Variable::Unary(col, i.into()))
            .collect::<Vec<Variable>>();
        Ok(Statement::Erase(column, vec_var))
    }

    fn r#for(parse: &mut BasicParser) -> Result<Statement> {
        let column = parse.col.clone();
        let (ident_col, ident) = parse.expect_ident()?;
        let var = Variable::Unary(ident_col, ident.into());
        parse.expect(Token::Operator(Operator::Equal))?;
        let expr_from = parse.expect_expression()?;
        parse.expect(Token::Word(Word::To))?;
        let expr_to = parse.expect_expression()?;
        let expr_step = if parse.maybe(Token::Word(Word::Step)) {
            parse.expect_expression()?
        } else {
            Expression::Integer(parse.col.end..parse.col.end, 1)
        };
        Ok(Statement::For(column, var, expr_from, expr_to, expr_step))
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
        let then_stmt = if parse.maybe(Token::Word(Word::Goto)) {
            vec![Statement::Goto(
                parse.col.clone(),
                parse.expect_line_number()?,
            )]
        } else {
            parse.expect(Token::Word(Word::Then))?;
            match parse.maybe_line_number()? {
                Some(n) => vec![Statement::Goto(
                    column.clone(),
                    Expression::Single(parse.col.clone(), n as f32),
                )],
                None => parse.expect_statements()?,
            }
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
        let var_list = parse.expect_var_list()?;
        Ok(Statement::Input(
            column,
            caps,
            Expression::String(prompt_col, prompt.into()),
            var_list,
        ))
    }

    fn r#let(parse: &mut BasicParser, is_shortcut: bool) -> Result<Statement> {
        let column = parse.col.clone();
        if let Some(Token::Ident(token::Ident::String(s))) = parse.peek() {
            if s == "MID$" {
                parse.next();
                parse.expect(Token::LParen)?;
                let var = parse.expect_var()?;
                parse.expect(Token::Comma)?;
                let pos = parse.expect_expression()?;
                let len = if parse.maybe(Token::Comma) {
                    parse.expect_expression()?
                } else {
                    Expression::Integer(parse.col.start..parse.col.start, i16::max_value())
                };
                parse.expect(Token::RParen)?;
                parse.expect(Token::Operator(Operator::Equal))?;
                let expr = parse.expect_expression()?;
                return Ok(Statement::Mid(column, var, pos, len, expr));
            }
        }
        let var = parse.expect_var()?;
        match parse.next() {
            Some(Token::Operator(Operator::Equal)) => {
                Ok(Statement::Let(column, var, parse.expect_expression()?))
            }
            _ => {
                if is_shortcut {
                    Err(error!(SyntaxError, ..&column; "UNKNOWN STATEMENT"))
                } else {
                    Err(error!(SyntaxError, ..&parse.col; "EXPECTED EQUALS SIGN"))
                }
            }
        }
    }

    fn r#list(parse: &mut BasicParser) -> Result<Statement> {
        let column = parse.col.clone();
        let (from, to) = parse.expect_line_number_range()?;
        Ok(Statement::List(column, from, to))
    }

    fn r#load(parse: &mut BasicParser) -> Result<Statement> {
        Ok(Statement::Load(
            parse.col.clone(),
            parse.expect_expression()?,
        ))
    }

    fn r#new(parse: &mut BasicParser) -> Result<Statement> {
        Ok(Statement::New(parse.col.clone()))
    }

    fn r#next(parse: &mut BasicParser) -> Result<Statement> {
        let column = parse.col.clone();
        let mut idents = parse.expect_ident_list()?;
        if idents.is_empty() {
            return Ok(Statement::Next(
                column,
                vec![Variable::Unary(0..0, Ident::Plain("".into()))],
            ));
        }
        let vec_var = idents
            .drain(..)
            .map(|(col, i)| Variable::Unary(col, i.into()))
            .collect::<Vec<Variable>>();

        Ok(Statement::Next(column, vec_var))
    }

    fn r#on(parse: &mut BasicParser) -> Result<Statement> {
        let column = parse.col.clone();
        let expr = parse.expect_expression()?;
        match parse.next() {
            Some(Token::Word(Word::Goto)) => Ok(Statement::OnGoto(
                column,
                expr,
                parse.expect_line_number_list()?,
            )),
            Some(Token::Word(Word::Gosub)) => Ok(Statement::OnGosub(
                column,
                expr,
                parse.expect_line_number_list()?,
            )),
            _ => Err(error!(SyntaxError, ..&parse.col; "EXPECTED GOTO OR GOSUB")),
        }
    }

    fn r#print(parse: &mut BasicParser) -> Result<Statement> {
        let column = parse.col.clone();
        let vec_expr = parse.expect_print_list()?;
        Ok(Statement::Print(column, vec_expr))
    }

    fn r#read(parse: &mut BasicParser) -> Result<Statement> {
        Ok(Statement::Read(parse.col.clone(), parse.expect_var_list()?))
    }

    fn r#renum(parse: &mut BasicParser) -> Result<Statement> {
        fn parse_start(parse: &mut BasicParser, default: f32) -> Result<Expression> {
            if parse.maybe(Token::Comma) {
                Ok(Expression::Single(parse.col.clone(), default))
            } else {
                match parse.peek() {
                    None | Some(Token::Colon) | Some(Token::Word(Word::Else)) => Ok(
                        Expression::Single(parse.col.start..parse.col.start, default),
                    ),
                    _ => {
                        let ln = parse.expect_line_number()?;
                        parse.maybe(Token::Comma);
                        Ok(ln)
                    }
                }
            }
        }
        let column = parse.col.clone();
        let new_start = parse_start(parse, 10.0)?;
        let old_start = parse_start(parse, 0.0)?;
        let step = match parse.peek() {
            None | Some(Token::Colon) | Some(Token::Word(Word::Else)) => {
                Expression::Single(parse.col.start..parse.col.start, 10.0)
            }
            _ => parse.expect_line_number()?,
        };
        Ok(Statement::Renum(column, new_start, old_start, step))
    }

    fn r#restore(parse: &mut BasicParser) -> Result<Statement> {
        let num = if let Some(num) = parse.maybe_line_number()? {
            num as f32
        } else {
            -1.0
        };
        Ok(Statement::Restore(
            parse.col.clone(),
            Expression::Single(parse.col.clone(), num),
        ))
    }

    fn r#return(parse: &mut BasicParser) -> Result<Statement> {
        Ok(Statement::Return(parse.col.clone()))
    }

    fn r#run(parse: &mut BasicParser) -> Result<Statement> {
        let column = parse.col.clone();
        if let Some(Token::Literal(Literal::String(s))) = parse.peek() {
            parse.next();
            Ok(Statement::Run(
                column,
                Expression::String(parse.col.clone(), s.clone().into()),
            ))
        } else if let Some(num) = parse.maybe_line_number()? {
            Ok(Statement::Run(
                column,
                Expression::Single(parse.col.clone(), num as f32),
            ))
        } else {
            let empty = parse.col.clone();
            let empty = empty.start..empty.start;
            Ok(Statement::Run(column, Expression::Single(empty, -1.0)))
        }
    }

    fn r#save(parse: &mut BasicParser) -> Result<Statement> {
        Ok(Statement::Save(
            parse.col.clone(),
            parse.expect_expression()?,
        ))
    }

    fn r#stop(parse: &mut BasicParser) -> Result<Statement> {
        Ok(Statement::Stop(parse.col.clone()))
    }

    fn r#swap(parse: &mut BasicParser) -> Result<Statement> {
        let col = parse.col.clone();
        let mut var_list = parse.expect_var_list()?;
        if var_list.len() != 2 {
            return Err(
                error!(SyntaxError, ..&(col.start..parse.col.end); "EXPECTED TWO VARIABLES" ),
            );
        }
        Ok(Statement::Swap(
            col,
            var_list.pop().unwrap(),
            var_list.pop().unwrap(),
        ))
    }

    fn r#troff(parse: &mut BasicParser) -> Result<Statement> {
        Ok(Statement::Troff(parse.col.clone()))
    }

    fn r#tron(parse: &mut BasicParser) -> Result<Statement> {
        Ok(Statement::Tron(parse.col.clone()))
    }

    fn r#wend(parse: &mut BasicParser) -> Result<Statement> {
        Ok(Statement::Wend(parse.col.clone()))
    }

    fn r#while(parse: &mut BasicParser) -> Result<Statement> {
        Ok(Statement::While(
            parse.col.clone(),
            parse.expect_expression()?,
        ))
    }
}

impl From<token::Ident> for Ident {
    fn from(ident: token::Ident) -> Self {
        use token::Ident::*;
        match ident {
            Plain(s) => Ident::Plain(s.into()),
            String(s) => Ident::String(s.into()),
            Single(s) => Ident::Single(s.into()),
            Double(s) => Ident::Double(s.into()),
            Integer(s) => Ident::Integer(s.into()),
        }
    }
}

impl From<(&token::Ident, &token::Ident)> for Ident {
    fn from(f: (&token::Ident, &token::Ident)) -> Self {
        let mut string = match f.0 {
            token::Ident::Plain(s) => s,
            token::Ident::String(s) => s,
            token::Ident::Single(s) => s,
            token::Ident::Double(s) => s,
            token::Ident::Integer(s) => s,
        }
        .to_string();
        string.push('.');
        match f.1 {
            token::Ident::Plain(s) => Ident::Plain({
                string.push_str(s);
                string.into()
            }),
            token::Ident::String(s) => Ident::String({
                string.push_str(s);
                string.into()
            }),
            token::Ident::Single(s) => Ident::Single({
                string.push_str(s);
                string.into()
            }),
            token::Ident::Double(s) => Ident::Double({
                string.push_str(s);
                string.into()
            }),
            token::Ident::Integer(s) => Ident::Integer({
                string.push_str(s);
                string.into()
            }),
        }
    }
}

impl token::Ident {
    fn is_user_function(&self) -> bool {
        use token::Ident::*;
        match self {
            Plain(s) => s,
            String(s) => s,
            Single(s) => s,
            Double(s) => s,
            Integer(s) => s,
        }
        .starts_with("FN")
    }
}
