/// Intermediate Representation for qed
///
/// This module defines the IR between the type-checked AST and LLVM IR.
/// The IR makes explicit:
/// - Unification operations
/// - Tabling/memoization points
/// - Mode information (input/output patterns)
/// - Memory allocation sites
use crate::ast::Type;
use std::collections::HashMap;

/// A compiled qed program in IR form
#[derive(Debug, Clone)]
pub struct IrProgram {
    pub types: Vec<IrTypeDef>,
    pub relations: Vec<IrRelation>,
    pub queries: Vec<IrQuery>,
}

/// IR type definition with memory layout info
#[derive(Debug, Clone)]
pub struct IrTypeDef {
    pub name: String,
    pub layout: TypeLayout,
}

#[derive(Debug, Clone)]
pub enum TypeLayout {
    /// Product type with known fields
    Struct {
        fields: Vec<(String, Type)>,
        size_bytes: usize,
        align_bytes: usize,
    },
    /// Sum type (tagged union)
    Enum {
        variants: Vec<String>,
        tag_size: usize,
        max_variant_size: usize,
    },
}

/// IR relation with compiled implementations for each mode
#[derive(Debug, Clone)]
pub struct IrRelation {
    pub name: String,
    pub signature: Type,
    pub facts: Vec<IrFact>,
    pub rules: Vec<IrRule>,
    pub modes: Vec<IrMode>,
}

/// Mode specifies input/output pattern for a relation
#[derive(Debug, Clone)]
pub struct IrMode {
    pub pattern: Vec<ModeAnnotation>,
    pub implementation: IrPredicate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeAnnotation {
    Input,  // Ground at call time
    Output, // Computed by predicate
}

/// A fact in IR form (ground terms only)
#[derive(Debug, Clone)]
pub struct IrFact {
    pub args: Vec<IrValue>,
}

/// Rule in IR form with explicit unification
#[derive(Debug, Clone)]
pub struct IrRule {
    pub head: IrAtom,
    pub body: Vec<IrGoal>,
    pub needs_tabling: bool,
}

#[derive(Debug, Clone)]
pub struct IrAtom {
    pub relation: String,
    pub args: Vec<IrTerm>,
}

#[derive(Debug, Clone)]
pub enum IrGoal {
    /// Call a relation with mode
    Call {
        relation: String,
        mode_index: usize,
        args: Vec<IrTerm>,
    },
    /// Unify two terms
    Unify { left: IrTerm, right: IrTerm },
    /// Comparison
    Compare {
        op: CompareOp,
        left: IrTerm,
        right: IrTerm,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum CompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

/// Terms in IR
#[derive(Debug, Clone)]
pub enum IrTerm {
    /// Variable (SSA style - each var assigned once)
    Var(VarId),
    /// Constant value
    Value(IrValue),
    /// Field access
    FieldAccess { base: Box<IrTerm>, field: String },
    /// Binary operation
    BinOp {
        op: BinOp,
        left: Box<IrTerm>,
        right: Box<IrTerm>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VarId(pub usize);

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

/// Ground values in IR
#[derive(Debug, Clone)]
pub enum IrValue {
    Int(i64),
    String(String),
    Struct {
        type_name: String,
        fields: Vec<IrValue>,
    },
    Variant {
        type_name: String,
        variant: String,
    },
}

/// Compiled predicate implementation
#[derive(Debug, Clone)]
pub struct IrPredicate {
    pub params: Vec<IrParam>,
    pub body: Vec<IrInstruction>,
}

#[derive(Debug, Clone)]
pub struct IrParam {
    pub var: VarId,
    pub ty: Type,
    pub mode: ModeAnnotation,
}

/// Low-level IR instructions
#[derive(Debug, Clone)]
pub enum IrInstruction {
    /// Allocate in query arena
    Alloc { dest: VarId, ty: Type },
    /// Load from memory
    Load {
        dest: VarId,
        src: VarId,
        offset: usize,
    },
    /// Store to memory
    Store {
        dest: VarId,
        src: VarId,
        offset: usize,
    },
    /// Unify two variables
    Unify { left: VarId, right: VarId },
    /// Call relation
    Call {
        dest: Option<VarId>,
        relation: String,
        mode: usize,
        args: Vec<VarId>,
    },
    /// Table lookup
    TableLookup {
        dest: VarId,
        table: String,
        key: VarId,
    },
    /// Table insert
    TableInsert {
        table: String,
        key: VarId,
        value: VarId,
    },
    /// Conditional branch
    Branch {
        cond: VarId,
        true_block: BlockId,
        false_block: BlockId,
    },
    /// Return from predicate
    Return { value: Option<VarId> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);

/// Query to execute
#[derive(Debug, Clone)]
pub struct IrQuery {
    pub goals: Vec<IrGoal>,
}

/// IR Lowering context
pub struct IrLowering {
    var_counter: usize,
    relations: HashMap<String, IrRelation>,
}

impl IrLowering {
    pub fn new() -> Self {
        IrLowering {
            var_counter: 0,
            relations: HashMap::new(),
        }
    }

    fn fresh_var(&mut self) -> VarId {
        let id = VarId(self.var_counter);
        self.var_counter += 1;
        id
    }

    /// Lower AST to IR
    pub fn lower(&mut self, program: &crate::ast::Program) -> IrProgram {
        let mut types = Vec::new();
        let mut queries = Vec::new();

        // First pass: collect type definitions
        for item in &program.items {
            if let crate::ast::Item::TypeDef(typedef) = item {
                types.push(self.lower_type_def(typedef));
            }
        }

        // Second pass: initialize relations from declarations
        for item in &program.items {
            if let crate::ast::Item::RelationDecl(rel) = item {
                self.relations.insert(
                    rel.name.clone(),
                    IrRelation {
                        name: rel.name.clone(),
                        signature: rel.signature.clone(),
                        facts: vec![],
                        rules: vec![],
                        modes: vec![],
                    },
                );
            }
        }

        // Third pass: collect facts and rules
        for item in &program.items {
            match item {
                crate::ast::Item::Fact(fact) => {
                    let lowered = self.lower_fact(fact);
                    if let Some(rel) = self.relations.get_mut(&fact.relation) {
                        rel.facts.push(lowered);
                    }
                }
                crate::ast::Item::Rule(rule) => {
                    let lowered = self.lower_rule(rule);
                    if let Some(rel) = self.relations.get_mut(&rule.head.relation) {
                        rel.rules.push(lowered);
                    }
                }
                _ => {}
            }
        }

        // Fourth pass: collect queries
        for item in &program.items {
            if let crate::ast::Item::Query(query) = item {
                queries.push(self.lower_query(query));
            }
        }

        IrProgram {
            types,
            relations: self.relations.values().cloned().collect(),
            queries,
        }
    }

    fn lower_type_def(&self, typedef: &crate::ast::TypeDef) -> IrTypeDef {
        let layout = match &typedef.def {
            crate::ast::TypeDefKind::Product {
                constructor: _,
                fields,
            } => {
                // Calculate struct layout (simplified - assuming 8-byte alignment)
                let mut size = 0usize;
                let field_types: Vec<_> = fields
                    .iter()
                    .map(|f| {
                        let field_size = Self::type_size(&f.ty);
                        size += field_size;
                        (f.name.clone(), f.ty.clone())
                    })
                    .collect();

                TypeLayout::Struct {
                    fields: field_types,
                    size_bytes: size,
                    align_bytes: 8,
                }
            }
            crate::ast::TypeDefKind::Sum { variants } => TypeLayout::Enum {
                variants: variants.clone(),
                tag_size: 8, // i64 tag
                max_variant_size: 0,
            },
        };

        IrTypeDef {
            name: typedef.name.clone(),
            layout,
        }
    }

    fn type_size(ty: &crate::ast::Type) -> usize {
        match ty {
            crate::ast::Type::Named(name) => match name.as_str() {
                "Int" => 8,
                "String" => 16, // ptr + len
                _ => 8,         // pointer to struct
            },
            crate::ast::Type::List(_) => 16,   // ptr + len
            crate::ast::Type::Option(_) => 16, // tag + value
            crate::ast::Type::Product(types) => types.iter().map(Self::type_size).sum(),
        }
    }

    fn lower_fact(&self, fact: &crate::ast::Fact) -> IrFact {
        IrFact {
            args: fact.args.iter().map(Self::lower_term_to_value).collect(),
        }
    }

    fn lower_term_to_value(term: &crate::ast::Term) -> IrValue {
        match term {
            crate::ast::Term::Int(n, _) => IrValue::Int(*n),
            crate::ast::Term::String(s, _) => IrValue::String(s.clone()),
            crate::ast::Term::Construct {
                constructor, args, ..
            } => IrValue::Struct {
                type_name: constructor.clone(),
                fields: args.iter().map(Self::lower_term_to_value).collect(),
            },
            crate::ast::Term::Var(_, _) => {
                panic!("Variables should not appear in facts")
            }
            crate::ast::Term::BinOp { .. } => {
                panic!("Binary ops in facts should be pre-evaluated")
            }
        }
    }

    fn lower_rule(&mut self, rule: &crate::ast::Rule) -> IrRule {
        // Check if this rule needs tabling (recursive)
        let needs_tabling = rule
            .body
            .iter()
            .any(|g| matches!(g, crate::ast::Goal::Atom(a) if a.relation == rule.head.relation));

        IrRule {
            head: self.lower_atom(&rule.head),
            body: rule.body.iter().map(|g| self.lower_goal(g)).collect(),
            needs_tabling,
        }
    }

    fn lower_atom(&mut self, atom: &crate::ast::Atom) -> IrAtom {
        IrAtom {
            relation: atom.relation.clone(),
            args: atom.args.iter().map(|t| self.lower_term(t)).collect(),
        }
    }

    fn lower_term(&mut self, term: &crate::ast::Term) -> IrTerm {
        match term {
            crate::ast::Term::Var(name, _) => {
                // For now, create a fresh var - in real impl would track var mappings
                IrTerm::Var(self.fresh_var())
            }
            crate::ast::Term::Int(n, _) => IrTerm::Value(IrValue::Int(*n)),
            crate::ast::Term::String(s, _) => IrTerm::Value(IrValue::String(s.clone())),
            crate::ast::Term::Construct {
                constructor, args, ..
            } => IrTerm::Value(IrValue::Struct {
                type_name: constructor.clone(),
                fields: args.iter().map(Self::lower_term_to_value).collect(),
            }),
            crate::ast::Term::BinOp {
                op, left, right, ..
            } => IrTerm::BinOp {
                op: self.lower_binop(*op),
                left: Box::new(self.lower_term(left)),
                right: Box::new(self.lower_term(right)),
            },
        }
    }

    fn lower_binop(&self, op: crate::ast::BinOp) -> BinOp {
        match op {
            crate::ast::BinOp::Add => BinOp::Add,
            crate::ast::BinOp::Sub => BinOp::Sub,
            crate::ast::BinOp::Mul => BinOp::Mul,
            crate::ast::BinOp::Div => BinOp::Div,
            crate::ast::BinOp::Mod => BinOp::Mod,
        }
    }

    fn lower_goal(&mut self, goal: &crate::ast::Goal) -> IrGoal {
        match goal {
            crate::ast::Goal::Atom(atom) => IrGoal::Call {
                relation: atom.relation.clone(),
                mode_index: 0, // Default mode - will be refined later
                args: atom.args.iter().map(|t| self.lower_term(t)).collect(),
            },
            crate::ast::Goal::Unify(left, right, _) => IrGoal::Unify {
                left: self.lower_term(left),
                right: self.lower_term(right),
            },
            crate::ast::Goal::Compare(op, left, right, _) => IrGoal::Compare {
                op: self.lower_compare_op(*op),
                left: self.lower_term(left),
                right: self.lower_term(right),
            },
        }
    }

    fn lower_compare_op(&self, op: crate::ast::CompareOp) -> CompareOp {
        match op {
            crate::ast::CompareOp::Eq => CompareOp::Eq,
            crate::ast::CompareOp::Ne => CompareOp::Ne,
            crate::ast::CompareOp::Lt => CompareOp::Lt,
            crate::ast::CompareOp::Le => CompareOp::Le,
            crate::ast::CompareOp::Gt => CompareOp::Gt,
            crate::ast::CompareOp::Ge => CompareOp::Ge,
        }
    }

    fn lower_query(&mut self, query: &crate::ast::Query) -> IrQuery {
        IrQuery {
            goals: query.goals.iter().map(|g| self.lower_goal(g)).collect(),
        }
    }
}

impl Default for IrLowering {
    fn default() -> Self {
        Self::new()
    }
}

/// Lower AST to IR (convenience function)
pub fn lower_to_ir(program: &crate::ast::Program) -> IrProgram {
    let mut lowering = IrLowering::new();
    lowering.lower(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    #[test]
    fn test_ir_creation() {
        let ir = IrProgram {
            types: vec![],
            relations: vec![],
            queries: vec![],
        };
        assert_eq!(ir.types.len(), 0);
    }

    #[test]
    fn test_lower_type_def() {
        let source = r#"
            type Person = person(name: String, age: Int)
        "#;
        let program = parser::parse(source).expect("Parse failed");
        let ir = lower_to_ir(&program);

        assert_eq!(ir.types.len(), 1);
        assert_eq!(ir.types[0].name, "Person");
        match &ir.types[0].layout {
            TypeLayout::Struct { fields, .. } => {
                assert_eq!(fields.len(), 2);
            }
            _ => panic!("Expected struct layout"),
        }
    }

    #[test]
    fn test_lower_fact() {
        let source = r#"
            type Person = person(name: String, age: Int)
            rel parent: Person × Person
            parent(person("Alice", 45), person("Bob", 20)).
        "#;
        let program = parser::parse(source).expect("Parse failed");
        let ir = lower_to_ir(&program);

        assert_eq!(ir.relations.len(), 1);
        let parent_rel = &ir.relations[0];
        assert_eq!(parent_rel.name, "parent");
        assert_eq!(parent_rel.facts.len(), 1);
        assert_eq!(parent_rel.facts[0].args.len(), 2);
    }

    #[test]
    fn test_lower_rule() {
        let source = r#"
            type Person = person(name: String, age: Int)
            rel parent: Person × Person
            rel ancestor: Person × Person
            ancestor(X, Y) :- parent(X, Y).
        "#;
        let program = parser::parse(source).expect("Parse failed");
        let ir = lower_to_ir(&program);

        let ancestor_rel = ir.relations.iter().find(|r| r.name == "ancestor").unwrap();
        assert_eq!(ancestor_rel.rules.len(), 1);
        assert!(!ancestor_rel.rules[0].needs_tabling);
    }

    #[test]
    fn test_lower_recursive_rule() {
        let source = r#"
            type Person = person(name: String, age: Int)
            rel parent: Person × Person
            rel ancestor: Person × Person
            ancestor(X, Z) :- parent(X, Y), ancestor(Y, Z).
        "#;
        let program = parser::parse(source).expect("Parse failed");
        let ir = lower_to_ir(&program);

        let ancestor_rel = ir.relations.iter().find(|r| r.name == "ancestor").unwrap();
        assert_eq!(ancestor_rel.rules.len(), 1);
        assert!(ancestor_rel.rules[0].needs_tabling); // Recursive!
    }

    #[test]
    fn test_lower_query() {
        let source = r#"
            type Person = person(name: String, age: Int)
            rel ancestor: Person × Person
            ?- ancestor(X, Y).
        "#;
        let program = parser::parse(source).expect("Parse failed");
        let ir = lower_to_ir(&program);

        assert_eq!(ir.queries.len(), 1);
        assert_eq!(ir.queries[0].goals.len(), 1);
    }
}
