/*!
# `DELETE [<from line number>][-<to line number>]`

## Purpose
Delete lines from the BASIC program currently in memory.

## Remarks
You must specify a line or range.

## Example
```text
DELETE          ' ?ILLEGAL FUNCTION CALL
DELETE 120      ' Only line 120.
DELETE 100-     ' All from 100 to the last.
DELETE -100     ' All up to and including 100.
DELETE 500-600  ' Only lines 500 to 600 inclusive.
```

*/
