use super::*;

#[test]
fn test_breaking_out_of_loop_with_goto() {
    let mut r = Runtime::default();
    r.enter("10fory=1to2");
    r.enter("20forx=8to9");
    r.enter("30?y;x");
    r.enter("40goto60");
    r.enter("50next");
    r.enter("60nexty");
    r.enter("60nexty");
    r.enter("run");
    assert_eq!(run(&mut r), " 1  8 \n 2  8 \n");
}
