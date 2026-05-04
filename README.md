# Lox-rs/Loxrs

A [lox](https://craftinginterpreters.com/the-lox-language.html) parser written
in [rust](https://rust-lang.org/).

A huge shout-out to Robert Nystrom for the [book](https://craftinginterpreters.com/) _Crafting Interpreters_ and the `lox` language!

## Purpose

This repository is just a private project based on the fact that I always
wanted to write a programming language parser and I really needed to have
a project that is complete. Code, documentation, tests. 

## Complete

In order to consider this project as  _completed_ the following goals are 
suggested:
- [x] Scanning: parse source code into tokens
- [ ] Parsing: create recurseive descent parser - `AST`
- [ ] Intepreter: walk the ast

As mentioned above, documentation and tests are mandatory.

## Quick Start

In the root of the project run:

```console
$ cargo run [FILE TO PARSE]
```
e.g. 
```console
$ cargo run ./test.lox
```

## Tests

```console
$ cargo test
```

## License

This piece of code is licensed under the MIT License. The license applies 
to all source files.
