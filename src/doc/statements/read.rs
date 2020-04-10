/*!
# `READ <variable>[,<variable>]`

## Purpose
Reads the information defined in `DATA` statements.

## Remarks
An `?OUT OF DATA` error will occur when reading past the end.

## Example
```text
10 READ A$,A%
20 PRINT A$;A%
30 DATA "NUGGET",3
RUN
NUGGET 3
```

*/
