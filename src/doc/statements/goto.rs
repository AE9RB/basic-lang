/*!
# `GOTO <line number>`

## Purpose
Immediately and unconditionally move execution to the specified line number.

## Remarks
If `<line number>` doesn't exist an `UNDEFINED LINE` error will occur.

## Example
```text
10 GOTO 30
20 PRINT "THIS WILL NOT PRINT"
30 PRINT "THIS WILL PRINT"
```

*/
