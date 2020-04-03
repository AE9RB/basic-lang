/*!
# `WHILE <expression> : WEND`

## Purpose
Loop until the expression evaluates false.

## Remarks
`WHILE` and `WEND` are matched up in the link phase according to their position
in the source. This is different from `FOR` loops which use the stack.

## Example
```text
10 READ A$
20 WHILE A$ <> "END"
30 PRINT A$;
40 READ A$
50 WEND
60 DATA "S","T","A","S","I","S","END"
RUN
STASIS
```

*/
