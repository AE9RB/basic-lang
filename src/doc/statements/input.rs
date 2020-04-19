/*!
# `INPUT [,]["<prompt string>";]<variable>[,<variable>...]`

## Purpose
Suspends execution and awaits a response from the terminal.

## Remarks
64K BASIC was designed to run programs from a time when lowercase was unexpected.
INPUT will capitalize ASCII lowercase by default. You can disable this feature
with a comma immediately after the INPUT.

## Example
```text
10 INPUT ,A$
20 INPUT "WHAT IS YOUR NAME AND AGE"; NAME$, AGE%
```

*/
