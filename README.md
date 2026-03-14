# tomi.u

**tomi.u** is a modern, statically-typed programming language combining Python's clean syntax with C#'s strict type system. It features ownership-based memory management, first-class async programming, pattern matching, and exception handling.

## Repository Structure

```
tomi.u/
├── tomc/                    # tomi.u compiler (Rust)
├── vscode-extensions/
│   ├── tomi-language-support/   # Syntax highlighting, snippets, language config
│   └── tomi-files-icons/        # File icon theme
├── docs/                    # Language specification and architecture docs
│   ├── language-specification.md
│   ├── compiler-architecture.md
│   └── ROADMAP.md
└── .gitignore
```

## Language Features

- **Indentation-based syntax** — no braces or semicolons
- **`def` keyword** for function definitions
- **Exception handling** — `try/catch` (Java/JS style) and `try/except` (Python style)
- **Static typing** with type inference (`let x = 42`)
- **Immutability by default** (`let`) with explicit mutability (`mut`)
- **Pattern matching** via `match`
- **Async/await** built-in
- **Unicode identifiers**
- **Decorators** (`@entrypoint`, `@constructor`, etc.)

## Quick Example

```tomi.u
struct Point:
    x: i32
    y: i32

def add(a: i32, b: i32) -> i32:
    return a + b

@entrypoint
def main():
    let p = Point { x: 10, y: 20 }
    let sum = add(p.x, p.y)

    try:
        let result = process(sum)
    except ValueError as e:
        print(e)
```

## Compiler (`tomc`)

The `tomc` compiler translates tomi.u source code to Rust.

### Build

```sh
cd tomc
cargo build --release
```

### Run

```sh
tomc compile input.tu -o output.rs
```

### Test

```sh
cd tomc
cargo test
```

## VSCode Extensions

| Extension | Description |
|---|---|
| `tomi-language-support` | Syntax highlighting, snippets, indentation rules |
| `tomi-files-icons` | File icon theme for `.tu` files |

### Install Extensions

```sh
cd vscode-extensions/tomi-language-support
npm install
npm run package
code --install-extension tomi-language-support-*.vsix
```

## Documentation

- [Language Specification](docs/language-specification.md)
- [Compiler Architecture](docs/compiler-architecture.md)
- [Roadmap](docs/ROADMAP.md)

## Version

**0.1.0** — Initial release. Compiler supports:
- Lexer (all keywords including `def`, `try`, `catch`, `except`, `finally`, `raise`)
- Parser (functions, structs, enums, traits, control flow, exception handling)
- Rust code generation backend
- CLI via `tomc`

## License

MIT
