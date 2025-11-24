# Architecture Decision Records

This document captures key architectural decisions and the reasoning behind them.

## ADR-001: Text-Based LLVM IR Generation (Not inkwell)

**Date:** 2025-11-24
**Status:** ✅ Accepted

### Context
Need to generate LLVM IR from qed programs. Two main approaches:
1. **inkwell** - FFI bindings to LLVM C++ API
2. **Text generation** - Emit `.ll` files as strings, use `clang`

### Decision
Use text-based LLVM IR generation.

### Rationale
- **Simplicity**: String formatting is simpler than managing FFI types
- **Portability**: No need for matching LLVM versions across systems
- **Debuggability**: Can inspect `.ll` files directly, easy to understand output
- **Proven**: patch-seq uses this approach successfully
- **Lower barrier**: Easier for contributors to understand

### Consequences
- ✅ Simpler codebase
- ✅ Easier to debug
- ✅ No LLVM version dependencies
- ⚠️ Slightly slower compile times (clang overhead)
- ⚠️ Can't use LLVM optimization API directly (but clang does this)

### Reference Implementation
See: `../patch-seq/compiler/src/codegen.rs`

---

## ADR-002: Concurrency Model (`may` vs tokio)

**Date:** 2025-11-24
**Status:** ✅ Accepted

### Context
Need concurrency primitives for:
- Future "batteries included" stdlib (http servers, json parsing, file I/O)
- Potential reactive rules engines
- Non-blocking query evaluation

Two main options:
1. **tokio** - Standard async runtime for Rust
2. **may** - Erlang-style green threads (CSP)

### Decision
Use `may` crate.

### Rationale

**Code sharing with patch-seq:**
- Both languages share concurrency model
- Common runtime patterns to optimize
- Potential for seq ↔ qed interop

**Erlang precedent:**
- Erlang = logic programming + lightweight processes
- Proven model for reactive systems
- Natural fit for rules engines

**Scalability:**
- Can handle millions of fibers if needed
- Good for event-driven rules (one fiber per event stream)
- Future: distributed knowledge bases

**Future-proofing:**
- Even if not needed initially, avoiding rebuilding `may` functionality later
- Non-blocking I/O is essential for modern systems

### Consequences
- ✅ Shared learning with patch-seq
- ✅ Scales to massive concurrency if needed
- ✅ Non-blocking I/O for stdlib
- ✅ Natural model for reactive rules
- ⚠️ Less familiar to Rust developers than tokio
- ⚠️ Smaller ecosystem than tokio

### Alternative Considered
**Pure Rust CSP without may:**
- Rejected: Would end up rebuilding `may` functionality
- Not worth the effort when `may` exists and is proven

---

## ADR-003: Memory Management (Arena Allocation)

**Date:** 2025-11-24
**Status:** ✅ Accepted

### Context
Logic programming typically involves lots of small allocations during query evaluation.

Options:
1. **Garbage collection** (like Prolog)
2. **Reference counting**
3. **Arena allocation**

### Decision
Use arena allocation with `bumpalo`.

### Rationale

**No GC overhead:**
- Native code performance
- Predictable latency
- Fits with "compile to native code" goal

**Natural fit for queries:**
- Each query gets its own arena
- All allocations during query evaluation use the arena
- Drop entire arena when query completes
- Stack-like lifecycle

**Proven in patch-seq:**
- Same approach works well there
- Can share arena implementation patterns

**Rust integration:**
- Arena lifetimes map to Rust ownership
- Natural for future Rust embedding

### Consequences
- ✅ No GC pauses
- ✅ Fast allocation (bump pointer)
- ✅ Simple memory model
- ⚠️ Must be careful about arena lifetimes
- ⚠️ Can't have long-lived query results without copying

### Implementation
- Runtime exports: `qed_arena_new()`, `qed_arena_alloc()`, `qed_arena_free()`
- Each compiled query function receives arena pointer
- Proof tracking is optional (can compile out for production)

---

## ADR-004: Type System Design

**Date:** 2025-11-24
**Status:** ✅ Accepted

### Context
Traditional Prolog is dynamically typed. Modern alternatives (Mercury, Formulog) are statically typed.

### Decision
Static typing with algebraic data types (ADTs).

### Rationale

**LLVM optimization:**
- Static types → unboxed, specialized code
- No runtime type checks or tagged unions
- Known memory layouts enable optimization

**Compile-time safety:**
- Catch nonsensical rules at compile time
- Example: Can't unify `Int` with `Person`

**Explainability:**
- Typed relations are self-documenting
- `ancestor: Person × Person` is clearer than `ancestor/2`

**Mode analysis:**
- Types + modes enable compile-time optimizations
- Generate different code for `parent(in, out)` vs `parent(in, in)`

**Rust synergy:**
- ADTs map directly to Rust enums/structs
- Natural for future Rust embedding

### Consequences
- ✅ Fast compiled code
- ✅ Type errors caught early
- ✅ Self-documenting programs
- ✅ Enables mode-directed compilation
- ⚠️ More complex than dynamic typing
- ⚠️ Type inference needed (but achievable)

### Inspiration
- **Mercury**: Static types + modes
- **Rust**: ADTs (enums + structs)
- **Not Prolog**: Everything-is-a-term flexibility sacrificed for safety

---

## ADR-005: Workspace Structure

**Date:** 2025-11-24
**Status:** ✅ Accepted

### Context
Project organization: monorepo vs separate crates, module structure.

### Decision
Workspace with two crates: `compiler/` and `runtime/`.

### Rationale

**Mirrors patch-seq:**
- Proven structure
- Consistent across Patch project
- Easier to maintain both languages

**Clear separation:**
- Compiler: development-time
- Runtime: links with compiled programs
- Different optimization needs

**Runtime as staticlib:**
```toml
crate-type = ["staticlib", "rlib"]
```
- `staticlib` for linking with LLVM-generated code
- `rlib` for Rust unit tests

**Easier later:**
- Setting up workspace now avoids painful refactor later
- Common dependencies managed at workspace level

### Consequences
- ✅ Clear separation of concerns
- ✅ Runtime can be linked independently
- ✅ Consistent with patch-seq
- ✅ Easy to test runtime separately
- ⚠️ Slightly more complex project structure initially

---

## ADR-006: Evaluation Strategy (Hybrid)

**Date:** 2025-11-24
**Status:** ✅ Accepted (See DESIGN.md)

### Context
Logic programming evaluation strategies:
1. **Top-down** (SLD resolution, like Prolog)
2. **Bottom-up** (Datalog-style)
3. **Tabled** (XSB-style memoization)

### Decision
Hybrid approach:
- Tabled evaluation for recursive rules
- Top-down goal-directed for queries
- Mode-directed compilation

### Rationale

**Tabling prevents infinite loops:**
- Memoize intermediate results
- Much more predictable performance than raw Prolog

**Top-down for queries:**
- Generate proof trees (essential for explainability)
- Natural query-driven execution

**Mode-directed compilation:**
- Different code for different input/output patterns
- Enables optimizations

### Implementation Strategy
- Hash tables for tabling (runtime)
- Track which predicates need tabling (IR analysis)
- Generate different LLVM functions per mode

### Consequences
- ✅ Predictable performance
- ✅ Explainability via proof trees
- ✅ Optimizations via mode analysis
- ⚠️ More complex than pure top-down
- ⚠️ Requires mode inference or annotations

---

## ADR-007: Explainability as First-Class Feature

**Date:** 2025-11-24
**Status:** ✅ Accepted

### Context
Motivation: LLMs are being misused for reasoning in critical applications ("disaster in the making").
Need: Systems that can explain their decisions.

### Decision
Build explainability into the language core, not as an afterthought.

### Features

**Proof trees:**
- Track every rule application
- Record facts used
- Build derivation tree

**Human-readable justifications:**
```
Bob is eligible for a bonus of $24,000 because:
  1. Bob is classified as Manager
     - Bob's salary ($120,000) >= $100,000
     - Bob's years of service (12) >= 5
  2. Manager bonus formula is salary / 5
     - $120,000 / 5 = $24,000
```

**Negative explanations:**
```
Bob cannot access secret_document because:
  - Bob has roles: [Employee]
  - secret_document sensitivity: Secret
  - No applicable authorization rules:
    ✗ Not owner (owner is Alice)
    ✗ Not Admin role
    ✗ Not Public resource
```

### Killer App: Authorization
Access control decisions MUST be auditable and explainable.
qed makes this natural and efficient.

### Implementation
- Proof tracking optional (compile flag)
- Can compile out for production performance
- Runtime: `QueryContext.track_proofs` flag
- Templates for rendering explanations

### Consequences
- ✅ Auditable decisions
- ✅ Debugging aid
- ✅ Trust building
- ✅ Regulatory compliance (GDPR, etc.)
- ⚠️ Small runtime overhead if enabled
- ✅ Can compile out for production

---

## ADR-008: No Module System (Initially)

**Date:** 2025-11-24
**Status:** ✅ Deferred

### Context
Large programs need organization.

### Decision
Defer module system to later.

### Rationale
- Get core language working first
- Learn from usage patterns
- Avoid premature design

### Consequences
- ⚠️ Large programs will be monolithic initially
- ✅ Simpler initial implementation
- ✅ Can design better system with experience

---

## ADR-009: Negation Strategy

**Date:** 2025-11-24
**Status:** ⚠️ Proposed (See DESIGN.md)

### Context
How to handle negation (`not P`)?

Options:
1. **Stratified negation** (Datalog-style)
2. **Negation as failure** (Prolog-style)
3. **Explicit closed-world assumption**

### Proposal
Stratified negation with compile-time checking.

### Rationale
- Predictable semantics
- Compile-time validation
- Fits with type-checking philosophy

### To Be Decided
- Syntax for negation
- How to detect non-stratification
- Error messages

---

## Decision Process

When making architectural decisions, consider:

1. **Alignment with vision**: Does it support typed, fast, explainable logic programming?
2. **Patch ecosystem**: How does it fit with patch-seq?
3. **Simplicity**: Can we explain it in one page?
4. **Proven**: Has someone else done this successfully?
5. **Future-proof**: Will we regret this in 2 years?

---

*These decisions can be revisited if new information emerges.*
*Document the reasoning for changes.*
