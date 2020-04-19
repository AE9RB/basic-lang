/*!
# `TRON | TROFF`

## Purpose
Enable and disable trace printing.

## Remarks
When tracing is on, the executing line number will be printed
as the program is running.

## Example
```text
10 FOR I = 1 TO 3
20 TROFF:IF I = 2 THEN TRON
30 PRINT I
40 NEXT I
RUN
 1
[30] 2
[40][20] 3
```

*/
