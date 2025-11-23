/// Abstract Syntax Tree definitions for qed
///
/// This module defines the AST representation of qed programs after parsing.

use std::fmt;

/// Source location information for error reporting
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// A complete qed program
#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}

/// Top-level items in a qed program
#[derive(Debug, Clone)]
pub enum Item {
    TypeDef(TypeDef),
    RelationDecl(RelationDecl),
    Fact(Fact),
    Rule(Rule),
    Query(Query),
}

/// Type definition: type Person = person(name: String, age: Int)
#[derive(Debug, Clone)]
pub struct TypeDef {
    pub name: String,
    pub def: TypeDefKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TypeDefKind {
    /// Product type: person(name: String, age: Int)
    Product {
        constructor: String,
        fields: Vec<Field>,
    },
    /// Sum type: Level = Public | Internal | Confidential
    Sum { variants: Vec<String> },
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

/// Type expressions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// Named type: Person, Int, String
    Named(String),
    /// List type: List<T>
    List(Box<Type>),
    /// Option type: Option<T>
    Option(Box<Type>),
    /// Product type (for relation signatures): Person × Int
    Product(Vec<Type>),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Named(name) => write!(f, "{}", name),
            Type::List(inner) => write!(f, "List<{}>", inner),
            Type::Option(inner) => write!(f, "Option<{}>", inner),
            Type::Product(types) => {
                let type_strs: Vec<_> = types.iter().map(|t| t.to_string()).collect();
                write!(f, "{}", type_strs.join(" × "))
            }
        }
    }
}

/// Relation declaration: rel parent: Person × Person
#[derive(Debug, Clone)]
pub struct RelationDecl {
    pub name: String,
    pub signature: Type,
    pub span: Span,
}

/// Fact: parent(alice, bob).
#[derive(Debug, Clone)]
pub struct Fact {
    pub relation: String,
    pub args: Vec<Term>,
    pub span: Span,
}

/// Rule: ancestor(X, Z) :- parent(X, Y), ancestor(Y, Z).
#[derive(Debug, Clone)]
pub struct Rule {
    pub head: Atom,
    pub body: Vec<Goal>,
    pub span: Span,
}

/// An atom in a rule: parent(X, Y)
#[derive(Debug, Clone)]
pub struct Atom {
    pub relation: String,
    pub args: Vec<Term>,
    pub span: Span,
}

/// Goals in rule bodies
#[derive(Debug, Clone)]
pub enum Goal {
    /// Relation call: parent(X, Y)
    Atom(Atom),
    /// Unification: X = Y
    Unify(Term, Term, Span),
    /// Comparison: X < Y
    Compare(CompareOp, Term, Term, Span),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Eq,    // ==
    Ne,    // !=
    Lt,    // <
    Le,    // <=
    Gt,    // >
    Ge,    // >=
}

/// Terms that can appear in relations
#[derive(Debug, Clone)]
pub enum Term {
    /// Variable: X, Y, Age
    Var(String, Span),
    /// Integer literal: 42
    Int(i64, Span),
    /// String literal: "hello"
    String(String, Span),
    /// Constructor application: person("Alice", 45)
    Construct {
        constructor: String,
        args: Vec<Term>,
        span: Span,
    },
    /// Binary operation: X + Y
    BinOp {
        op: BinOp,
        left: Box<Term>,
        right: Box<Term>,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl Term {
    pub fn span(&self) -> &Span {
        match self {
            Term::Var(_, span) => span,
            Term::Int(_, span) => span,
            Term::String(_, span) => span,
            Term::Construct { span, .. } => span,
            Term::BinOp { span, .. } => span,
        }
    }
}

/// Query: ?- ancestor(alice, X).
#[derive(Debug, Clone)]
pub struct Query {
    pub goals: Vec<Goal>,
    pub span: Span,
}
