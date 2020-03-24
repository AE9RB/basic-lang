/*!
# Functions
*/

pub mod COS {
    /*!
    ## COS(X)
    Returns the cosine of X in radians.
    ```text
    PRINT COS(0.123)
     0.99244505
    ```
    */
}

pub mod RND {
    /*!
    ## RND(X)
    Wichman-Hill Random Number Generator.
    Returns a random Single between 0 and 1 when X is missing or > 0.
    When X is 0, return the previous random number.
    When X < 0, seed the random number generator.
    ```text
    PRINT RND()
     0.6923401
    ```
    */
}

pub mod SIN {
    /*!
    ## SIN(X)
    Returns the sine of X in radians.
    ```text
    PRINT SIN(0.123)
     0.1226901
    ```
    */
}

pub mod TAB {
    /*!
    ## TAB(X)
    Used in a `PRINT` statement, moves to the requested column.
    Does nothing if already past the requested column.
    If X is negative, moves to the start of next -X wide zone.
    ```text
    PRINT 1.99 TAB(20) "furlongs per year"
     1.99               furlongs per year
    ```
    */
}
