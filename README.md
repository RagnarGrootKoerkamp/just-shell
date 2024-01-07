# Just-Shell

A very minimal shell to invoke [`just`](https://github.com/casey/just)
commands with aggressive auto-completion, as an alternative to `just --choose`.

**Install**
-   `cargo install just-shell`
-   I recommend aliasing it to e.g. `js` in your shell.

**Features**
-   Fuzzyfind rules using
    [`fuzzy-matcher`](https://crates.io/crates/fuzzy-matcher).
-   Interactive shell using
    [`rustyline`](https://crates.io/crates/rustyline).

**TODO**
-   Print executed rule.
-   Hide the shell command printed by just
-   Aliases
-   Arguments
-   History
-   Ordering by usage count

[![asciicinema](https://asciinema.org/a/4KZpurHoiwrdRaugU5DS5JW35.svg)](https://asciinema.org/a/4KZpurHoiwrdRaugU5DS5JW35)
