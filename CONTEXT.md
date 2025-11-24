# QED Project - Context at a Glance

**Last Session:** 2025-11-24  
**Status:** ✅ Foundation complete, ready for implementation  
**Directory:** Will be renamed from `leem/` to `qed/` or `patch-qed/`

## What QED Is

Typed logic programming language → compiles to native code (LLVM) → explainable reasoning

**Use cases:** Authorization, business rules, compliance, expert systems

## Project State

```
✅ DONE                    ⚠️ TODO
--------------------      --------------------
Workspace structure       Parser (grammar)
Runtime (arena, tables)   Type checker
CLI skeleton (qedc)       IR lowering
Lexer (tokens)           LLVM codegen ←BIG ONE
Build system (justfile)   Proof tree tracking
CI passing               REPL
Documentation            Advanced features
```

## Architecture (Mirrors patch-seq)

```
qed/
├── compiler/          # qed-compiler crate
│   └── src/
│       ├── codegen.rs    # Text LLVM IR gen ← IMPLEMENT THIS
│       ├── parser/       # ← START HERE
│       ├── types/        # Type checker
│       └── ir/           # Intermediate rep
└── runtime/           # qed-runtime (staticlib)
    └── src/lib.rs        # Arena, tables, FFI
```

## Key Decisions

| Decision | Choice | Why |
|----------|--------|-----|
| LLVM | Text-based IR | Simple, portable, debuggable |
| Concurrency | `may` crate | Shared with seq, scalable |
| Memory | Arena (`bumpalo`) | No GC, fast |
| Types | Static ADTs | Fast code, safety |
| Evaluation | Hybrid tabled | Predictable, explainable |

## Compilation Pipeline

```
.qed → parse → typecheck → IR → .ll → clang + libqed_runtime.a → binary
```

## Essential Files to Read

1. **RESUME.md** - Start here when returning
2. **STATUS.md** - Complete project status
3. **NEXT_STEPS.md** - Implementation roadmap
4. **DECISIONS.md** - Why we made choices
5. **DESIGN.md** - Language specification
6. **LLVM_IR.md** - What codegen produces

## Quick Commands

```bash
just ci              # Full CI (must pass!)
just build          # Build compiler + runtime
just test           # Run all tests
just fmt            # Format code
just check FILE     # Parse & typecheck (when ready)
just compile F O    # Compile to binary (when ready)
```

## Next Step Recommendations

**Option 1: Parser first** (systematic)
```bash
vim compiler/src/parser/grammar.rs
# Implement chumsky combinators
# Study: ../patch-seq/compiler/src/parser.rs
```

**Option 2: Tracer bullet** (faster to working system) ⭐
1. Create minimal `examples/minimal.qed`
2. Hand-write LLVM IR for it
3. Implement codegen to produce that IR
4. Get one example end-to-end
5. Expand incrementally

## Related Projects

**patch-seq** (formerly cem3): `../cem3/` or `../patch-seq/`
- Reference implementation for text LLVM codegen
- Study: `compiler/src/codegen.rs` and `compiler/src/lib.rs`
- Same workspace structure, same runtime patterns

## Example Programs (Test Cases)

```
examples/
├── family.qed           # Parent/ancestor (recursive)
├── business_rules.qed   # Employee classification
├── graph.qed           # Path finding
└── access_control.qed   # Authorization ← KILLER APP
```

## Success Criteria

| Milestone | Command | Success |
|-----------|---------|---------|
| 1. Parser | `just check examples/family.qed` | Parses ✓ |
| 2. Typeck | Same | Type checks ✓ |
| 3. Codegen | `just compile examples/minimal.qed m --keep-ir` | Valid .ll ✓ |
| 4. E2E | `just compile examples/minimal.qed m && ./m` | Runs ✓ |
| 5. Full | `just compile examples/family.qed f && ./f` | Works ✓ |

## Verify Before Starting

```bash
cd qed  # (after rename)
just ci
# Should output: ✅ All CI checks passed!
```

## The Vision

**Explainable** + **Fast** + **Typed** logic programming for **real-world** applications

Competitive with C in performance, better than Prolog in safety, unique in explainability.

---

*Remember: We're building this because LLMs are terrible at reasoning and the world needs auditable decision systems.*
