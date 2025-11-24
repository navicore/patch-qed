# QED: Next Steps

Prioritized roadmap for implementing the qed compiler.

## Phase 1: Parser Implementation (CURRENT)

### 1.1 Complete Lexer (`compiler/src/parser/lexer.rs`)
**Status:** ✅ Mostly done - tokens defined with logos

**Verify:**
- All tokens needed for examples are defined
- String escaping works correctly
- Comment handling (single-line `//`, multi-line `/* */`)

**Test with:**
```rust
cargo test --package qed-compiler lexer
```

### 1.2 Implement Grammar (`compiler/src/parser/grammar.rs`)
**Status:** ⚠️ Stubbed - needs full implementation

**Using chumsky parser combinators, implement:**

1. **Type expressions** - `Type`
   - Named types: `Int`, `String`, `Person`
   - Product types: `Person × Company`
   - Lists: `List<T>` (future)

2. **Type definitions** - `TypeDef`
   - Product: `type Person = person(name: String, age: Int)`
   - Sum: `type Level = Public | Internal | Secret`

3. **Relation declarations** - `RelationDecl`
   - `rel parent: Person × Person`

4. **Terms** - `Term`
   - Variables: `X`, `Y`, `Age`
   - Literals: `42`, `"hello"`
   - Constructors: `person("Alice", 45)`
   - Binary ops: `X + Y`, `Age * 2`

5. **Goals** - `Goal`
   - Atoms: `parent(X, Y)`
   - Unification: `X = person("Bob", 20)`
   - Comparisons: `Age > 18`, `X != Y`

6. **Facts** - `Fact`
   - `parent(person("Alice", 45), person("Bob", 20)).`

7. **Rules** - `Rule`
   - `ancestor(X, Y) :- parent(X, Y).`
   - `ancestor(X, Z) :- parent(X, Y), ancestor(Y, Z).`

8. **Queries** - `Query`
   - `?- ancestor(person("Alice", 45), Q).`

**Example to test:**
```bash
just check examples/family.qed
# Should parse without errors
```

**Reference:**
- Look at `../patch-seq/compiler/src/parser.rs` for patterns
- Chumsky docs: https://docs.rs/chumsky/latest/chumsky/

**Estimated effort:** 1-2 days

### 1.3 Wire Up Parser in `lib.rs`
**Status:** ✅ Already wired, just returns empty Program

**Update:**
```rust
pub fn parse(source: &str) -> Result<Program> {
    // Currently returns Ok(Program { items: vec![] })
    // Change to call actual parser
    let mut parser = Parser::new(source);
    parser.parse()
}
```

## Phase 2: Type Checker Implementation

### 2.1 Type Environment (`compiler/src/types/mod.rs`)
**Status:** ⚠️ Structure exists, methods stubbed

**Implement:**
1. `TypeEnv::add_type()` - Register type definitions
2. `TypeEnv::add_relation()` - Register relation signatures
3. Constructor lookup for ADTs

### 2.2 Type Checking
**Implement:**
1. `check_program()` - Two-pass:
   - Pass 1: Collect type/relation definitions
   - Pass 2: Check facts, rules, queries

2. `check_fact()` - Verify:
   - Relation exists
   - All terms are ground (no variables)
   - Term types match relation signature

3. `check_rule()` - Verify:
   - Head matches relation signature
   - Body goals are well-typed
   - Variable scoping (all head vars appear in body)
   - Safety (no unsafe negation, etc.)

4. `infer_term_type()` - Type inference for terms

**Test:**
```bash
just check examples/family.qed
# Should type-check successfully

just check examples/access_control.qed
# Should catch any type errors
```

**Estimated effort:** 2-3 days

## Phase 3: Code Generation (Critical Path)

### 3.1 IR Lowering (`compiler/src/ir/mod.rs`)
**Status:** ⚠️ IR types defined, lowering not implemented

**Implement:**
```rust
pub fn lower_to_ir(program: &Program, type_info: &TypeChecker) -> IrProgram
```

**Convert:**
- Types → Memory layouts (struct sizes, alignment)
- Facts → Static data tables
- Rules → IR instructions with mode analysis
- Identify which predicates need tabling

**Estimated effort:** 3-4 days

### 3.2 LLVM IR Generation (`compiler/src/codegen.rs`)
**Status:** ⚠️ Stub emits basic skeleton

**This is the big one!** Study `../patch-seq/compiler/src/codegen.rs` closely.

**Implement:**

1. **Type layouts**
   ```llvm
   %Person = type { ptr, i64 }  ; String name, i64 age
   ```

2. **Fact tables as static data**
   ```llvm
   @parent_facts = constant [2 x [2 x ptr]] [...]
   ```

3. **Runtime function declarations**
   - Already stubbed in current codegen.rs
   - Verify they match runtime exports

4. **Predicate compilation**
   - Generate function per relation per mode
   - Example: `parent_in_out(ptr %parent, ptr %child_out) -> i1`
   - Fact table iteration
   - Unification logic
   - Tabling for recursive predicates

5. **Main function**
   - Initialize query context (arena)
   - Execute queries
   - Print results (or return success/failure)

**Emit LLVM IR as string:** (like seq does)
```rust
writeln!(self.output, "define i32 @main() {{").unwrap();
writeln!(self.output, "entry:").unwrap();
// ...
```

**Test:**
```bash
just compile examples/family.qed family --keep-ir
cat family.ll  # Inspect LLVM IR
./family       # Run it!
```

**Estimated effort:** 5-7 days (most complex part)

### 3.3 Clang Integration (`compiler/src/lib.rs`)
**Status:** ✅ Already implemented, just needs codegen to work

**Current implementation:**
```rust
pub fn compile_file(source_path, output_path, keep_ir) -> Result<(), String> {
    // 1. Parse
    // 2. Type check
    // 3. Generate LLVM IR
    // 4. Write .ll file
    // 5. Call clang to link with libqed_runtime.a
}
```

**Just works once codegen is complete!**

## Phase 4: Runtime Enhancements

### 4.1 Proof Tree Generation
**Status:** ⚠️ Data structures defined, not used

**Enhance runtime to track proof trees:**
- Modify predicate functions to record derivations
- `QueryContext.track_proofs` flag
- Build `ProofNode` tree during evaluation

**Estimated effort:** 2-3 days

### 4.2 Explainability Rendering
**Add to runtime or compiler:**
- Proof tree → Human-readable explanation
- "X is true because..."
- Negative explanations ("X is false because...")

**Test with:**
```bash
just explain examples/access_control.qed "can_access(bob, doc1, Read)"
```

**Estimated effort:** 2-3 days

## Phase 5: Advanced Features

### 5.1 Negation (Stratified)
- Detect stratification violations
- Compile negation as failure

### 5.2 Aggregation
- `sum`, `count`, `min`, `max`
- See DESIGN.md for syntax options

### 5.3 Lists and Recursion
- Pattern matching: `[H | T]`
- Built-in list operations

### 5.4 Module System
- `module ... { export ... import ... }`

### 5.5 REPL
- Interactive query evaluation
- Load programs dynamically
- `:explain` command

## Quick Win: Get One Example Working

**Fastest path to working system:**

1. **Simplify family.qed** to minimal example:
   ```qed
   type Person = person(name: String)

   rel parent: Person × Person

   parent(person("Alice"), person("Bob")).

   main() :- parent(person("Alice"), person("Bob")).
   ```

2. **Hand-code LLVM IR** for this
   - See LLVM_IR.md for reference
   - Understand exactly what needs to be generated

3. **Implement codegen** to produce that IR

4. **Verify it compiles and runs**

5. **Expand from there**

This "tracer bullet" approach lets you:
- Test the full pipeline early
- Learn LLVM IR generation incrementally
- Have something working to demo

**Estimated effort:** 3-4 days for minimal end-to-end

## Recommended Order

**For fastest time-to-working-compiler:**

1. ✅ Foundation (DONE)
2. **Parser** (1-2 days) ← START HERE
3. **Minimal codegen** (hand-code LLVM for simple case) (1 day)
4. **Type checker basics** (1-2 days)
5. **Expand codegen** (3-4 days)
6. **IR lowering** (2-3 days)
7. **Runtime integration** (1-2 days)
8. **Proof trees & explainability** (2-3 days)
9. **Advanced features** (ongoing)

**Total to working compiler:** ~2-3 weeks focused work

## Resources

- **seq compiler:** `../patch-seq/compiler/src/`
  - Especially `codegen.rs` and `lib.rs`

- **Prolog implementations:**
  - WAM (Warren Abstract Machine) - classical approach
  - XSB - tabled evaluation

- **LLVM IR docs:**
  - https://llvm.org/docs/LangRef.html
  - Especially: functions, types, memory operations

- **Mercury language:**
  - Similar goals (typed logic programming)
  - https://mercurylang.org/

## Testing Strategy

**Unit tests:**
- Parser: Each grammar rule
- Type checker: Valid/invalid programs
- Codegen: IR generation correctness

**Integration tests:**
- Examples compile and run
- Proof trees are correct
- Explanations are accurate

**Add to justfile:**
```just
test-examples: build
    #!/usr/bin/env bash
    for f in examples/*.qed; do
        echo "Testing $f..."
        target/release/qedc compile "$f" -o /tmp/qed-test && /tmp/qed-test
    done
```

## When You Return

**First commands:**
```bash
cd qed  # (renamed directory)
just ci # Verify everything still works
git status # Check what's been done
cat STATUS.md # Read this file
cat NEXT_STEPS.md # Read this file (you are here!)
```

**Then pick up with:**
"Let's implement the parser" or "Let's do a tracer bullet with minimal family.qed"
