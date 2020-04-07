/*!
# `DEFINT/SNG/DBL/STR <range of letters>`

## Purpose
Change type of undecorated variables.

## Remarks
By default, variables that don't specify a type with !,#,%,$ are Singles.
You can change the type of all variables that start with a particular letter.
Any existing variables not matching the new type are dropped.

## Example
```text
ST="NO DOLLAR$":?ST
?TYPE MISMATCH
DEFSTR S:ST="NO DOLLAR$":?ST
NO DOLLAR$
DEFSNG A-Z:?ST
 0
```

*/
