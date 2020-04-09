# Ready to play on 64K

The programs in this folder are retrieved by passing their name,
preceded by two slashes, to the 64K BASIC executable.
For example, to obtain Super Star Trek, use:

```
$ basic //superstartrek
Patch mode.

Super Star Trek puts you in command of the starship Enterprise.
Its 64K mission is to destroy the fleet of Klingon warships.

Retrieving from http://vintage-basic.net/bcg/superstartrekins.bas
Saving to superstartrekins.bas
Retrieving from http://vintage-basic.net/bcg/superstartrek.bas
Saving to superstartrek.bas
```

The program(s) will be downloaded to your local filesystem.
A line number 0 will be patched in with a link to instructions.

To avoid any copyright issues, the programs are not hosted here.
Instead, these files contain a link and patches.
The question mark is a shortcut to the official repository but
you can host your own collection of patches using the long form:

```
$ basic https://example.com/games/superstartrek
```

If the URL points to a file that doesn't have patch information,
the program is run as if it was loaded from the local filesystem.
