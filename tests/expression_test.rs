mod common;
use basic::mach::Runtime;
use common::*;

#[test]
fn test_precedence() {
    let mut r = Runtime::default();
    r.enter(r#"?1+2*3"#);
    assert_eq!(exec(&mut r), " 7 \n");
    r.enter(r#"?(1+2)*3"#);
    assert_eq!(exec(&mut r), " 9 \n");
}

#[test]
fn test_left_assoc() {
    let mut r = Runtime::default();
    r.enter(r#"?1.5/2*3"#);
    assert_eq!(exec(&mut r), " 2.25 \n");
    r.enter(r#"?1.5/(2.0*3)"#);
    assert_eq!(exec(&mut r), " 0.25 \n");
}

#[test]
fn test_int_division() {
    let mut r = Runtime::default();
    r.enter(r#"?10/4"#);
    assert_eq!(exec(&mut r), " 2.5 \n");
}

#[test]
fn test_variables() {
    let mut r = Runtime::default();
    r.enter(r#"a=1+2*3:?a*2"#);
    assert_eq!(exec(&mut r), " 14 \n");
    r.enter(r#"a%=300*300"#);
    assert_eq!(exec(&mut r), "?OVERFLOW\n");
}

#[test]
fn test_array_basics() {
    let mut r = Runtime::default();
    r.enter(r#"10 DIM A$(100), X(10,10)"#);
    r.enter(r#"20 A$(42)="THE ANSWER""#);
    r.enter(r#"30 X(4,2)=2.7182818"#);
    r.enter(r#"40 PRINT A$(42)+"!", X(4,2)"#);
    r.enter(r#"run"#);
    assert_eq!(exec(&mut r), "THE ANSWER!    2.7182817 \n");
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
fn test_fn_int() {
    let mut r = Runtime::default();
    r.enter(r#"?int(9.9)int(-9.9)"#);
    assert_eq!(exec(&mut r), " 9 -10 \n");
}
