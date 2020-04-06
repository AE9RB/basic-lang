mod common;
use basic::mach::Runtime;
use common::*;

#[test]
fn test_built_in_reserved() {
    let mut r = Runtime::default();
    r.enter(r#"len$(0)="foo":?len$(0)"#);
    assert_eq!(exec(&mut r), "foo\n");
    r.enter(r#"rnd=42:?rnd"#);
    assert_eq!(exec(&mut r), " 42 \n");
    r.enter(r#"val(0)=42"#);
    assert_eq!(exec(&mut r), "?SYNTAX ERROR; RESERVED FOR BUILT-IN\n");
    r.enter(r#"time$="42""#);
    assert_eq!(exec(&mut r), "?SYNTAX ERROR; RESERVED FOR BUILT-IN\n");
    r.enter(r#"rnd()=42"#);
    assert_eq!(exec(&mut r), "?SYNTAX ERROR; EXPECTED EXPRESSION\n");
}

#[test]
fn test_fn_abs() {
    let mut r = Runtime::default();
    r.enter(r#"?abs(9)abs(-9)"#);
    assert_eq!(exec(&mut r), " 9  9 \n");
}

#[test]
fn test_fn_asc() {
    let mut r = Runtime::default();
    r.enter(r#"?asc("A")"#);
    assert_eq!(exec(&mut r), " 65 \n");
}

#[test]
fn test_fn_atn() {
    let mut r = Runtime::default();
    r.enter(r#"?atn(3)"#);
    assert_eq!(exec(&mut r), " 1.2490457 \n");
}

#[test]
fn test_fn_cdbl() {
    let mut r = Runtime::default();
    r.enter(r#"?cdbl(3.1)"#);
    assert_eq!(exec(&mut r), " 3.0999999046325684 \n");
}

#[test]
fn test_fn_chr() {
    let mut r = Runtime::default();
    r.enter(r#"?chr$(65.9)"#);
    assert_eq!(exec(&mut r), "A\n");
}

#[test]
fn test_fn_cint() {
    let mut r = Runtime::default();
    r.enter(r#"?cint(-3.7)"#);
    assert_eq!(exec(&mut r), "-4 \n");
}

#[test]
fn test_fn_cos() {
    let mut r = Runtime::default();
    r.enter(r#"?cos(3.14159265)"#);
    assert_eq!(exec(&mut r), "-1 \n");
}

#[test]
fn test_fn_csng() {
    let mut r = Runtime::default();
    r.enter(r#"?csng(-3.123456789)"#);
    assert_eq!(exec(&mut r), "-3.1234567 \n");
}

#[test]
fn test_fn_date() {
    let mut r = Runtime::default();
    r.enter(r#"?len(date$)"#);
    assert_eq!(exec(&mut r), " 10 \n");
}

#[test]
fn test_fn_exp() {
    let mut r = Runtime::default();
    r.enter(r#"?exp(-9.9)"#);
    assert_eq!(exec(&mut r), " 5.01747E-5 \n");
}

#[test]
fn test_fn_fix() {
    let mut r = Runtime::default();
    r.enter(r#"?fix(-9.9)"#);
    assert_eq!(exec(&mut r), "-9 \n");
}

#[test]
fn test_fn_hex() {
    let mut r = Runtime::default();
    r.enter(r#"?hex$(13)"#);
    assert_eq!(exec(&mut r), "D\n");
}

#[test]
fn test_fn_instr() {
    let mut r = Runtime::default();
    r.enter(r#"?instr("abcdeb","b")"#);
    assert_eq!(exec(&mut r), " 2 \n");
    r.enter(r#"?instr(6, "abcdeb","b")"#);
    assert_eq!(exec(&mut r), " 6 \n");
    r.enter(r#"?instr(0, "abcdeb","b")"#);
    assert_eq!(exec(&mut r), "?ILLEGAL FUNCTION CALL; START IS 0\n");
    r.enter(r#"?instr(4, "abcdeb","")"#);
    assert_eq!(exec(&mut r), " 4 \n");
    r.enter(r#"?instr(9, "abcdeb","a")"#);
    assert_eq!(exec(&mut r), " 0 \n");
    r.enter(r#"?instr(2,"","a")"#);
    assert_eq!(exec(&mut r), " 0 \n");
    r.enter(r#"?instr("","a")"#);
    assert_eq!(exec(&mut r), " 0 \n");
}

#[test]
fn test_fn_int() {
    let mut r = Runtime::default();
    r.enter(r#"?int(9.9)int(-9.9)"#);
    assert_eq!(exec(&mut r), " 9 -10 \n");
}

#[test]
fn test_fn_left() {
    let mut r = Runtime::default();
    r.enter(r#"?left$("TASTY",2)"#);
    assert_eq!(exec(&mut r), "TA\n");
}

#[test]
fn test_fn_len() {
    let mut r = Runtime::default();
    r.enter(r#"?len("TASTY")"#);
    assert_eq!(exec(&mut r), " 5 \n");
}

#[test]
fn test_fn_log() {
    let mut r = Runtime::default();
    r.enter(r#"?log(8/37)"#);
    assert_eq!(exec(&mut r), "-1.5314764 \n");
}

#[test]
fn test_fn_mid() {
    let mut r = Runtime::default();
    r.enter(r#"?mid$("TASTY",4)"#);
    assert_eq!(exec(&mut r), "TY\n");
    r.enter(r#"?mid$("TASTY",4,1)"#);
    assert_eq!(exec(&mut r), "T\n");
}

#[test]
fn test_fn_oct() {
    let mut r = Runtime::default();
    r.enter(r#"?oct$(13)"#);
    assert_eq!(exec(&mut r), "15\n");
}

#[test]
fn test_fn_pos() {
    let mut r = Runtime::default();
    r.enter(r#"?"     ";pos()"#);
    assert_eq!(exec(&mut r), "      5 \n");
    r.enter(r#"?"      ";pos(-10)"#);
    assert_eq!(exec(&mut r), "       6 \n");
}

#[test]
fn test_fn_right() {
    let mut r = Runtime::default();
    r.enter(r#"?right$("TASTY",3)"#);
    assert_eq!(exec(&mut r), "STY\n");
}

#[test]
fn test_fn_rnd() {
    let mut r = Runtime::default();
    r.enter(r#"?rnd()rnd()rnd(0)rnd(1)"#);
    assert_eq!(
        exec(&mut r),
        " 1.6930906E-2  0.89525414  0.89525414  1.11491084E-1 \n"
    );
    r.enter(r#"?rnd(-1.61803)rnd(0)rnd()"#);
    assert_eq!(exec(&mut r), " 0.2008394  0.2008394  1.7587423E-2 \n");
}

#[test]
fn test_fn_sgn() {
    let mut r = Runtime::default();
    r.enter(r#"?sgn(0.0);sgn(-1.0/0.0);sgn(10000000000)"#);
    assert_eq!(exec(&mut r), " 0 -1  1 \n");
}

#[test]
fn test_fn_sin() {
    let mut r = Runtime::default();
    r.enter(r#"?sin(0.707)"#);
    assert_eq!(exec(&mut r), " 0.64955574 \n");
}

#[test]
fn test_fn_sqr() {
    let mut r = Runtime::default();
    r.enter(r#"?sqr(5)"#);
    assert_eq!(exec(&mut r), " 2.236068 \n");
}

#[test]
fn test_fn_str() {
    let mut r = Runtime::default();
    r.enter(r#"?str$(5)"#);
    assert_eq!(exec(&mut r), " 5\n");
}

#[test]
fn test_fn_tab() {
    let mut r = Runtime::default();
    r.enter(r#"?tab(5)"!""#);
    assert_eq!(exec(&mut r), "     !\n");
}

#[test]
fn test_fn_tan() {
    let mut r = Runtime::default();
    r.enter(r#"?tan(5/13)"#);
    assert_eq!(exec(&mut r), " 0.40477434 \n");
}

#[test]
fn test_fn_val() {
    let mut r = Runtime::default();
    r.enter(r#"?val(123)"#);
    assert_eq!(exec(&mut r), "?TYPE MISMATCH\n");
    r.enter(r#"?val("123")"#);
    assert_eq!(exec(&mut r), " 123 \n");
    r.enter(r#"?val("one")"#);
    assert_eq!(exec(&mut r), " 0 \n");
    r.enter(r#"?val("    42  ")"#);
    assert_eq!(exec(&mut r), " 42 \n");
}
