/*!
# `IF <expression> THEN <statements> [ELSE <statements>]`
Also `IF <expression> GOTO <line>`.

## Purpose
Do something contingent on a predicate.

## Remarks
Statements may omit the word GOTO.

## Example
```text
10 A=10
20 IF A<30 THEN PRINT A:GOSUB 100:GOTO 20
90 END
100 A=A+10:RETURN
RUN
10
20
```

*/
