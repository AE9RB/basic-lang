/*!
# `[LET] MID$(<string>,n[,m])=<expr> `

## Purpose
Assign the result of expression to variable.

## Remarks
The word `LET` is optional. Replaces part of the `string`
beginning at position `n` with `expr`. The `string` will not
change size so extra chars from the `expr` are ignored.
The length of `expr` used may be limited by the optional `m`.

## Example
```text
A$(5)="PORTLAND, ME"
LET MID$(A$(5),11)="OR"
PRINT A$(5)
PORTLAND, OR
```

*/
