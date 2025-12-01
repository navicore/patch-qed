/// Type checking and inference for qed
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
            TypeDefKind::Product {
                constructor,
                fields,
            } => {
                let field_types: Vec<_> = fields.iter().map(|f| f.ty.clone()).collect();
                self.constructors
                    .insert(constructor.clone(), (field_types, name.clone()));
            }
            TypeDefKind::Sum { variants } => {
                for variant in variants {
                    self.constructors
                        .insert(variant.clone(), (vec![], name.clone()));
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

/// Type checker for qed programs
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
                    self.env
                        .add_type(typedef.name.clone(), typedef.def.clone())?;
                }
                Item::RelationDecl(rel) => {
                    self.env
                        .add_relation(rel.name.clone(), rel.signature.clone())?;
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
        // 1. Look up relation signature
        let expected_types = self.get_relation_arg_types(&fact.relation)?;

        // 2. Check arity
        if fact.args.len() != expected_types.len() {
            return Err(anyhow!(
                "Fact {} has {} arguments, expected {}",
                fact.relation,
                fact.args.len(),
                expected_types.len()
            ));
        }

        // 3. Check each argument is ground (no variables in facts)
        for (i, arg) in fact.args.iter().enumerate() {
            if !self.is_ground(arg) {
                return Err(anyhow!(
                    "Fact {} argument {} contains variables (facts must be ground)",
                    fact.relation,
                    i + 1
                ));
            }
        }

        // 4. Type check each argument
        let empty_env = HashMap::new();
        for (i, (arg, expected_ty)) in fact.args.iter().zip(expected_types.iter()).enumerate() {
            let actual_ty = self.infer_term_type(arg, &empty_env)?;
            if !self.types_compatible(&actual_ty, expected_ty) {
                return Err(anyhow!(
                    "Fact {} argument {} has type {}, expected {}",
                    fact.relation,
                    i + 1,
                    actual_ty,
                    expected_ty
                ));
            }
        }

        Ok(())
    }

    fn check_rule(&self, rule: &Rule) -> Result<()> {
        // 1. Get expected types for head relation
        let head_types = self.get_relation_arg_types(&rule.head.relation)?;

        if rule.head.args.len() != head_types.len() {
            return Err(anyhow!(
                "Rule head {} has {} arguments, expected {}",
                rule.head.relation,
                rule.head.args.len(),
                head_types.len()
            ));
        }

        // 2. Build variable type environment from head pattern
        let mut var_env: HashMap<String, Type> = HashMap::new();
        for (arg, expected_ty) in rule.head.args.iter().zip(head_types.iter()) {
            self.collect_var_types(arg, expected_ty, &mut var_env)?;
        }

        // 3. Check each goal in body and extend var_env
        for goal in &rule.body {
            self.check_goal(goal, &mut var_env)?;
        }

        // 4. Safety check: all variables in head must appear in body
        let head_vars = self.collect_vars_in_terms(&rule.head.args);
        let body_vars = self.collect_vars_in_goals(&rule.body);

        for var in &head_vars {
            if !body_vars.contains(var) {
                return Err(anyhow!(
                    "Unsafe rule: variable {} in head of {} does not appear in body",
                    var,
                    rule.head.relation
                ));
            }
        }

        Ok(())
    }

    fn check_query(&self, query: &Query) -> Result<()> {
        let mut var_env: HashMap<String, Type> = HashMap::new();
        for goal in &query.goals {
            self.check_goal(goal, &mut var_env)?;
        }
        Ok(())
    }

    /// Check a goal and update variable environment
    fn check_goal(&self, goal: &Goal, var_env: &mut HashMap<String, Type>) -> Result<()> {
        match goal {
            Goal::Atom(atom) => {
                let expected_types = self.get_relation_arg_types(&atom.relation)?;

                if atom.args.len() != expected_types.len() {
                    return Err(anyhow!(
                        "Goal {} has {} arguments, expected {}",
                        atom.relation,
                        atom.args.len(),
                        expected_types.len()
                    ));
                }

                // Check/infer types for each argument
                for (arg, expected_ty) in atom.args.iter().zip(expected_types.iter()) {
                    self.check_term_against_type(arg, expected_ty, var_env)?;
                }
            }
            Goal::Unify(left, right, _) => {
                // For unification, infer types of both sides
                // If one side has known type, propagate to other
                let left_ty = self.try_infer_term_type(left, var_env);
                let right_ty = self.try_infer_term_type(right, var_env);

                match (left_ty, right_ty) {
                    (Some(lt), Some(rt)) => {
                        if !self.types_compatible(&lt, &rt) {
                            return Err(anyhow!(
                                "Cannot unify {} with {} (type mismatch: {} vs {})",
                                self.term_to_string(left),
                                self.term_to_string(right),
                                lt,
                                rt
                            ));
                        }
                    }
                    (Some(ty), None) => {
                        self.collect_var_types(right, &ty, var_env)?;
                    }
                    (None, Some(ty)) => {
                        self.collect_var_types(left, &ty, var_env)?;
                    }
                    (None, None) => {
                        // Both sides are variables with unknown types - allowed for now
                    }
                }
            }
            Goal::Compare(op, left, right, _) => {
                // Comparisons require compatible types
                let int_ty = Type::Named("Int".to_string());

                // For now, assume comparisons are on Int
                self.check_term_against_type(left, &int_ty, var_env)?;
                self.check_term_against_type(right, &int_ty, var_env)?;
            }
        }
        Ok(())
    }

    /// Check a term against an expected type, updating var_env for any variables
    fn check_term_against_type(
        &self,
        term: &Term,
        expected_ty: &Type,
        var_env: &mut HashMap<String, Type>,
    ) -> Result<()> {
        match term {
            Term::Var(name, _) => {
                if let Some(existing_ty) = var_env.get(name) {
                    if !self.types_compatible(existing_ty, expected_ty) {
                        return Err(anyhow!(
                            "Variable {} has conflicting types: {} vs {}",
                            name,
                            existing_ty,
                            expected_ty
                        ));
                    }
                } else {
                    var_env.insert(name.clone(), expected_ty.clone());
                }
            }
            _ => {
                let actual_ty = self.infer_term_type(term, var_env)?;
                if !self.types_compatible(&actual_ty, expected_ty) {
                    return Err(anyhow!(
                        "Type mismatch: {} has type {}, expected {}",
                        self.term_to_string(term),
                        actual_ty,
                        expected_ty
                    ));
                }
            }
        }
        Ok(())
    }

    /// Try to infer term type, returning None if variables are unbound
    fn try_infer_term_type(&self, term: &Term, var_env: &HashMap<String, Type>) -> Option<Type> {
        match term {
            Term::Var(name, _) => var_env.get(name).cloned(),
            Term::Int(_, _) => Some(Type::Named("Int".to_string())),
            Term::String(_, _) => Some(Type::Named("String".to_string())),
            Term::Construct { constructor, .. } => self
                .env
                .get_constructor_info(constructor)
                .map(|(_, result_ty)| Type::Named(result_ty.clone())),
            Term::BinOp { .. } => Some(Type::Named("Int".to_string())),
        }
    }

    /// Collect variable types from a pattern
    fn collect_var_types(
        &self,
        term: &Term,
        expected_ty: &Type,
        var_env: &mut HashMap<String, Type>,
    ) -> Result<()> {
        match term {
            Term::Var(name, _) => {
                if let Some(existing_ty) = var_env.get(name) {
                    if !self.types_compatible(existing_ty, expected_ty) {
                        return Err(anyhow!(
                            "Variable {} has conflicting types: {} vs {}",
                            name,
                            existing_ty,
                            expected_ty
                        ));
                    }
                } else {
                    var_env.insert(name.clone(), expected_ty.clone());
                }
            }
            Term::Construct {
                constructor, args, ..
            } => {
                if let Some((param_types, _)) = self.env.get_constructor_info(constructor) {
                    for (arg, param_ty) in args.iter().zip(param_types.iter()) {
                        self.collect_var_types(arg, param_ty, var_env)?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Collect all variable names from terms
    fn collect_vars_in_terms(&self, terms: &[Term]) -> Vec<String> {
        let mut vars = Vec::new();
        for term in terms {
            Self::collect_vars_in_term(term, &mut vars);
        }
        vars
    }

    fn collect_vars_in_term(term: &Term, vars: &mut Vec<String>) {
        match term {
            Term::Var(name, _) => {
                if !vars.contains(name) {
                    vars.push(name.clone());
                }
            }
            Term::Construct { args, .. } => {
                for arg in args {
                    Self::collect_vars_in_term(arg, vars);
                }
            }
            Term::BinOp { left, right, .. } => {
                Self::collect_vars_in_term(left, vars);
                Self::collect_vars_in_term(right, vars);
            }
            _ => {}
        }
    }

    /// Collect all variable names from goals
    fn collect_vars_in_goals(&self, goals: &[Goal]) -> Vec<String> {
        let mut vars = Vec::new();
        for goal in goals {
            match goal {
                Goal::Atom(atom) => {
                    for arg in &atom.args {
                        Self::collect_vars_in_term(arg, &mut vars);
                    }
                }
                Goal::Unify(left, right, _) => {
                    Self::collect_vars_in_term(left, &mut vars);
                    Self::collect_vars_in_term(right, &mut vars);
                }
                Goal::Compare(_, left, right, _) => {
                    Self::collect_vars_in_term(left, &mut vars);
                    Self::collect_vars_in_term(right, &mut vars);
                }
            }
        }
        vars
    }

    /// Simple term to string for error messages
    fn term_to_string(&self, term: &Term) -> String {
        Self::format_term(term)
    }

    fn format_term(term: &Term) -> String {
        match term {
            Term::Var(name, _) => name.clone(),
            Term::Int(n, _) => n.to_string(),
            Term::String(s, _) => format!("\"{}\"", s),
            Term::Construct {
                constructor, args, ..
            } => {
                if args.is_empty() {
                    constructor.clone()
                } else {
                    format!(
                        "{}({})",
                        constructor,
                        args.iter()
                            .map(Self::format_term)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
            Term::BinOp {
                left, right, op, ..
            } => {
                format!(
                    "({} {:?} {})",
                    Self::format_term(left),
                    op,
                    Self::format_term(right)
                )
            }
        }
    }

    fn infer_term_type(&self, term: &Term, var_env: &HashMap<String, Type>) -> Result<Type> {
        match term {
            Term::Var(name, _) => var_env
                .get(name)
                .cloned()
                .ok_or_else(|| anyhow!("Unbound variable: {}", name)),
            Term::Int(_, _) => Ok(Type::Named("Int".to_string())),
            Term::String(_, _) => Ok(Type::Named("String".to_string())),
            Term::Construct {
                constructor,
                args,
                span,
            } => {
                // Look up constructor info
                let (param_types, result_type) = self
                    .env
                    .get_constructor_info(constructor)
                    .ok_or_else(|| anyhow!("Unknown constructor: {}", constructor))?;

                // Check arity
                if args.len() != param_types.len() {
                    return Err(anyhow!(
                        "Constructor {} expects {} arguments, got {}",
                        constructor,
                        param_types.len(),
                        args.len()
                    ));
                }

                // Check each argument type
                for (i, (arg, expected_ty)) in args.iter().zip(param_types.iter()).enumerate() {
                    let actual_ty = self.infer_term_type(arg, var_env)?;
                    if !self.types_compatible(&actual_ty, expected_ty) {
                        return Err(anyhow!(
                            "Argument {} of {} has type {}, expected {}",
                            i + 1,
                            constructor,
                            actual_ty,
                            expected_ty
                        ));
                    }
                }

                Ok(Type::Named(result_type.clone()))
            }
            Term::BinOp {
                op,
                left,
                right,
                span,
            } => {
                // Check both operands are Int
                let left_ty = self.infer_term_type(left, var_env)?;
                let right_ty = self.infer_term_type(right, var_env)?;

                let int_ty = Type::Named("Int".to_string());
                if !self.types_compatible(&left_ty, &int_ty) {
                    return Err(anyhow!(
                        "Left operand of {:?} has type {}, expected Int",
                        op,
                        left_ty
                    ));
                }
                if !self.types_compatible(&right_ty, &int_ty) {
                    return Err(anyhow!(
                        "Right operand of {:?} has type {}, expected Int",
                        op,
                        right_ty
                    ));
                }

                Ok(int_ty)
            }
        }
    }

    /// Check if two types are compatible (equal or unifiable)
    fn types_compatible(&self, t1: &Type, t2: &Type) -> bool {
        Self::check_types_compatible(t1, t2)
    }

    fn check_types_compatible(t1: &Type, t2: &Type) -> bool {
        match (t1, t2) {
            (Type::Named(n1), Type::Named(n2)) => n1 == n2,
            (Type::List(inner1), Type::List(inner2)) => {
                Self::check_types_compatible(inner1, inner2)
            }
            (Type::Option(inner1), Type::Option(inner2)) => {
                Self::check_types_compatible(inner1, inner2)
            }
            (Type::Product(types1), Type::Product(types2)) => {
                types1.len() == types2.len()
                    && types1
                        .iter()
                        .zip(types2.iter())
                        .all(|(t1, t2)| Self::check_types_compatible(t1, t2))
            }
            _ => false,
        }
    }

    /// Check if a term is ground (contains no variables)
    fn is_ground(&self, term: &Term) -> bool {
        Self::check_is_ground(term)
    }

    fn check_is_ground(term: &Term) -> bool {
        match term {
            Term::Var(_, _) => false,
            Term::Int(_, _) => true,
            Term::String(_, _) => true,
            Term::Construct { args, .. } => args.iter().all(Self::check_is_ground),
            Term::BinOp { left, right, .. } => {
                Self::check_is_ground(left) && Self::check_is_ground(right)
            }
        }
    }

    /// Get the expected types for a relation's arguments
    fn get_relation_arg_types(&self, name: &str) -> Result<Vec<Type>> {
        let sig = self
            .env
            .get_relation_signature(name)
            .ok_or_else(|| anyhow!("Unknown relation: {}", name))?;

        match sig {
            Type::Product(types) => Ok(types.clone()),
            single => Ok(vec![single.clone()]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

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

        // Verify constructor was registered
        let info = env.get_constructor_info("person");
        assert!(info.is_some());
        let (param_types, result_type) = info.unwrap();
        assert_eq!(param_types.len(), 2);
        assert_eq!(result_type, "Person");
    }

    #[test]
    fn test_check_valid_program() {
        let source = r#"
            type Person = person(name: String, age: Int)
            rel parent: Person × Person
            parent(person("Alice", 45), person("Bob", 20)).
        "#;

        let program = parser::parse(source).expect("Parse failed");
        let mut checker = TypeChecker::new();
        assert!(checker.check_program(&program).is_ok());
    }

    #[test]
    fn test_check_rule_with_variables() {
        let source = r#"
            type Person = person(name: String, age: Int)
            rel parent: Person × Person
            rel ancestor: Person × Person
            ancestor(X, Y) :- parent(X, Y).
        "#;

        let program = parser::parse(source).expect("Parse failed");
        let mut checker = TypeChecker::new();
        assert!(checker.check_program(&program).is_ok());
    }

    #[test]
    fn test_check_recursive_rule() {
        let source = r#"
            type Person = person(name: String, age: Int)
            rel parent: Person × Person
            rel ancestor: Person × Person
            ancestor(X, Y) :- parent(X, Y).
            ancestor(X, Z) :- parent(X, Y), ancestor(Y, Z).
        "#;

        let program = parser::parse(source).expect("Parse failed");
        let mut checker = TypeChecker::new();
        assert!(checker.check_program(&program).is_ok());
    }

    #[test]
    fn test_check_unknown_relation() {
        let source = r#"
            type Person = person(name: String, age: Int)
            unknown_rel(person("Alice", 45)).
        "#;

        let program = parser::parse(source).expect("Parse failed");
        let mut checker = TypeChecker::new();
        let result = checker.check_program(&program);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown relation"));
    }

    #[test]
    fn test_check_wrong_arity() {
        let source = r#"
            type Person = person(name: String, age: Int)
            rel parent: Person × Person
            parent(person("Alice", 45)).
        "#;

        let program = parser::parse(source).expect("Parse failed");
        let mut checker = TypeChecker::new();
        let result = checker.check_program(&program);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("arguments"));
    }

    #[test]
    fn test_check_type_mismatch() {
        let source = r#"
            type Person = person(name: String, age: Int)
            rel parent: Person × Person
            parent(person("Alice", 45), 42).
        "#;

        let program = parser::parse(source).expect("Parse failed");
        let mut checker = TypeChecker::new();
        let result = checker.check_program(&program);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_variable_in_fact() {
        let source = r#"
            type Person = person(name: String, age: Int)
            rel parent: Person × Person
            parent(person("Alice", 45), X).
        "#;

        let program = parser::parse(source).expect("Parse failed");
        let mut checker = TypeChecker::new();
        let result = checker.check_program(&program);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("variables"));
    }

    #[test]
    fn test_check_query() {
        let source = r#"
            type Person = person(name: String, age: Int)
            rel ancestor: Person × Person
            ?- ancestor(X, Y).
        "#;

        let program = parser::parse(source).expect("Parse failed");
        let mut checker = TypeChecker::new();
        assert!(checker.check_program(&program).is_ok());
    }

    #[test]
    fn test_check_comparison() {
        let source = r#"
            type Person = person(name: String, age: Int)
            rel parent: Person × Person
            rel older_parent: Person × Person
            older_parent(X, Y) :- parent(X, Y), X = person(N1, A1), Y = person(N2, A2), A1 > A2.
        "#;

        let program = parser::parse(source).expect("Parse failed");
        let mut checker = TypeChecker::new();
        // This should work - A1 and A2 are Int from the person constructor
        assert!(checker.check_program(&program).is_ok());
    }
}
