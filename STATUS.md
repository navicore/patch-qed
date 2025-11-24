# QED Project Status

**Last Updated:** 2025-11-24

## Current State: Foundation Complete ✅

The qed project has been successfully renamed from "leem" and restructured to match the seq (formerly cem3) architecture.

### Completed Work

#### 1. Project Rename
- ✅ `leem` → `qed` (all references updated)
- ✅ `leemc` → `qedc` binary
- ✅ `cem3/ceem` → `patch-seq` references
- ✅ Example files: `*.leem` → `*.qed`
- ✅ All documentation updated

#### 2. Workspace Architecture (Modeled after seq)
```
qed/
├── Cargo.toml              # Workspace root
├── compiler/               # qed-compiler crate
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         # qedc CLI binary
│       ├── lib.rs          # Compiler library with compile_file()
│       ├── ast/            # AST definitions
│       ├── parser/         # Lexer (logos) + grammar (chumsky)
│       ├── types/          # Type checker
│       ├── ir/             # Intermediate representation
│       └── codegen.rs      # Text-based LLVM IR generation
└── runtime/                # qed-runtime crate
    ├── Cargo.toml          # [lib] crate-type = ["staticlib", "rlib"]
    └── src/lib.rs          # Arena, Tables, FFI exports
```

#### 3. Key Architectural Decisions

**Memory Management:**
- Using `bumpalo` for arena allocation (same as seq)
- Each query gets its own arena
- No garbage collection

**Concurrency:**
- Using `may` crate for Erlang-style green threads
- CSP concurrency model
- Chosen for:
  - Code sharing with seq
  - Non-blocking I/O for future stdlib (http, json, yaml, fileio)
  - Potential for reactive rules engines
  - Future-proofing for distributed/incremental computation

**LLVM Backend:**
- **Text-based IR generation** (like seq, NOT inkwell)
- Generate `.ll` files as strings
- Use `clang` to compile and link with `libqed_runtime.a`
- Much simpler than FFI bindings

**Compilation Pipeline:**
```
.qed source → parse → typecheck → codegen → .ll file
                                               ↓
                                            clang + libqed_runtime.a
                                               ↓
                                           executable
```

#### 4. Dependencies

**Workspace Level:**
- `may` = "0.3" - Green threads
- `bumpalo` = "3.16" - Bump allocation
- `libc` = "0.2" - System calls
- `logos` = "0.14" - Lexer
- `chumsky` = "0.9" - Parser combinators
- `clap` = "4.5" - CLI
- `anyhow`, `thiserror` - Error handling
- `ariadne`, `codespan-reporting` - Pretty diagnostics

**Runtime Exports (C ABI):**
```c
ptr qed_arena_new(i64 capacity)
void qed_arena_free(ptr arena)
ptr qed_arena_alloc(ptr arena, i64 size, i64 align)
ptr qed_table_new(i64 bucket_count)
void qed_table_free(ptr table)
void qed_table_insert(ptr table, i64 key_hash, ptr key, ptr value)
```

#### 5. Build System (justfile)

```bash
just build           # Build runtime + compiler
just build-runtime   # Just runtime staticlib
just build-compiler  # Just compiler binary
just test           # Run all tests
just clippy         # Lint
just fmt            # Format
just ci             # Full CI pipeline (GitHub Actions uses this)
```

**CI Status:** ✅ All checks passing
- Code formatting ✓
- Clippy lints ✓
- 19 unit tests ✓
- Release build ✓

### What's Stubbed/Incomplete

#### Compiler (`compiler/src/`)
- ✅ `ast/` - Full ADT definitions for types, relations, facts, rules, queries
- ⚠️ `parser/` - Lexer defined (logos), grammar stubbed (chumsky)
- ⚠️ `types/` - Type checker structure exists, methods stubbed
- ⚠️ `ir/` - IR types defined, lowering not implemented
- ⚠️ `codegen.rs` - Stub that emits basic LLVM IR skeleton

#### Runtime (`runtime/src/lib.rs`)
- ✅ Arena allocator - Complete with tests
- ✅ Hash tables for tabling - Complete with tests
- ⚠️ Proof tree structures - Defined but not used yet
- ✅ FFI exports - All declared

#### Examples (`examples/`)
Four comprehensive example programs exist but can't compile yet:
- `family.qed` - Recursive rules (parent, ancestor, sibling)
- `business_rules.qed` - Classification, bonus eligibility
- `graph.qed` - Path finding
- `access_control.qed` - Authorization rules (killer app!)

### Design Documents

- **DESIGN.md** - Full language specification
  - Type system (ADTs, product/sum types)
  - Relations and facts
  - Evaluation strategy (hybrid tabled + top-down)
  - Explainability architecture
  - Open questions (negation, aggregation, modes, lists)

- **LLVM_IR.md** - Detailed compilation example
  - Shows how family.qed would compile to LLVM IR
  - Data layouts, fact tables, tabling for recursion

- **README.md** - Project overview with examples

### Integration with Patch Ecosystem

QED is the second language in the Patch project:
- **patch-seq**: Concatenative (stack-based) programming
- **patch-qed**: Logic (fact-based) programming

Both:
- Compile to native code via LLVM
- Use Rust + LLVM infrastructure
- No garbage collection
- Share concurrency model (`may`)
- Long-term: Embeddable in Rust, interoperate via Rust

## Directory Rename

**IMPORTANT:** This directory is currently named `leem/` but should be renamed to match the GitHub repo `patch-qed`.

**Safe to rename** - All internal references use `qed`, not the directory name.

After rename, verify:
```bash
just ci  # Should still pass
```

## Related Projects

- **patch-seq** (formerly cem3): `../cem3` or `../patch-seq`
  - Study `compiler/src/codegen.rs` for text-based LLVM IR generation patterns
  - Study `compiler/src/lib.rs` for compile_file() implementation
  - Runtime structure is identical

## Questions to Resume With

When picking this up again, ask yourself:
1. Does `just ci` still pass?
2. Which component to implement first? (Recommendation: parser)
3. Do we need to revisit any architecture decisions?
