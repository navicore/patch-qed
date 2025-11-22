/// Type checking and inference for leem
///
/// This module implements:
/// - Type environment management
/// - Type inference for terms and goals
/// - Relation signature checking
/// - Mode analysis (input/output pattern detection)

use crate::ast::*;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// Type environment tracks type definitions and relation signatures
#[derive(Debug, Clone)]
pub struct TypeEnv {
    /// Type definitions: Person -> TypeDef
    pub types: HashMap<String, TypeDefKind>,

    /// Relation signatures: parent -> Person × Person
    pub relations: HashMap<String, Type>,

    /// Constructor signatures: person -> (String, Int) -> Person
    pub constructors: HashMap<String, (Vec<Type>, String)>,
}

impl TypeEnv {
    pub fn new() -> Self {
        let mut env = TypeEnv {
            types: HashMap::new(),
            relations: HashMap::new(),
            constructors: HashMap::new(),
        };

        // Add built-in types
        env.add_builtin_types();
        env
    }

    fn add_builtin_types(&mut self) {
        // Built-in types: Int, String, Bool
        // These don't need definitions but should be recognized
    }

    pub fn add_type(&mut self, name: String, def: TypeDefKind) -> Result<()> {
        if self.types.contains_key(&name) {
            return Err(anyhow!("Type {} already defined", name));
        }

        // Extract constructor info
        match &def {
            TypeDefKind::Product { constructor, fields } => {
                let field_types: Vec<_> = fields.iter().map(|f| f.ty.clone()).collect();
                self.constructors.insert(
                    constructor.clone(),
                    (field_types, name.clone()),
                );
            }
            TypeDefKind::Sum { variants } => {
                for variant in variants {
                    self.constructors.insert(
                        variant.clone(),
                        (vec![], name.clone()),
                    );
                }
            }
        }

        self.types.insert(name, def);
        Ok(())
    }

    pub fn add_relation(&mut self, name: String, signature: Type) -> Result<()> {
        if self.relations.contains_key(&name) {
            return Err(anyhow!("Relation {} already defined", name));
        }
        self.relations.insert(name, signature);
        Ok(())
    }

    pub fn get_relation_signature(&self, name: &str) -> Option<&Type> {
        self.relations.get(name)
    }

    pub fn get_constructor_info(&self, name: &str) -> Option<&(Vec<Type>, String)> {
        self.constructors.get(name)
    }
}

/// Type checker for leem programs
pub struct TypeChecker {
    env: TypeEnv,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            env: TypeEnv::new(),
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Result<()> {
        // First pass: collect type and relation definitions
        for item in &program.items {
            match item {
                Item::TypeDef(typedef) => {
                    self.env.add_type(typedef.name.clone(), typedef.def.clone())?;
                }
                Item::RelationDecl(rel) => {
                    self.env.add_relation(rel.name.clone(), rel.signature.clone())?;
                }
                _ => {}
            }
        }

        // Second pass: type check facts and rules
        for item in &program.items {
            match item {
                Item::Fact(fact) => self.check_fact(fact)?,
                Item::Rule(rule) => self.check_rule(rule)?,
                Item::Query(query) => self.check_query(query)?,
                _ => {}
            }
        }

        Ok(())
    }

    fn check_fact(&self, fact: &Fact) -> Result<()> {
        // TODO: Implement fact type checking
        // 1. Look up relation signature
        // 2. Check each argument matches expected type
        // 3. All terms must be ground (no variables)
        Ok(())
    }

    fn check_rule(&self, rule: &Rule) -> Result<()> {
        // TODO: Implement rule type checking
        // 1. Check head atom against relation signature
        // 2. Build variable type environment from head
        // 3. Check each goal in body
        // 4. Ensure all variables in head appear in body (safety)
        Ok(())
    }

    fn check_query(&self, query: &Query) -> Result<()> {
        // TODO: Implement query type checking
        Ok(())
    }

    fn infer_term_type(&self, term: &Term, var_env: &HashMap<String, Type>) -> Result<Type> {
        // TODO: Implement type inference for terms
        match term {
            Term::Var(name, _) => {
                var_env.get(name)
                    .cloned()
                    .ok_or_else(|| anyhow!("Unbound variable: {}", name))
            }
            Term::Int(_, _) => Ok(Type::Named("Int".to_string())),
            Term::String(_, _) => Ok(Type::Named("String".to_string())),
            Term::Construct { constructor, args, .. } => {
                // Look up constructor, infer result type
                todo!("Constructor type inference")
            }
            Term::BinOp { .. } => {
                // Binary ops currently only on Int
                Ok(Type::Named("Int".to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_env_creation() {
        let env = TypeEnv::new();
        assert_eq!(env.types.len(), 0);
    }

    #[test]
    fn test_add_type() {
        let mut env = TypeEnv::new();
        let def = TypeDefKind::Product {
            constructor: "person".to_string(),
            fields: vec![
                Field {
                    name: "name".to_string(),
                    ty: Type::Named("String".to_string()),
                },
                Field {
                    name: "age".to_string(),
                    ty: Type::Named("Int".to_string()),
                },
            ],
        };
        assert!(env.add_type("Person".to_string(), def).is_ok());
    }
}
