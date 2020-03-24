mod common;
use basic::mach::Runtime;
use common::*;

#[test]
fn test_breaking_out_of_loop_with_goto() {
    let mut r = Runtime::default();
    r.enter(r#"10fory=1to2"#);
    r.enter(r#"20forx=8to9"#);
    r.enter(r#"30?y;x"#);
    r.enter(r#"40goto60"#);
    r.enter(r#"50next"#);
    r.enter(r#"60nexty"#);
    r.enter(r#"run"#);
    assert_eq!(exec(&mut r), " 1  8 \n 2  8 \n");
}

#[test]
fn test_input_to_array() {
    let mut r = Runtime::default();
    r.enter(r#"input a%,b(a%):print a%;: print b(2-a%);"#);
    assert_eq!(exec(&mut r), "? ");
    r.enter(r#"1,2"#);
    assert_eq!(exec(&mut r), " 1  2 \n");
}

#[test]
fn test_if_then() {
    let mut r = Runtime::default();
    r.enter(r#"if 1 then ? "one""#);
    assert_eq!(exec(&mut r), "one\n");
}

#[test]
fn test_if_then_else() {
    let mut r = Runtime::default();

    r.enter(r#"if 0 then ? "one" else ? "two";:?2"#);
    assert_eq!(exec(&mut r), "two 2 \n");
    r.enter(r#"if 1 then ? "one" else ? "two":?2"#);
    assert_eq!(exec(&mut r), "one\n");
}
