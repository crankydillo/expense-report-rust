# Introduction

Messing around with [Rust](https://www.rust-lang.org/en-US/).  I'm attempting a
port of [expense-report](https://github.com/crankydillo/expense-report) from
Node to Rust.  Most important goal is learning; however, I am interested in a
lower hardware footprint.

This may not seem very Rust-like.  My initial effort is basically a
line-to-line port of moderately-functional JavaScript to moderately functional
Rust.  Basically, just using the language support for function calls.

# TODO

### Error handling

Quit unwrapping everywhere..  Just return `Result`.

### Prepared statements

