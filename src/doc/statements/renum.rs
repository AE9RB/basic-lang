/*!
# `RENUM [<new number>][,<old number>][,<increment>]`

## Purpose
Renumber a program.

## Remarks
New number defaults to 10. If old number is specified, lines less than
that number are not renumbered. Increment defaults to 10.
All statements containing line numbers are updated.
You can not change the order of lines.
Failures do not modify the program in memory.

## Example
```text
RENUM 1000
RENUM 100,,100
```

*/
