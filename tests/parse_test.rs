use basic::lang::{ast::*, lex, parse};

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
        Variable::Unary(4..7, Ident::Plain("TER".into())),
        Expression::Variable(Variable::Unary(8..11, Ident::Plain("BAR".into()))),
    );
    assert_eq!(parse_str("letter=bar:"), Some(answer));
    let answer = Statement::Let(
        0..3,
        Variable::Unary(0..3, Ident::Plain("TER".into())),
        Expression::Variable(Variable::Unary(4..7, Ident::Plain("BAR".into()))),
    );
    assert_eq!(parse_str("ter=bar:"), Some(answer));
}

#[test]
fn test_literals() {
    let answer = Statement::Let(
        0..1,
        Variable::Unary(0..1, Ident::Plain("A".into())),
        Expression::Integer(2..4, 12),
    );
    assert_eq!(parse_str("A=12"), Some(answer));
    let answer = Statement::Let(
        0..1,
        Variable::Unary(0..1, Ident::Plain("A".into())),
        Expression::Single(2..5, 12.0),
    );
    assert_eq!(parse_str("A=12!"), Some(answer));
    let answer = Statement::Let(
        0..1,
        Variable::Unary(0..1, Ident::Plain("A".into())),
        Expression::Double(2..6, 12e4),
    );
    assert_eq!(parse_str("A=12d4"), Some(answer));
    let answer = Statement::Let(
        0..1,
        Variable::Unary(0..1, Ident::Plain("A".into())),
        Expression::String(2..8, "food".into()),
    );
    assert_eq!(parse_str("A=\"food\""), Some(answer));
    let answer = Statement::Let(
        0..1,
        Variable::Unary(0..1, Ident::Plain("A".into())),
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
        Variable::Unary(0..1, Ident::Plain("A".into())),
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
        Variable::Unary(0..1, Ident::Plain("A".into())),
        Expression::Variable(Variable::Array(
            2..11,
            Ident::Plain("COS".into()),
            vec![Expression::Single(6..10, 3.11)],
        )),
    );
    assert_eq!(parse_str("A=cos(3.11)"), Some(answer));
}

#[test]
fn test_precedence_and_paren() {
    let answer = Statement::Let(
        0..3,
        Variable::Unary(4..5, Ident::Plain("A".into())),
        Expression::Subtract(
            8..9,
            Box::new(Expression::Integer(7..8, 2)),
            Box::new(Expression::Multiply(
                22..23,
                Box::new(Expression::Add(
                    11..12,
                    Box::new(Expression::Integer(10..11, 3)),
                    Box::new(Expression::Variable(Variable::Array(
                        12..21,
                        Ident::Plain("COS".into()),
                        vec![Expression::Single(16..20, 3.11)],
                    ))),
                )),
                Box::new(Expression::Integer(23..24, 4)),
            )),
        ),
    );
    assert_eq!(parse_str("let A=(2-(3+cos(3.11))*4)"), Some(answer));
}

#[test]
fn test_printer_list() {
    let (lin, tokens) = lex("? 1 2,3;:?0.0");
    assert_eq!(
        parse(lin, &tokens).ok(),
        Some(vec!(
            Statement::Print(
                0..5,
                vec![
                    Expression::Integer(6..7, 1),
                    Expression::Integer(8..9, 2),
                    Expression::Variable(Variable::Array(
                        9..10,
                        Ident::String("TAB".into()),
                        vec![Expression::Integer(9..10, -14)]
                    )),
                    Expression::Integer(10..11, 3),
                ]
            ),
            Statement::Print(
                13..18,
                vec![
                    Expression::Single(19..22, 0.0),
                    Expression::String(22..22, "\n".into())
                ]
            )
        ))
    );
}

#[test]
fn test_remarks() {
    let (lin, tokens) = lex("10 PRINT REMARK");
    assert_eq!(
        parse(lin, &tokens).ok(),
        Some(vec!(Statement::Print(
            0..5,
            vec![Expression::String(6..6, "\n".into())]
        )))
    );
    let (lin, tokens) = lex("10 PRINT 'REMARK");
    assert_eq!(
        parse(lin, &tokens).ok(),
        Some(vec!(Statement::Print(
            0..5,
            vec![Expression::String(6..6, "\n".into())]
        )))
    );
}
