/*!
# `ERASE [<array variable>][,<array variable>...]`

## Purpose
Erases a dimensioned array.

## Remarks
Arrays may be redimensioned after they are erased.

## Example
```text
10 DIM A$(10,10)
20 ERASE A$
30 A$(5) = "Five"
```

*/
