/*!
# `DATA <literal>[,<literal>]`

## Purpose
`DATA` defines a list of constants to be read in sequentially.

## Remarks
The `READ` statement will load the next data into a variable.
An `OUT OF DATA` error will occur when reading past the end.
Some versions of BASIC allow simple strings without quotes;
64K BASIC requires quotes.

## Example
```text
10 READ A$,A%
20 PRINT A$;A%
30 DATA "NUGGET",3
RUN
NUGGET 3
```

*/
