/*!
# `NEXT [<variable>][,<variable>...]`
Also see `FOR`

## Purpose
Used to indicate the end of a `FOR` loop.

## Remarks
`FOR` loops are stack based. Specifying an optional variable
here will enforce that the stack frame being used matches.
Some confusion (or abuse) can happen if using `GOTO` to break a loop.

## Example
```text
FOR I=1 to 10:NEXT
FOR J=1 to 20:FOR I=1 to 20:NEXT I,J
```

*/
