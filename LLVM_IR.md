# LLVM IR Compilation Example

This document shows how a simple leem program compiles to LLVM IR.

## Source Program

```leem
type Person = person(name: String, age: Int)

rel parent: Person × Person

parent(person("Alice", 45), person("Bob", 20)).
parent(person("Bob", 20), person("Charlie", 2)).

ancestor(X, Y) :- parent(X, Y).
ancestor(X, Z) :- parent(X, Y), ancestor(Y, Z).

?- ancestor(person("Alice", 45), Q).
```

## Compilation Strategy

### 1. Data Layout

First, define how Person is laid out in memory:

```llvm
; Person is a struct with string pointer and i64 age
%Person = type { ptr, i64 }

; String is a pointer to bytes + length (fat pointer)
%String = type { ptr, i64 }
```

### 2. Fact Tables

Facts are compiled to static data tables:

```llvm
; String constants for names
@str.alice = private constant [6 x i8] c"Alice\00"
@str.bob = private constant [4 x i8] c"Bob\00"
@str.charlie = private constant [8 x i8] c"Charlie\00"

; String structs
@string.alice = private constant %String { ptr @str.alice, i64 5 }
@string.bob = private constant %String { ptr @str.bob, i64 3 }
@string.charlie = private constant %String { ptr @str.charlie, i64 7 }

; Person structs
@person.alice = private constant %Person { ptr @string.alice, i64 45 }
@person.bob = private constant %Person { ptr @string.bob, i64 20 }
@person.charlie = private constant %Person { ptr @string.charlie, i64 2 }

; Fact table for parent/2
; Array of pairs of Person pointers
@parent_facts = private constant [2 x [2 x ptr]] [
  [2 x ptr] [ptr @person.alice, ptr @person.bob],
  [2 x ptr] [ptr @person.bob, ptr @person.charlie]
]
@parent_facts_count = private constant i64 2
```

### 3. Query Context and Arena

```llvm
; Query arena for allocations during evaluation
%Arena = type { ptr, i64, i64 }  ; data, size, capacity

; Proof tree node for explainability
%ProofNode = type {
  i32,           ; rule_id
  ptr,           ; relation_name
  ptr,           ; arguments array
  i32,           ; arg_count
  ptr,           ; children array
  i32            ; children_count
}

; Query context holds arena and proof tracking
%QueryContext = type {
  %Arena,        ; memory arena
  ptr,           ; proof_root (ProofNode*)
  i1             ; track_proofs (bool)
}
```

### 4. Compiled Relations

Each relation compiles to a function. We compile different versions based on modes.

#### parent/2 - Mode: parent(in, out)

```llvm
; Returns true if parent fact exists, sets output via pointer
; parent_in_out(context, parent_person, child_out) -> bool
define i1 @parent_in_out(ptr %ctx, ptr %parent, ptr %child_out) {
entry:
  ; Iterate through fact table
  %facts = load ptr, ptr @parent_facts
  %count = load i64, ptr @parent_facts_count
  br label %loop

loop:
  %i = phi i64 [ 0, %entry ], [ %i.next, %loop.continue ]
  %done = icmp uge i64 %i, %count
  br i1 %done, label %not_found, label %check_fact

check_fact:
  ; Get fact pair
  %fact_ptr = getelementptr [2 x ptr], ptr %facts, i64 %i
  %fact_parent_ptr = getelementptr [2 x ptr], ptr %fact_ptr, i32 0, i32 0
  %fact_child_ptr = getelementptr [2 x ptr], ptr %fact_ptr, i32 0, i32 1

  %fact_parent = load ptr, ptr %fact_parent_ptr
  %fact_child = load ptr, ptr %fact_child_ptr

  ; Compare parent (call person_eq)
  %match = call i1 @person_eq(ptr %parent, ptr %fact_parent)
  br i1 %match, label %found, label %loop.continue

loop.continue:
  %i.next = add i64 %i, 1
  br label %loop

found:
  ; Unify child_out with fact_child
  %fact_child_reload = load ptr, ptr %fact_child_ptr
  store ptr %fact_child_reload, ptr %child_out

  ; Record proof if tracking
  %tracking = getelementptr %QueryContext, ptr %ctx, i32 0, i32 2
  %track = load i1, ptr %tracking
  br i1 %track, label %record_proof, label %return_true

record_proof:
  ; TODO: Add proof node for this fact
  br label %return_true

return_true:
  ret i1 true

not_found:
  ret i1 false
}
```

#### ancestor/2 - Mode: ancestor(in, out)

This is more complex because it's recursive. We use tabling to avoid infinite loops.

```llvm
; Table entry for memoization
%TableEntry = type {
  ptr,           ; input (Person*)
  ptr,           ; output (Person*)
  ptr            ; next (TableEntry*)
}

; Global table for ancestor/2
@ancestor_table = global ptr null
@ancestor_table_lock = global i32 0  ; Simple spinlock

; ancestor_in_out with tabling
define i1 @ancestor_in_out(ptr %ctx, ptr %x, ptr %y_out) {
entry:
  ; Check table first
  %cached = call ptr @table_lookup(ptr @ancestor_table, ptr %x)
  %is_cached = icmp ne ptr %cached, null
  br i1 %is_cached, label %return_cached, label %compute

return_cached:
  store ptr %cached, ptr %y_out
  ret i1 true

compute:
  ; Base case: parent(X, Y)
  %parent_result = call i1 @parent_in_out(ptr %ctx, ptr %x, ptr %y_out)
  br i1 %parent_result, label %record_and_return, label %recursive_case

recursive_case:
  ; Recursive case: parent(X, Mid), ancestor(Mid, Y)
  %arena_ptr = getelementptr %QueryContext, ptr %ctx, i32 0, i32 0
  %mid_ptr = call ptr @arena_alloc(ptr %arena_ptr, i64 16)  ; sizeof(Person)

  %parent_mid = call i1 @parent_in_out(ptr %ctx, ptr %x, ptr %mid_ptr)
  br i1 %parent_mid, label %check_ancestor, label %fail

check_ancestor:
  %mid = load ptr, ptr %mid_ptr
  %ancestor_result = call i1 @ancestor_in_out(ptr %ctx, ptr %mid, ptr %y_out)
  br i1 %ancestor_result, label %record_and_return, label %fail

record_and_return:
  %y = load ptr, ptr %y_out
  call void @table_insert(ptr @ancestor_table, ptr %x, ptr %y)
  ret i1 true

fail:
  ret i1 false
}
```

### 5. Helper Functions

```llvm
; Compare two Person structs for equality
define i1 @person_eq(ptr %p1, ptr %p2) {
entry:
  ; Compare name strings
  %name1_ptr = getelementptr %Person, ptr %p1, i32 0, i32 0
  %name2_ptr = getelementptr %Person, ptr %p2, i32 0, i32 0
  %name1 = load ptr, ptr %name1_ptr
  %name2 = load ptr, ptr %name2_ptr
  %names_eq = call i1 @string_eq(ptr %name1, ptr %name2)
  br i1 %names_eq, label %check_age, label %not_equal

check_age:
  %age1_ptr = getelementptr %Person, ptr %p1, i32 0, i32 1
  %age2_ptr = getelementptr %Person, ptr %p2, i32 0, i32 1
  %age1 = load i64, ptr %age1_ptr
  %age2 = load i64, ptr %age2_ptr
  %ages_eq = icmp eq i64 %age1, %age2
  ret i1 %ages_eq

not_equal:
  ret i1 false
}

; String equality
define i1 @string_eq(ptr %s1, ptr %s2) {
entry:
  ; Compare lengths
  %len1_ptr = getelementptr %String, ptr %s1, i32 0, i32 1
  %len2_ptr = getelementptr %String, ptr %s2, i32 0, i32 1
  %len1 = load i64, ptr %len1_ptr
  %len2 = load i64, ptr %len2_ptr
  %lens_eq = icmp eq i64 %len1, %len2
  br i1 %lens_eq, label %compare_bytes, label %not_equal

compare_bytes:
  %data1_ptr = getelementptr %String, ptr %s1, i32 0, i32 0
  %data2_ptr = getelementptr %String, ptr %s2, i32 0, i32 0
  %data1 = load ptr, ptr %data1_ptr
  %data2 = load ptr, ptr %data2_ptr
  %result = call i32 @memcmp(ptr %data1, ptr %data2, i64 %len1)
  %is_eq = icmp eq i32 %result, 0
  ret i1 %is_eq

not_equal:
  ret i1 false
}

; External memcmp
declare i32 @memcmp(ptr, ptr, i64)

; Arena allocation
define ptr @arena_alloc(ptr %arena, i64 %size) {
  ; Simple bump allocator
  ; TODO: Full implementation
  ret ptr null
}

; Table operations
define ptr @table_lookup(ptr %table, ptr %key) {
  ; Hash table lookup
  ; TODO: Full implementation
  ret ptr null
}

define void @table_insert(ptr %table, ptr %key, ptr %value) {
  ; Hash table insert
  ; TODO: Full implementation
  ret void
}
```

### 6. Query Entry Point

```llvm
define i32 @main() {
entry:
  ; Initialize query context
  %ctx = alloca %QueryContext
  %arena_ptr = getelementptr %QueryContext, ptr %ctx, i32 0, i32 0
  call void @arena_init(ptr %arena_ptr, i64 4096)

  ; Set proof tracking
  %track_ptr = getelementptr %QueryContext, ptr %ctx, i32 0, i32 2
  store i1 true, ptr %track_ptr

  ; Query: ancestor(person("Alice", 45), Q)
  %alice = load ptr, ptr @person.alice
  %result_ptr = alloca ptr

  ; Call ancestor
  %success = call i1 @ancestor_in_out(ptr %ctx, ptr %alice, ptr %result_ptr)

  br i1 %success, label %print_result, label %no_solution

print_result:
  %result = load ptr, ptr %result_ptr
  call void @print_person(ptr %result)

  ; Clean up
  call void @arena_destroy(ptr %arena_ptr)
  ret i32 0

no_solution:
  call void @arena_destroy(ptr %arena_ptr)
  ret i32 1
}

declare void @arena_init(ptr, i64)
declare void @arena_destroy(ptr)
declare void @print_person(ptr)
```

## Key Observations

1. **Facts as static data**: No need for heap allocation for ground facts
2. **Specialized functions**: Different modes compile to different code paths
3. **Tabling for recursion**: Memoization prevents infinite loops
4. **Arena allocation**: Query-scoped memory, no GC needed
5. **Proof tracking**: Optional, can be compiled out for production
6. **Type-driven layout**: Person struct is concrete, enables optimization

## Optimizations

LLVM can apply:
- **Inlining**: Small predicates inline into callers
- **Constant propagation**: When queries have ground terms
- **Dead code elimination**: Unused proof tracking removed
- **Loop optimization**: Fact table iteration
- **Devirtualization**: Type information enables direct calls

## Next: More Complex Features

Future documents will show:
- Pattern matching compilation
- Expression evaluation
- Arithmetic constraints
- List operations
- Aggregation
- Stratified negation
