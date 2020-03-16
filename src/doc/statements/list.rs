/*!
# `LIST [<from line number>][-<to line number>]`

## Purpose
Show the BASIC program currently in memory.

## Remarks
You can optionally specify a line or range.

## Example
```text
LIST          ' Everything.
LIST 120      ' Only line 120.
LIST 100-     ' All from 100 to the last.
LIST -100     ' All up to and including 100.
LIST 500-600  ' Only lines 500 to 600 inclusive.
```

*/
