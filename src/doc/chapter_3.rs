/*!
# Functions
*/

pub mod ABS {
    /*!
    ## `ABS(X)` Returns the absolute value of X.
    ```text
    PRINT ABS(-0.123)
     0.123
    ```
    */
}

pub mod ASC {
    /*!
    ## `ASC(X$)` Returns the unicode value of the first character of X.
    ```text
    PRINT ASC("A")
     65
    ```
    */
}

pub mod ATN {
    /*!
    ## `ATN(X)` Returns the arctangent of X.
    ```text
    PRINT ATN(3)
      1.2490457
    ```
    */
}

pub mod CDBL {
    /*!
    ## `CDBL(X)` Returns X as a Double.
    ```text
    PRINT 1/CDBL(3)
      0.3333333333333333
    ```
    */
}

pub mod CHR {
    /*!
    ## `CHR$(X)` Returns a character of ASCII X.
    ```text
    PRINT CHR$(65)
    A
    ```
    */
}

pub mod CINT {
    /*!
    ## `CINT(X)` Returns X as an Integer. Overflow error mar occur.
    ```text
    PRINT CINT(-9.9)
    -10
    ```
    */
}

pub mod COS {
    /*!
    ## `COS(X)` Returns the cosine of X in radians.
    ```text
    PRINT COS(0.123)
     0.99244505
    ```
    */
}

pub mod CSNG {
    /*!
    ## `CSNG(X)` Returns X as a Single.
    ```text
    PRINT 1/CSNG(3#)
      0.33333334
    ```
    */
}

pub mod DATE {
    /*!
    ## `DATE$` Returns the system date.
    ```text
    PRINT DATE$
    12-31-2000
    ```
    */
}

pub mod EXP {
    /*!
    ## `EXP(X)` Returns e to the power of X.
    ```text
    PRINT EXP(1)
     2.7182817
    ```
    */
}

pub mod FIX {
    /*!
    ## `FIX(X)` Returns the truncated integer value of X.
    See `INT` for a flooring function.
    ```text
    PRINT FIX( 9.9) FIX(-9.9)
     9  9
    ```
    */
}

pub mod HEX {
    /*!
    ## `HEX$(X)` Returns hexadecimal string of integer X.
    ```text
    PRINT HEX$(-1)
    FFFF
    ```
    */
}

pub mod INKEY {
    /*!
    ## `INKEY$` Returns a single key press, empty string if none.
    ```text
    I$="":WHILE LEN(I$)=0:I$=INKEY$:WEND:PRINT I$
    ```
    */
}

pub mod INT {
    /*!
    ## `INT(X)` Returns the largest integer <= X.
    See `FIX` for a truncating function.
    ```text
    PRINT INT(9.9) INT(-9.9)
     9 -10
    ```
    */
}

pub mod LEFT {
    /*!
    ## `LEFT$(A$,X)` Returns the leftmost X characters of A$.
    ```text
    PRINT LEFT$("HUNT THE WUMPUS", 4)
    HUNT
    ```
    */
}

pub mod LEN {
    /*!
    ## `LEN(X$)` Returns the number of characters in X$.
    ```text
    PRINT LEN("TO")
     2
    ```
    */
}

pub mod LOG {
    /*!
    ## `LOG(X)` Returns the natural logarithm of X.
    ```text
    PRINT LOG(8/37)
    -1.5314764
    ```
    */
}

pub mod MID {
    /*!
    ## `MID$(A$,X,[Y])` Returns a portion of A$.
    The returned string will begin with the character in position X.
    If Y is present it will limit the length, otherwise everything
    until the end of the string is returned.
    ```text
    PRINT MID$("HUNT THE WUMPUS", 6, 3)
    THE
    PRINT MID$("HUNT THE WUMPUS", 6)
    THE WUMPUS
    ```
    */
}

pub mod OCT {
    /*!
    ## `OCT$(X)` Returns octal string of integer X.
    ```text
    PRINT OCT$(-1)
    177777
    ```
    */
}

pub mod RIGHT {
    /*!
    ## `RIGHT$(A$,X)` Returns the rightmost X characters of A$.
    ```text
    PRINT RIGHT$("HUNT THE WUMPUS", 6)
    WUMPUS
    ```
    */
}

pub mod POS {
    /*!
    ## `POS(X)` Returns the horizontal cursor position of the terminal.
    `X` is optional and ignored. First position is 0.
    ```text
    PRINT "     ";POS()
          5
    ```
    */
}

pub mod RND {
    /*!
    ## `RND(X)` Returns a pseudo-random number.
    Wichman-Hill Random Number Generator.
    Returns a random Single between 0 and 1 when X is missing or > 0.
    When X is 0, return the previous random number.
    When X < 0 the random number generator is seeded with X.
    ```text
    PRINT RND()
     0.6923401
    ```
    */
}

pub mod SGN {
    /*!
    ## `SGN(X)` Returns the sign of X.
    Returns -1 if X is negative, 1 if positive, and 0 if zero.
    ```text
    PRINT SGN(+1)
     1
    ```
    */
}

pub mod SIN {
    /*!
    ## `SIN(X)` Returns the sine of X in radians.
    ```text
    PRINT SIN(0.123)
     0.1226901
    ```
    */
}

pub mod SQR {
    /*!
    ## `SQR(X)` Returns the square root of X.
    ```text
    PRINT SQR(5)
     2.236068
    ```
    */
}

pub mod STR {
    /*!
    ## `STR$(X)` Returns the number X as a string.
    ```text
    PRINT STR$(-3.14) + "!"
     -3.14!
    ```
    */
}

pub mod TAB {
    /*!
    ## `TAB(X)` Returns a string of spaces.
    Used in a `PRINT` statement, moves to the requested column.
    Does nothing if already past the requested column.
    If X is negative, moves to the start of next -X wide zone.
    ```text
    PRINT 1.99 TAB(20) "furlongs per year"
     1.99               furlongs per year
    ```
    */
}

pub mod TAN {
    /*!
    ## `TAN(X)` Returns the tangent of X in radians.
    ```text
    PRINT TAN(5/13)
     0.40477434
    ```
    */
}

pub mod TIME {
    /*!
    ## `TIME$` Returns the system time.
    ```text
    PRINT TIME$
    23:59:59
    ```
    */
}

pub mod VAL {
    /*!
    ## `VAL(X$)` Returns a number parsed from string X$.
    ```text
    PRINT VAL("1E-2")
     0.01
    ```
    */
}
