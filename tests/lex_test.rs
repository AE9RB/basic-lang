use basic::lang::{lex, token::*, Line};

fn token(s: &str) -> Option<Token> {
    let s = format!("?{}", s);
    let (_, mut tokens) = lex(&s);
    let mut t = tokens.drain(2..3);
    t.next()
}

#[test]
fn test_eq_gt() {
    let (ln, v) = lex("10 1=<>=<>2");
    assert_eq!(ln, Some(10));
    let mut x = v.iter();
    assert_eq!(
        x.next(),
        Some(&Token::Literal(Literal::Integer("1".to_string())))
    );
    assert_eq!(x.next(), Some(&Token::Operator(Operator::LessEqual)));
    assert_eq!(x.next(), Some(&Token::Operator(Operator::GreaterEqual)));
    assert_eq!(x.next(), Some(&Token::Operator(Operator::NotEqual)));
    assert_eq!(
        x.next(),
        Some(&Token::Literal(Literal::Integer("2".to_string())))
    );
    assert_eq!(x.next(), None);
}

#[test]
fn test_go_to_1() {
    let (ln, v) = lex("10 go to");
    assert_eq!(ln, Some(10));
    let mut x = v.iter();
    assert_eq!(x.next(), Some(&Token::Word(Word::Goto)));
    assert_eq!(x.next(), None);
}

#[test]
fn test_go_to_2() {
    assert_eq!(token("GO TO"), Some(Token::Word(Word::Goto)));
}

#[test]
fn test_go_sub_2() {
    assert_eq!(token("GO SUB"), Some(Token::Word(Word::Gosub)));
}

#[test]
fn test_print_1() {
    let (ln, v) = lex("10 ?");
    assert_eq!(ln, Some(10));
    let mut x = v.iter();
    assert_eq!(x.next(), Some(&Token::Word(Word::Print)));
    assert_eq!(x.next(), None);
}

#[test]
fn test_print_2() {
    assert_eq!(token("?"), Some(Token::Word(Word::Print)));
}

#[test]
fn test_numbers() {
    assert_eq!(
        token("3.141593"),
        Some(Token::Literal(Literal::Single("3.141593".to_string())))
    );
    assert_eq!(
        token("3.1415926"),
        Some(Token::Literal(Literal::Double("3.1415926".to_string())))
    );
    assert_eq!(
        token("32767"),
        Some(Token::Literal(Literal::Integer("32767".to_string())))
    );
    assert_eq!(
        token("32768"),
        Some(Token::Literal(Literal::Single("32768".to_string())))
    );
    assert_eq!(
        token("24e9"),
        Some(Token::Literal(Literal::Single("24E9".to_string())))
    );
}

#[test]
fn test_annotated_numbers() {
    assert_eq!(
        token("12334567890!"),
        Some(Token::Literal(Literal::Single("12334567890!".to_string())))
    );
    assert_eq!(
        token("0#"),
        Some(Token::Literal(Literal::Double("0#".to_string())))
    );
    assert_eq!(
        token("24e9%"),
        Some(Token::Literal(Literal::Integer("24E9%".to_string())))
    );
}

#[test]
fn test_remark1() {
    let (ln, v) = lex("100 REM  A fortunate comment");
    assert_eq!(ln, Some(100));
    let mut x = v.iter();
    assert_eq!(x.next(), Some(&Token::Word(Word::Rem1)));
    assert_eq!(
        x.next(),
        Some(&Token::Unknown("  A fortunate comment".to_string()))
    );
    assert_eq!(x.next(), None);
}

#[test]
fn test_remark2() {
    let (ln, v) = lex("100  'The comment  ");
    assert_eq!(ln, Some(100));
    let mut x = v.iter();
    assert_eq!(x.next(), Some(&Token::Whitespace(1)));
    assert_eq!(x.next(), Some(&Token::Word(Word::Rem2)));
    assert_eq!(x.next(), Some(&Token::Unknown("The comment".to_string())));
    assert_eq!(x.next(), None);
}

#[test]
fn test_ident_with_word() {
    let (ln, v) = lex("BANDS");
    assert_eq!(ln, None);
    let mut x = v.iter();
    assert_eq!(x.next(), Some(&Token::Ident(Ident::Plain("B".into()))));
    assert_eq!(x.next(), Some(&Token::Whitespace(1)));
    assert_eq!(x.next(), Some(&Token::Operator(Operator::And)));
    assert_eq!(x.next(), Some(&Token::Whitespace(1)));
    assert_eq!(x.next(), Some(&Token::Ident(Ident::Plain("S".into()))));
    assert_eq!(x.next(), None);
}

#[test]
fn test_for_loop() {
    let (ln, v) = lex("forI%=1to30");
    assert_eq!(ln, None);
    let mut x = v.iter();
    assert_eq!(x.next(), Some(&Token::Word(Word::For)));
    assert_eq!(x.next(), Some(&Token::Whitespace(1)));
    assert_eq!(
        x.next(),
        Some(&Token::Ident(Ident::Integer("I%".to_string())))
    );
    assert_eq!(x.next(), Some(&Token::Operator(Operator::Equal)));
    assert_eq!(
        x.next(),
        Some(&Token::Literal(Literal::Integer("1".to_string())))
    );
    assert_eq!(x.next(), Some(&Token::Whitespace(1)));
    assert_eq!(x.next(), Some(&Token::Word(Word::To)));
    assert_eq!(x.next(), Some(&Token::Whitespace(1)));
    assert_eq!(
        x.next(),
        Some(&Token::Literal(Literal::Integer("30".to_string())))
    );
    assert_eq!(x.next(), None);
}

#[test]
fn test_trim_start() {
    let (ln, v) = lex(" 10 PRINT 10");
    assert_eq!(ln, Some(10));
    let mut x = v.iter();
    assert_eq!(x.next(), Some(&Token::Word(Word::Print)));
    assert_eq!(x.next(), Some(&Token::Whitespace(1)));
}

#[test]
fn test_do_not_trim_start() {
    let (ln, v) = lex("  PRINT 10");
    assert_eq!(ln, None);
    let mut x = v.iter();
    assert_eq!(x.next(), Some(&Token::Whitespace(2)));
    assert_eq!(x.next(), Some(&Token::Word(Word::Print)));
    assert_eq!(x.next(), Some(&Token::Whitespace(1)));
}

#[test]
fn test_empty() {
    let (ln, v) = lex("");
    assert_eq!(ln, None);
    let mut x = v.iter();
    assert_eq!(x.next(), None);
}

#[test]
fn test_line_number_only() {
    let (ln, v) = lex("10");
    assert_eq!(ln, Some(10));
    let mut x = v.iter();
    assert_eq!(x.next(), None);
}

#[test]
fn test_string_at_start() {
    let (ln, v) = lex("\"HELLO\"");
    assert_eq!(ln, None);
    let mut x = v.iter();
    assert_eq!(
        x.next(),
        Some(&Token::Literal(Literal::String("HELLO".to_string())))
    );
}

#[test]
fn test_unknown() {
    let (ln, v) = lex("10 for %w");
    assert_eq!(ln, Some(10));
    let mut x = v.iter();
    assert_eq!(x.next(), Some(&Token::Word(Word::For)));
    assert_eq!(x.next(), Some(&Token::Whitespace(1)));
    assert_eq!(x.next(), Some(&Token::Unknown("%".to_string())));
    assert_eq!(x.next(), Some(&Token::Ident(Ident::Plain("W".to_string()))));
    assert_eq!(x.next(), None);
}

#[test]
fn test_insert_spacing() {
    let (ln, v) = lex("10 printJ:printK");
    assert_eq!(ln, Some(10));
    let mut x = v.iter();
    assert_eq!(x.next(), Some(&Token::Word(Word::Print)));
    assert_eq!(x.next(), Some(&Token::Whitespace(1)));
    assert_eq!(x.next(), Some(&Token::Ident(Ident::Plain("J".to_string()))));
    assert_eq!(x.next(), Some(&Token::Colon));
    assert_eq!(x.next(), Some(&Token::Word(Word::Print)));
    assert_eq!(x.next(), Some(&Token::Whitespace(1)));
    assert_eq!(x.next(), Some(&Token::Ident(Ident::Plain("K".to_string()))));
    assert_eq!(x.next(), None);
}

#[test]
fn test_direct() {
    let l = Line::new("run");
    assert_eq!(&l.to_string(), "RUN");
}

#[test]
fn test_indirect() {
    let l = Line::new("100 end");
    assert_eq!(&l.to_string(), "100 END");
    assert_eq!(l.number(), Some(100));
}
