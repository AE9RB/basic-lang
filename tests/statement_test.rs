mod common;
use basic::mach::Runtime;
use common::*;

#[test]
fn test_breaking_out_of_for_loop_with_goto() {
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
fn test_for_loop_always_runs_once() {
    let mut r = Runtime::default();
    r.enter(r#"FOR I=3 TO 0:PRINT I:NEXT I"#);
    assert_eq!(exec(&mut r), " 3 \n");
}

#[test]
fn test_for_loop_assign_step_after_var() {
    let mut r = Runtime::default();
    r.enter(r#"I=1:FOR I=3 TO 9 STEP I:PRINT I;:NEXT"#);
    assert_eq!(exec(&mut r), " 3  6  9 \n");
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
    r.enter(r#"if 1 then ? "one";:?2"#);
    assert_eq!(exec(&mut r), "one 2 \n");
    r.enter(r#"if 0 then ? "one";:?2"#);
    assert_eq!(exec(&mut r), "");
}

#[test]
fn test_gosub_return() {
    let mut r = Runtime::default();
    r.enter(r#"10 GOSUB 100"#);
    r.enter(r#"20 PRINT "WORLD""#);
    r.enter(r#"90 END"#);
    r.enter(r#"100 PRINT "HELLO ";"#);
    r.enter(r#"110 RETURN"#);
    r.enter(r#"RUN"#);
    assert_eq!(exec(&mut r), "HELLO WORLD\n");
}

#[test]
fn test_new() {
    let mut r = Runtime::default();
    r.enter(r#"10 A=1"#);
    r.enter(r#"20 NEW"#);
    r.enter(r#"RUN:PRINT 9"#);
    assert_eq!(exec(&mut r), "");
    r.enter(r#"PRINT A"#);
    assert_eq!(exec(&mut r), " 0 \n");
    r.enter(r#"LIST"#);
    assert_eq!(exec(&mut r), "");
}

#[test]
fn test_end_cont() {
    let mut r = Runtime::default();
    r.enter(r#"10 A=1"#);
    r.enter(r#"20 END"#);
    r.enter(r#"30 PRINT A"#);
    r.enter(r#"RUN"#);
    assert_eq!(exec(&mut r), "");
    r.enter(r#"CONT"#);
    assert_eq!(exec(&mut r), " 1 \n");
}
#[test]
fn test_stop_cont() {
    let mut r = Runtime::default();
    r.enter(r#"10 A=1"#);
    r.enter(r#"20 STOP"#);
    r.enter(r#"30 PRINT A"#);
    r.enter(r#"RUN"#);
    assert_eq!(exec(&mut r), "?BREAK IN 20\n");
    r.enter(r#"CONT"#);
    assert_eq!(exec(&mut r), " 1 \n");
}

#[test]
fn test_on_gosub() {
    let mut r = Runtime::default();
    r.enter(r#"10 X=2"#);
    r.enter(r#"20 ON X GOSUB 100,200"#);
    r.enter(r#"30 PRINT 30:END"#);
    r.enter(r#"100 PRINT 100;"#);
    r.enter(r#"200 PRINT 200;"#);
    r.enter(r#"300 RETURN"#);
    r.enter(r#"RUN"#);
    assert_eq!(exec(&mut r), " 200  30 \n");
}

#[test]
fn test_on_gosub_neg() {
    let mut r = Runtime::default();
    r.enter(r#"10 X=-1"#);
    r.enter(r#"20 ON X GOSUB 100,200"#);
    r.enter(r#"30 PRINT 30:END"#);
    r.enter(r#"100 PRINT 100;"#);
    r.enter(r#"200 PRINT 200;"#);
    r.enter(r#"300 RETURN"#);
    r.enter(r#"RUN"#);
    assert_eq!(exec(&mut r), "?ILLEGAL FUNCTION CALL IN 20\n");
}

#[test]
fn test_on_gosub_invalid() {
    let mut r = Runtime::default();
    r.enter(r#"10 X=3"#);
    r.enter(r#"20 ON X GOSUB 100,200"#);
    r.enter(r#"30 PRINT 30:END"#);
    r.enter(r#"100 PRINT 100;"#);
    r.enter(r#"200 PRINT 200;"#);
    r.enter(r#"300 RETURN"#);
    r.enter(r#"RUN"#);
    assert_eq!(exec(&mut r), " 30 \n");
}

#[test]
fn test_def_fn() {
    let mut r = Runtime::default();
    r.enter(r#"10 DEF FN(X)=X*2"#);
    r.enter(r#"20 DEF FNA(X,Y)=FN(X)/Y"#);
    r.enter(r#"30 PRINT FNA(1,3)"#);
    r.enter(r#"RUN"#);
    assert_eq!(exec(&mut r), " 0.6666667 \n");
}

#[test]
fn test_indirect_error() {
    let mut r = Runtime::default();
    r.enter(r#"10 GOTO 100"#);
    r.enter(r#"RUN"#);
    assert_eq!(exec(&mut r), "?UNDEFINED LINE IN 10:9\n");
}

#[test]
fn test_read_data() {
    let mut r = Runtime::default();
    r.enter(r#"10 READ A, A$"#);
    r.enter(r#"20 PRINT A; A$"#);
    r.enter(r#"30 DATA 99, "Red Balloons"#);
    r.enter(r#"RUN"#);
    assert_eq!(exec(&mut r), " 99 Red Balloons\n");
}

#[test]
fn test_restore_data() {
    let mut r = Runtime::default();
    r.enter(r#"10 DATA 10"#);
    r.enter(r#"20 DATA 20"#);
    r.enter(r#"30 DATA -30"#);
    r.enter(r#"READ A,B,C:PRINT A;B;C"#);
    assert_eq!(exec(&mut r), " 10  20 -30 \n");
    r.enter(r#"RESTORE:READ A,B,C:PRINT A;B;C"#);
    assert_eq!(exec(&mut r), " 10  20 -30 \n");
    r.enter(r#"RESTORE 30:READ A:PRINT A"#);
    assert_eq!(exec(&mut r), "-30 \n");
}

#[test]
fn test_while_wend_nested() {
    let mut r = Runtime::default();
    r.enter(r#"10 WHILE I<2:I=I+1:PRINT I;"#);
    r.enter(r#"RUN"#);
    assert_eq!(exec(&mut r), "?WHILE WITHOUT WEND IN 10:4\n");
    r.enter(r#"30 WEND"#);
    r.enter(r#"RUN"#);
    assert_eq!(exec(&mut r), " 1  2 \n");
    r.enter(r#"40 WEND"#);
    r.enter(r#"RUN"#);
    assert_eq!(exec(&mut r), "?WEND WITHOUT WHILE IN 40:4\n");
    r.enter(r#"20 WHILE J<2:J=J+1:PRINT J+10;"#);
    r.enter(r#"RUN"#);
    assert_eq!(exec(&mut r), " 1  11  12  2 \n");
    r.enter(r#"35 J=0"#);
    r.enter(r#"RUN"#);
    assert_eq!(exec(&mut r), " 1  11  12  2  11  12 \n");
}

#[test]
fn test_while_wend_not_nested() {
    let mut r = Runtime::default();
    r.enter(r#"10 WHILE A<2:A=A+1:PRINT A;:WEND"#);
    r.enter(r#"20 WHILE B<2:B=B+1:PRINT B;:WEND"#);
    r.enter(r#"RUN"#);
    assert_eq!(exec(&mut r), " 1  2  1  2 \n");
}

#[test]
fn test_deftype() {
    let mut r = Runtime::default();
    r.enter(r#"s$="ess":?s$"#);
    assert_eq!(exec(&mut r), "ess\n");
    r.enter(r#"DEFSTR s:s="foo":?s"#);
    assert_eq!(exec(&mut r), "foo\n");
    r.enter(r#"?s$"#);
    assert_eq!(exec(&mut r), "ess\n");
    r.enter(r#"DEFSTR t:?t"#);
    assert_eq!(exec(&mut r), "\n");
    r.enter(r#"DEFINT i-"#);
    assert_eq!(exec(&mut r), "?SYNTAX ERROR; EXPECTED VARIABLE\n");
    r.enter(r#"DEFINT i-j:i=3.14:?i"#);
    assert_eq!(exec(&mut r), " 3 \n");
    r.enter(r#"DEFINT ii"#);
    assert_eq!(exec(&mut r), "?SYNTAX ERROR\n");
    r.enter(r#"a=1.1:DEFINT a-a:?a"#);
    assert_eq!(exec(&mut r), " 0 \n");
    r.enter(r#"a=1.1:DEFINT a-a:?a"#);
    assert_eq!(exec(&mut r), " 1 \n");
}

#[test]
fn test_erase() {
    let mut r = Runtime::default();
    r.enter(r#"DIM A$(10,10):A$(5,5)="FIVE":PRINT A$(5,5)"#);
    assert_eq!(exec(&mut r), "FIVE\n");
    r.enter(r#"ERASE A$:PRINT A$(5,5)"#);
    assert_eq!(exec(&mut r), "\n");
    r.enter(r#"DIM A$(20):PRINT A$(20)"#);
    assert_eq!(exec(&mut r), "?REDIMENSIONED ARRAY\n");
    r.enter(r#"ERASE A$:DIM A$(20):PRINT A$(20)"#);
    assert_eq!(exec(&mut r), "\n");
}

#[test]
fn test_swap() {
    let mut r = Runtime::default();
    r.enter(r#"SWAP A,B!:PRINT A"#);
    assert_eq!(exec(&mut r), " 0 \n");
    r.enter(r#"A=1:B=2:SWAPA,B:PRINTA;B"#);
    assert_eq!(exec(&mut r), " 2  1 \n");
    r.enter(r#"DEFSTR S:S="S":A$="A":SWAP S,A$:PRINTA$;S"#);
    assert_eq!(exec(&mut r), "SA\n");
    r.enter(r#"A%=127:SWAP A%,B#"#);
    assert_eq!(exec(&mut r), "?TYPE MISMATCH\n");
    r.enter(r#"PRINT A%"#);
    assert_eq!(exec(&mut r), " 127 \n");
}

#[test]
fn test_let_mid_statement() {
    let mut r = Runtime::default();
    r.enter(r#"A$="PORTLAND, ME":MID$(A$,11)="OR":?A$"#);
    assert_eq!(exec(&mut r), "PORTLAND, OR\n");
    r.enter(r#"LET MID$(A$,11)="MEH":?A$"#);
    assert_eq!(exec(&mut r), "PORTLAND, ME\n");
    r.enter(r#"MID$(A$,0)="Portland":?A$"#);
    assert_eq!(exec(&mut r), "?ILLEGAL FUNCTION CALL; POSITION IS ZERO\n");
    r.enter(r#"MID$(A$,1)="Portland":?A$"#);
    assert_eq!(exec(&mut r), "Portland, ME\n");
    r.enter(r#"MID$(A$,5,1)="LALA":?A$"#);
    assert_eq!(exec(&mut r), "PortLand, ME\n");
    r.enter(r#"MID$(A$,1,0)="LALA":?A$"#);
    assert_eq!(exec(&mut r), "PortLand, ME\n");
    r.enter(r#"A$(5)="PORTLAND, ME":MID$(A$(5),11)="OR":?A$(5)"#);
    assert_eq!(exec(&mut r), "PORTLAND, OR\n");
}
