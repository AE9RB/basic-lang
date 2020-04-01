/*!
# `RESTORE [<line number>]`

## Purpose
Changes the `DATA` pointer to a different location.

## Remarks
Not specifying a line number restores the pointer to the first element
of the first `DATA` statement. You can also move the pointer to the
first element of any line.

## Example
```text
10 FOR I=1 TO 5
20 READ A$: PRINT A$;:RESTORE 110
30 NEXT
100 DATA "HELLO"
110 DATA "."
RUN
HELLO....
```

*/
