# leem Language Design

**leem**: A typed logic programming language that compiles to fast, native code via LLVM.

## Vision

leem is a logic programming language designed for building explainable rules engines and fact-based systems. Unlike traditional Prolog:
- **Statically typed** with algebraic data types
- **Compiles to native code** via LLVM (no GC, no interpreter overhead)
- **Built for explainability** - proof trees to human-readable justifications
- **Practical** - designed for real-world applications like authorization, compliance, business rules

## Design Principles

1. **Explainability First**: Every conclusion must be traceable to its logical derivation
2. **Type Safety**: Catch logical errors at compile time
3. **Performance**: Native code execution, arena allocation, no GC
4. **Pragmatism**: Not trying to be Prolog - make different choices where it helps

## Type System

### Algebraic Data Types

leem supports product types (structs) and sum types (enums):

```leem
// Product type (struct-like)
type Person = person(name: String, age: Int)

// Sum type (enum-like)
type Level =
    | Public
    | Internal
    | Confidential
    | Secret

// Nested types
type Employee = employee(
    id: Int,
    name: String,
    person: Person
)
```

### Built-in Types

- `Int`: 64-bit signed integers
- `String`: UTF-8 strings
- `Bool`: true/false
- `List<T>`: Homogeneous lists (to be designed)
- `Option<T>`: Maybe type (to be designed)

### Type Inference

leem uses bidirectional type inference:
- Relation signatures must be declared
- Variable types are inferred from usage
- Function-like expressions are checked against expected types

## Relations and Facts

### Relation Declarations

Relations define the schema for facts and rules:

```leem
rel parent: Person × Person              // Binary relation
rel age: Person × Int                    // Person to Int
rel works_in: Employee × Department      // Multi-type relation
```

### Facts

Facts are ground instances of relations (no variables):

```leem
parent(person("Alice", 45), person("Bob", 20)).
age(person("Alice", 45), 45).
```

## Rules and Queries

### Basic Rules

Rules define logical inference:

```leem
// Simple rule
ancestor(X, Y) :- parent(X, Y).

// Recursive rule
ancestor(X, Z) :- parent(X, Y), ancestor(Y, Z).

// Multiple conditions
sibling(X, Y) :-
    parent(P, X),
    parent(P, Y),
    X != Y.
```

### Pattern Matching

Destructure ADTs in rule heads and bodies:

```leem
// Match structure in rule body
eligible_for_bonus(Emp, Bonus) :-
    employee(Id, Name, Years) = Emp,
    Years >= 5,
    Bonus = 1000.

// Match in relation call
classification(employee(_, _, Sal, Years), Senior) :-
    Sal >= 70000,
    Years >= 3.
```

### Expressions

leem supports limited expressions in rule bodies:

```leem
age_difference(person(_, Y1), person(_, Y2), Diff) :-
    Diff = Y1 - Y2.

eligible(Person, Amount) :-
    age(Person, Age),
    Age >= 18,
    Amount = Age * 100 + 500.
```

Supported operators:
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Logical: `&&`, `||`, `!` (in guards, not full boolean logic)

## Evaluation Strategy

### Hybrid Approach

leem uses a **hybrid evaluation strategy**:

1. **Tabled/Memoized** for recursive rules
   - Bottom-up evaluation for stratified rules
   - Memoize intermediate results
   - Predictable performance

2. **Top-down goal-directed** for queries
   - SLD resolution with tabling
   - Generate proof trees for explainability

3. **Mode-directed compilation**
   - Compile different code paths based on input/output patterns
   - Static analysis determines which arguments are ground at call time

### Memory Model

- **Arena allocation**: Each query gets an arena
- **No garbage collection**: Arenas are dropped when query completes
- **Stack-based unification**: Where possible, avoid heap allocation
- **Specialized code**: LLVM optimizes based on type information

## Explainability

### Proof Trees

Every successful query produces a proof tree:

```
Query: ancestor(alice, david)
Proof:
  ancestor(alice, david) ←
    parent(alice, bob)     [fact]
    ancestor(bob, david) ←
      parent(bob, david)   [fact]
```

### Justification Generation

Proof trees can be rendered as human-readable explanations:

```
Bob is eligible for a bonus of $24,000 because:
  1. Bob is classified as Manager
     - Bob's salary ($120,000) >= $100,000
     - Bob's years of service (12) >= 5
  2. Manager bonus formula is salary / 5
     - $120,000 / 5 = $24,000
```

### Negative Explanations

When a query fails, explain why:

```
Bob cannot access secret_document because:
  - Bob has roles: [Employee]
  - secret_document sensitivity: Secret
  - No applicable authorization rules:
    ✗ Not owner (owner is Alice)
    ✗ Not Admin role
    ✗ Not Public resource
    ✗ No delegation from owner
```

## Syntax Summary

```leem
// Type definitions
type TypeName = constructor(field: Type, ...)
type EnumName = | Variant1 | Variant2 | ...

// Relation declarations
rel relation_name: Type1 × Type2 × ... × TypeN

// Facts
relation_name(value1, value2, ...).

// Rules
head(Args) :- body1, body2, ..., bodyN.

// Queries (REPL or query files)
?- query_goal(Args).

// Comments
// Single line comment
/* Multi-line
   comment */
```

## Open Design Questions

### 1. Negation

How should we handle negation?
- **Stratified negation** (standard in Datalog): `not P` where P doesn't depend on not P
- **Negation as failure**: Traditional Prolog semantics
- **Explicit closed-world assumption**: Mark which relations are complete

**Proposal**: Start with stratified negation, add warnings for non-stratified use.

### 2. Aggregation

How to handle `count`, `sum`, `min`, `max`?

```leem
// Option 1: Built-in aggregate syntax
total_salary(Dept, Sum) :-
    sum(Sal : works_in(employee(_, _, Sal, _), Dept), Sum).

// Option 2: Special query mode
?- aggregate(sum, Sal, works_in(employee(_, _, Sal, _), dept("Eng", _)), Total).

// Option 3: Compile-time aggregate functions
rel total_salary: Department × Int
total_salary(Dept, Sum) :- aggregate_sum(employee_salary(Dept), Sum).
```

**Proposal**: Start with explicit aggregate predicates, add sugar later.

### 3. Mode Declarations

Should modes be explicit or inferred?

```leem
// Explicit modes (Mercury style)
:- mode ancestor(in, out).
:- mode ancestor(in, in).

// vs inferred from usage
```

**Proposal**: Infer modes initially, add explicit annotations for optimization hints.

### 4. Lists and Recursion

How should list operations work?

```leem
// Pattern matching on lists
rel length: List<T> × Int
length([], 0).
length([H | T], N) :- length(T, N1), N = N1 + 1.

// Or built-in list operations?
rel length: List<T> × Int
length(L, N) :- N = builtin_length(L).
```

**Proposal**: Support pattern matching on lists, provide built-ins for common operations.

### 5. Module System

How to organize large programs?

```leem
module authorization {
    export can_access/3
    export has_role/2

    import users
    import resources

    // definitions...
}
```

**Proposal**: Design module system after core language stabilizes.

## Compilation Pipeline

```
Source (.leem)
    ↓
  Lexer/Parser
    ↓
  AST
    ↓
  Type Checker
    ↓
  Typed AST
    ↓
  Mode Analysis
    ↓
  IR Generation (leem IR)
    ↓
  Optimization
    ↓
  LLVM IR Generation
    ↓
  LLVM Optimization
    ↓
  Native Code
```

### Intermediate Representation

leem will have its own IR before LLVM:
- Explicit unification operations
- Tabled predicates marked
- Mode information attached
- Proof tree construction embedded

This IR is then lowered to LLVM with:
- Specialized code per mode
- Arena allocation calls
- Proof tracking (optional, for explainability)

## Example Compilation

See `LLVM_IR.md` for a worked example of compiling a simple rule to LLVM IR.

## Tooling

### REPL

Interactive query interface:
```
leem> ?- parent(alice, X).
X = bob
X = carol

leem> :explain
parent(alice, bob) [fact at family.leem:12]
```

### Compiler

```bash
leemc compile program.leem -o program
leemc check program.leem           # Type check only
leemc explain program.leem query   # Show proof tree for query
```

### Integration

```rust
// Embed leem in Rust programs (future)
use leem::Program;

let prog = Program::load("rules.leem")?;
let result = prog.query("can_access(user1, resource1, Read)")?;
if result.success {
    println!("Allowed: {}", result.explain());
} else {
    println!("Denied: {}", result.explain());
}
```

## Comparison to Other Languages

| Feature | Prolog | Datalog | Mercury | leem |
|---------|--------|---------|---------|------|
| Type System | Dynamic | Dynamic | Static (complex) | Static (ADTs) |
| Compilation | Interpreted/WAM | Interpreted | Native | Native (LLVM) |
| Explainability | Limited | Limited | None | Built-in |
| Negation | NAF | Stratified | Stratified | Stratified |
| Memory | GC | Varies | GC | Arena (no GC) |
| Target Use Case | General | Databases | General | Rules engines |

## Next Steps

1. **Implement minimal parser** - Parse type defs, facts, simple rules
2. **Type checker** - Infer types, check relation usage
3. **Simple evaluator** - Interpret rules (before LLVM)
4. **Proof tree generation** - Track derivations
5. **LLVM backend** - Compile simplest cases
6. **Iterate and expand**

## References

- Prolog: ISO Prolog standard
- Datalog: "What You Always Wanted to Know About Datalog"
- Mercury: mercury-lang.org
- XSB Prolog: Tabled evaluation
- Formulog: Datalog + ML types + SMT
