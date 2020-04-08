/*!
# Conversions and Compatibility

64K BASIC is the classic experience of 8-bit BASIC on a modern terminal.
Part of that experience was a slightly different version of BASIC for
each model of computer. BASIC was usually on ROM which wasn't trivial to
update like modern storage, so you were stuck with the same version for
the entire life of the computer. So instead of keeping your compiler up
to date as we do today, you would know how to adjust programs written
on another platform. Most books with program listings would include a page
or two about the dialect the code was written for. This is that page.

## `RANDOMIZE X`
Old computers often didn't have useful entropy, not even a real-time clock.
Many implementations would reset the random number generator to the same state
on ever run, but almost all would reset to the same state every boot.
`RANDOMIZE` was used to get around this by asking the user for a seed.
64K BASIC reseeds the random number generator with good entropy on every run,
so in most cases you simply delete the unneeded code.

`RANDOMIZE` without an X will prompt the user for a seed value.
This allowed someone to replay the same game if they wanted.
To emulate this behavior, here's a replacement.

```text
INPUT "Random Number Seed";A:RND=RND(-ABS(A))
```

Probably the most common way to seed the random number generator is to time
how long it takes the user to respond to a prompt. This is not necessary
in 64K BASIC so if you see something similar to the following, you can
delete it.

```text
PRINT "Press any key to continue";:WHILE INKEY$="":RND=RND():WEND:PRINT
```

## `GET A$`

To get a single keypress and store it in A$, 64K BASIC uses the newer style:
```text
A$ = INKEY$
```

## `LOCATE ROW,COL`

This is probably a GW-BASIC program so it might use graphics and sound.
However, in many cases it is simply used to center text on the screen.
Use a `PRINT` statement for each row you want to move down and `TAB(-X)`
to move to the column.
```text
REM CLS:LOCATE 3,20:PRINT "TITLE"
CLS:PRINT:PRINT:PRINT TAB(-20) "TITLE"
```

## `SOUND` and `BEEP`

64K BASIC does not support sound. Your terminal might with:
```text
PRINT CHR$(7)
```

*/
