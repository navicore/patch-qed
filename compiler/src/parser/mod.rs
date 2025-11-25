/// Parser for qed source code
///
/// Uses logos for lexing and chumsky for parsing.
use crate::ast::*;
use anyhow::Result;

pub mod grammar;
pub mod lexer;

/// Parse a qed source file into an AST
pub fn parse(source: &str) -> Result<Program> {
    use chumsky::prelude::*;
    use logos::Logos;

    // 1. Tokenize with logos
    let tokens: Vec<lexer::Token> = lexer::Token::lexer(source)
        .filter_map(|tok| tok.ok())
        .collect();

    // 2. Parse with chumsky
    let program = grammar::program_parser()
        .parse(&tokens)
        .into_result()
        .map_err(|errors| {
            // Format parse errors nicely
            let error_msgs: Vec<String> = errors
                .iter()
                .map(|e| format!("Parse error: {:?}", e))
                .collect();
            anyhow::anyhow!("Parse errors:\n{}", error_msgs.join("\n"))
        })?;

    Ok(program)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let result = parse("");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_type_def() {
        let source = "type Person = person(name: String, age: Int)";
        let result = parse(source);
        // TODO: Assert AST structure
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_relation() {
        let source = "rel parent: Person × Person";
        let result = parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_fact() {
        let source = r#"parent(person("Alice", 45), person("Bob", 20))."#;
        let result = parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_rule() {
        let source = "ancestor(X, Y) :- parent(X, Y).";
        let result = parse(source);
        assert!(result.is_ok());
    }
}
