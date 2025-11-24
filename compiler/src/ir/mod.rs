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

/// Lower AST to IR
pub fn lower_to_ir(program: &crate::ast::Program) -> IrProgram {
    // TODO: Implement AST -> IR lowering
    IrProgram {
        types: vec![],
        relations: vec![],
        queries: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ir_creation() {
        let ir = IrProgram {
            types: vec![],
            relations: vec![],
            queries: vec![],
        };
        assert_eq!(ir.types.len(), 0);
    }
}
