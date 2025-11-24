# Quick Resume Guide

**For when you come back to this project.**

## TL;DR

✅ **Foundation is complete and working**
⚠️ **Parser, type checker, and codegen need implementation**

## Verify Everything Still Works

```bash
cd qed  # (directory renamed from leem)
just ci
# Should see: ✅ All CI checks passed!
```

## What We Built

**Architecture:** Workspace with `compiler/` and `runtime/` (mirrors patch-seq)

**Key files:**
- `STATUS.md` - Full project status and decisions
- `NEXT_STEPS.md` - Detailed implementation roadmap
- `DESIGN.md` - Language specification + architecture
- `LLVM_IR.md` - LLVM compilation example

**Works:**
- ✅ Lexer (logos tokens defined)
- ✅ Runtime (arena, tables, FFI exports)
- ✅ Build system (justfile, CI passes)
- ✅ CLI skeleton (qedc binary)

**Needs implementation:**
- ⚠️ Parser (chumsky grammar)
- ⚠️ Type checker
- ⚠️ IR lowering
- ⚠️ LLVM codegen (the big one!)

## Where to Start

**Option 1: Linear approach**
```bash
# Implement parser first
vim compiler/src/parser/grammar.rs
# Study: ../patch-seq/compiler/src/parser.rs
```

**Option 2: Tracer bullet** (RECOMMENDED)
1. Create minimal `examples/minimal.qed`:
   ```qed
   type Person = person(name: String)
   rel parent: Person × Person
   parent(person("Alice"), person("Bob")).
   ```

2. Hand-write LLVM IR for it (see `LLVM_IR.md`)

3. Make codegen produce that IR

4. Expand from there

**This gets you end-to-end working fast!**

## Key Architecture Decisions (Made)

1. **Text-based LLVM IR** (not inkwell) - like seq
2. **`may` crate** for concurrency - Erlang-style green threads
3. **`bumpalo`** for arena allocation - no GC
4. **Runtime as staticlib** - links with compiled programs
5. **`clang`** for final linking

## Essential Reading

Before implementing:
1. `STATUS.md` - Understand current state
2. `NEXT_STEPS.md` - See implementation roadmap
3. `../patch-seq/compiler/src/codegen.rs` - Text LLVM IR patterns
4. `LLVM_IR.md` - What codegen needs to produce

## Quick Commands

```bash
# Build everything
just build

# Run tests
just test

# Check a qed file (when parser works)
just check examples/family.qed

# Compile a qed file (when everything works)
just compile examples/family.qed family

# Format code
just fmt

# Full CI
just ci
```

## Questions to Ask Yourself

1. **Do I understand the compilation pipeline?**
   `.qed` → parse → typecheck → IR → `.ll` → clang → binary

2. **Do I understand text-based LLVM IR generation?**
   Read `../patch-seq/compiler/src/codegen.rs` line by line

3. **Which component to tackle first?**
   - Parser if you like parsing
   - Codegen if you want to see output
   - Type checker if you like type systems
   - Recommend: tracer bullet (minimal example end-to-end)

## Files to Understand

**Critical path:**
1. `compiler/src/parser/grammar.rs` - WHERE TO START
2. `compiler/src/codegen.rs` - THE BIG IMPLEMENTATION
3. `compiler/src/lib.rs` - Wires everything together
4. `runtime/src/lib.rs` - Runtime that programs link against

**Supporting:**
- `compiler/src/ast/mod.rs` - Data structures (complete)
- `compiler/src/types/mod.rs` - Type checking (stubbed)
- `compiler/src/ir/mod.rs` - Intermediate rep (stubbed)

## Example Programs (Can't Compile Yet)

- `examples/family.qed` - Parent/ancestor relations
- `examples/business_rules.qed` - Employee classification
- `examples/graph.qed` - Path finding
- `examples/access_control.qed` - Authorization (killer app!)

These are your test cases!

## Success Metrics

**Milestone 1:** Parser works
```bash
just check examples/family.qed
# Parses successfully
```

**Milestone 2:** Type checker works
```bash
just check examples/family.qed
# Type checks successfully
```

**Milestone 3:** Codegen works
```bash
just compile examples/minimal.qed minimal --keep-ir
cat minimal.ll  # Valid LLVM IR
```

**Milestone 4:** End-to-end works
```bash
just compile examples/minimal.qed minimal
./minimal
# Runs successfully!
```

**Milestone 5:** Real examples work
```bash
just compile examples/family.qed family
./family
# Executes queries, prints results!
```

## Context from Last Session

**What we did:**
1. Renamed leem → qed (everywhere)
2. Restructured to workspace (compiler + runtime)
3. Modeled after patch-seq architecture
4. Decided on `may` for concurrency
5. Set up text-based LLVM IR approach
6. Got `just ci` passing

**What we discussed:**
- Why typed logic programming is useful
- QED vs Prolog vs Mercury vs Datalog
- Explainability as core feature
- Authorization rules as killer app
- Long-term: batteries-included stdlib (http, json, yaml, etc.)
- Integration potential with patch-seq

**Decision rationale:**
- `may` over tokio: shared with seq, future-proof for reactive rules
- Text LLVM over inkwell: simpler, portable, debuggable
- Arena allocation: no GC overhead
- Workspace structure: cleaner, mirrors seq

## When in Doubt

1. Look at `../patch-seq/` - it's the reference implementation
2. Read `STATUS.md` - it has all the context
3. Check `NEXT_STEPS.md` - it has the roadmap
4. Run `just ci` - make sure nothing broke

## The Vision

Build a **typed logic programming language** that compiles to **native code** and provides **explainable reasoning** for **real-world applications** like:
- Authorization and access control
- Business rules engines
- Compliance checking
- Planning and scheduling
- Expert systems

With **performance** comparable to C, **explainability** built-in, and **types** to catch bugs early.

**Let's make it happen!**

---

*Last updated: 2025-11-24*
*Project status: Foundation complete, ready for implementation*
*Next step: Implement parser or start tracer bullet*
