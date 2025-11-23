# qed

**Part of the Patch Project** — Fact-based logic programming that compiles to native code.

A typed logic programming language that compiles to fast, native code via LLVM.

## Overview

**qed** combines the expressiveness of logic programming with the performance of compiled languages. It's designed for building explainable rules engines and fact-based systems where you need:

- **Type safety**: Static typing with algebraic data types
- **Performance**: Native code via LLVM, no GC, arena allocation
- **Explainability**: Proof trees and human-readable justifications
- **Practicality**: Real-world applications like authorization, compliance, business rules

## Example

```qed
type Person = person(name: String, age: Int)

rel parent: Person × Person
rel ancestor: Person × Person

parent(person("Alice", 45), person("Bob", 20)).
parent(person("Bob", 20), person("Charlie", 2)).

ancestor(X, Y) :- parent(X, Y).
ancestor(X, Z) :- parent(X, Y), ancestor(Y, Z).

?- ancestor(person("Alice", 45), Q).
```

## Features

- **Algebraic Data Types**: Product types (structs) and sum types (enums)
- **Typed Relations**: Statically checked relation signatures
- **Pattern Matching**: Destructure data in rules
- **Explainability**: Generate proof trees and justifications
- **LLVM Backend**: Compile to optimized native code
- **No Garbage Collection**: Arena-based memory management

## Project Structure

```
qed/
├── src/
│   ├── ast/         # Abstract syntax tree definitions
│   ├── parser/      # Lexer and parser (logos + chumsky)
│   ├── types/       # Type checking and inference
│   ├── ir/          # Intermediate representation
│   ├── codegen/     # LLVM code generation (inkwell)
│   ├── runtime/     # Runtime support (arena, tables)
│   └── main.rs      # CLI interface
├── examples/        # Example qed programs
├── DESIGN.md        # Language design document
└── LLVM_IR.md       # LLVM IR compilation guide

```

## Building

Requires Rust 1.70+ and LLVM 17.

```bash
cargo build --release
```

## Usage

```bash
# Compile a qed program
qedc compile program.qed -o program

# Type check only
qedc check program.qed

# Show proof tree for a query
qedc explain program.qed "ancestor(alice, X)"

# Start REPL
qedc repl
```

## Development Status

**Early stage development** - Currently implementing:

- [x] Project structure
- [x] AST definitions
- [x] Lexer
- [ ] Parser
- [ ] Type checker
- [ ] IR generation
- [ ] LLVM codegen
- [ ] Runtime
- [ ] REPL

See `DESIGN.md` for the full language specification.

## Comparison to Other Languages

| Feature | Prolog | Datalog | Mercury | qed |
|---------|--------|---------|---------|------|
| Type System | Dynamic | Dynamic | Static | Static (ADTs) |
| Compilation | Interpreted | Interpreted | Native | Native (LLVM) |
| Explainability | Limited | Limited | None | Built-in |
| Memory | GC | Varies | GC | Arena (no GC) |
| Target Use | General | Databases | General | Rules engines |

## Inspiration

- **Prolog**: Logic programming model, unification
- **Mercury**: Type system, mode analysis
- **Datalog**: Stratified evaluation, practicality
- **Rust**: Memory safety, type system, tooling

## License

MIT

## Related Projects

- [patch-seq](../patch-seq): Concatenative language with Rust + LLVM (sister project under Patch)
