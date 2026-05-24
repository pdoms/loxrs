# lox-rs
A tree-walking interpreter for the [Lox](https://craftinginterpreters.com/the-lox-language.html) 
language, written in Rust.

Huge shout-out to Robert Nystrom for [Crafting Interpreters](https://craftinginterpreters.com/) 
and the Lox language!

## About
A personal project born from wanting to build something complete end-to-end —
scanner, parser, static analysis, and interpreter — with tests and documentation
throughout. Lox is a small but non-trivial dynamically typed scripting language
that supports first-class functions, closures, and lexical scoping.

Classes (`class`, `this`, `super`) are not implemented.

## Architecture
Source code flows through four stages:

**Scanner** (`scanner.rs`) — reads raw source bytes and emits a flat list of
tokens. Handles string literals, numbers, identifiers, keywords, and two-character
operators with one character of lookahead.

**Parser** (`parser.rs`) — recursive descent parser that consumes tokens and
builds an AST. Implements the full Lox expression grammar including operator
precedence, logical short-circuiting, and desugars `for` loops into `while` loops.

**Resolver** (`resolver.rs`) — static analysis pass that walks the AST before
interpretation. Resolves variable bindings to exact scope depths, catches use of
variables in their own initializers, and detects `return` statements outside
functions.

**Interpreter** (`interpreter.rs`) — tree-walking interpreter that evaluates the
AST. Environments are linked via `Rc<RefCell<...>>` to support closures that
correctly share mutable state across calls.

## Example

```lox
fun makeCounter() {
    var count = 0;
    fun increment() {
        count = count + 1;
        return count;
    }
    return increment;
}

var counter = makeCounter();
print counter();  // 1
print counter();  // 2
```

## Native Functions
| Function | Args | Description |
|---|---|---|
| `clock()` | 0 | Unix timestamp as float |
| `sqrt(x)` | 1 | Square root |
| `floor(x)` | 1 | Round down |
| `ceil(x)` | 1 | Round up |
| `abs(x)` | 1 | Absolute value |
| `pow(x, y)` | 2 | x to the power of y |
| `type_of(x)` | 1 | Type name as string |
| `to_string(x)` | 1 | Convert to string |
| `to_number(x)` | 1 | Parse string to number, nil on failure |
| `len(x)` | 1 | String length |
| `read_file(path)` | 1 | Read file contents, nil on failure |
| `write_file(path, content)` | 2 | Write file, returns bool |
| `append_file(path, content)` | 2 | Append to file, returns bool |

## Quick Start
```console
$ cargo run [--verbose/-v] <FILE>
```

## Tests
```console
$ cargo test
```

## Goals
- [x] Scanning
- [x] Parsing — recursive descent, full expression grammar
- [x] Resolver — static variable resolution, error detection
- [x] Interpreter — tree-walking, closures, native functions
- [ ] Classes (not planned, as i am not a OOP person)

## License
MIT
