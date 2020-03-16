# 64K BASIC

The BASIC programming language as it was on 8-bit CPUs.

[![asciicast](https://asciinema.org/a/1iKC7OZXMyAVUF2Sqw404wTtn.svg)](https://asciinema.org/a/1iKC7OZXMyAVUF2Sqw404wTtn?speed=2&autoplay=true)

Future versions of BASIC used more structure and less
lines numbers and GOTOs. This project is a historical
preservation of BASIC just before then. A time when most
people were learning about computers by typing in BASIC
programs published in books and magazines.

The first computer book to sell one million copies was a
collection of BASIC Computer Games. It was followed up with
More BASIC Computer Games. You can run all those and more
with this dialect of BASIC.

## Installation

There are many missing many statements and functions but everything that is currently implemented should be fully working with good error messages and no panics. Until I release binaries, you'll need Rust
to install. Use:
```
$ cargo install basic-lang
```
To uninstall:
```
$ cargo uninstall basic-lang
```
Or `cargo build` from the latest source, of course.
