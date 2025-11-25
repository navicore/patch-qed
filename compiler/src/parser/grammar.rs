/// Grammar definitions using chumsky parser combinators
///
/// This module contains the parser implementation for qed syntax.
use super::lexer::Token;
use crate::ast::*;
use chumsky::prelude::*;

/// Convert chumsky span to our Span
fn to_span(span: SimpleSpan) -> Span {
    Span {
        start: span.start,
        end: span.end,
    }
}

/// Parse a type expression
/// Examples: Person, Int, String, Person × Person, List<Int>
pub fn type_parser<'a>() -> impl Parser<'a, &'a [Token], Type, extra::Err<Simple<'a, Token>>> + Clone
{
    recursive(|ty| {
        // Named type: Person, Int, String
        let named = select! {
            Token::UpperId(name) => Type::Named(name),
        };

        // List type: List<T>
        let list = just(Token::UpperId("List".to_string()))
            .ignore_then(just(Token::Lt))
            .ignore_then(ty.clone())
            .then_ignore(just(Token::Gt))
            .map(|inner| Type::List(Box::new(inner)));

        // Option type: Option<T>
        let option = just(Token::UpperId("Option".to_string()))
            .ignore_then(just(Token::Lt))
            .ignore_then(ty.clone())
            .then_ignore(just(Token::Gt))
            .map(|inner| Type::Option(Box::new(inner)));

        // Base type (named, list, or option)
        let base = choice((list, option, named));

        // Product type: Person × Int × String
        base.clone().foldl(
            just(Token::Times).ignore_then(base.clone()).repeated(),
            |acc, next| match acc {
                Type::Product(mut types) => {
                    types.push(next);
                    Type::Product(types)
                }
                single => Type::Product(vec![single, next]),
            },
        )
    })
}

/// Parse a term (with binary operators)
/// Examples: X, 42, "hello", person("Alice", 45), X + Y
pub fn term_parser<'a>() -> impl Parser<'a, &'a [Token], Term, extra::Err<Simple<'a, Token>>> + Clone
{
    recursive(|term| {
        // Variable: X, Y, Age
        let var = select! {
            Token::UpperId(name) => name,
        }
        .map_with(|name, e| Term::Var(name, to_span(e.span())));

        // Integer literal: 42
        let int = select! {
            Token::Int(n) => n,
        }
        .map_with(|n, e| Term::Int(n, to_span(e.span())));

        // String literal: "hello"
        let string = select! {
            Token::String(s) => s,
        }
        .map_with(|s, e| Term::String(s, to_span(e.span())));

        // Constructor: person("Alice", 45) or just a lowercase identifier
        let constructor = select! {
            Token::LowerId(name) => name,
        }
        .then(
            term.clone()
                .separated_by(just(Token::Comma))
                .collect()
                .delimited_by(just(Token::LParen), just(Token::RParen))
                .or_not(),
        )
        .map_with(|(constructor, args_opt), e| {
            if let Some(args) = args_opt {
                Term::Construct {
                    constructor,
                    args,
                    span: to_span(e.span()),
                }
            } else {
                // Constructor with no args is treated as a zero-arity constructor
                Term::Construct {
                    constructor,
                    args: vec![],
                    span: to_span(e.span()),
                }
            }
        });

        // Parenthesized term
        let parens = term
            .clone()
            .delimited_by(just(Token::LParen), just(Token::RParen));

        // Atom (base term without operators)
        let atom = choice((parens, constructor, var, int, string));

        // Binary operators with precedence
        // Multiplicative: *, /, %
        let op_mul = choice((
            just(Token::Star).to(BinOp::Mul),
            just(Token::Slash).to(BinOp::Div),
            just(Token::Percent).to(BinOp::Mod),
        ));

        let multiplicative =
            atom.clone()
                .foldl(op_mul.then(atom.clone()).repeated(), |left, (op, right)| {
                    let start = left.span().start;
                    let end = right.span().end;
                    Term::BinOp {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                        span: Span { start, end },
                    }
                });

        // Additive: +, -
        let op_add = choice((
            just(Token::Plus).to(BinOp::Add),
            just(Token::Minus).to(BinOp::Sub),
        ));

        multiplicative.clone().foldl(
            op_add.then(multiplicative).repeated(),
            |left, (op, right)| {
                let start = left.span().start;
                let end = right.span().end;
                Term::BinOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                    span: Span { start, end },
                }
            },
        )
    })
}

/// Parse an atom: parent(X, Y)
pub fn atom_parser<'a>() -> impl Parser<'a, &'a [Token], Atom, extra::Err<Simple<'a, Token>>> + Clone
{
    select! {
        Token::LowerId(name) => name,
    }
    .then(
        term_parser()
            .separated_by(just(Token::Comma))
            .collect()
            .delimited_by(just(Token::LParen), just(Token::RParen)),
    )
    .map_with(|(relation, args), e| Atom {
        relation,
        args,
        span: to_span(e.span()),
    })
}

/// Parse a goal in a rule body
/// Examples: parent(X, Y), X = Y, Age > 18
pub fn goal_parser<'a>() -> impl Parser<'a, &'a [Token], Goal, extra::Err<Simple<'a, Token>>> + Clone
{
    let term = term_parser();

    // Comparison: X > Y, Age >= 18, etc.
    let comparison = term
        .clone()
        .then(choice((
            just(Token::EqEq).to(CompareOp::Eq),
            just(Token::Ne).to(CompareOp::Ne),
            just(Token::Le).to(CompareOp::Le),
            just(Token::Ge).to(CompareOp::Ge),
            just(Token::Lt).to(CompareOp::Lt),
            just(Token::Gt).to(CompareOp::Gt),
        )))
        .then(term.clone())
        .map_with(|((left, op), right), e| Goal::Compare(op, left, right, to_span(e.span())));

    // Unification: X = Y
    let unify = term
        .clone()
        .then_ignore(just(Token::Eq))
        .then(term.clone())
        .map_with(|(left, right), e| Goal::Unify(left, right, to_span(e.span())));

    // Atom: parent(X, Y)
    let atom = atom_parser().map(Goal::Atom);

    // Try comparison and unify before atom (to avoid ambiguity)
    choice((comparison, unify, atom))
}

/// Parse a type definition
/// Example: type Person = person(name: String, age: Int)
pub fn type_def_parser<'a>(
) -> impl Parser<'a, &'a [Token], TypeDef, extra::Err<Simple<'a, Token>>> + Clone {
    let field = select! {
        Token::LowerId(name) => name,
    }
    .then_ignore(just(Token::Colon))
    .then(type_parser())
    .map(|(name, ty)| Field { name, ty });

    // Product type: person(name: String, age: Int)
    let product = select! {
        Token::LowerId(constructor) => constructor,
    }
    .then(
        field
            .separated_by(just(Token::Comma))
            .collect()
            .delimited_by(just(Token::LParen), just(Token::RParen)),
    )
    .map(|(constructor, fields)| TypeDefKind::Product {
        constructor,
        fields,
    });

    // Sum type: Public | Internal | Secret
    let sum = select! {
        Token::UpperId(variant) => variant,
    }
    .separated_by(just(Token::Pipe))
    .at_least(1)
    .collect()
    .map(|variants| TypeDefKind::Sum { variants });

    just(Token::Type)
        .ignore_then(select! {
            Token::UpperId(name) => name,
        })
        .then_ignore(just(Token::Eq))
        .then(choice((product, sum)))
        .map_with(|(name, def), e| TypeDef {
            name,
            def,
            span: to_span(e.span()),
        })
}

/// Parse a relation declaration
/// Example: rel parent: Person × Person
pub fn relation_decl_parser<'a>(
) -> impl Parser<'a, &'a [Token], RelationDecl, extra::Err<Simple<'a, Token>>> + Clone {
    just(Token::Rel)
        .ignore_then(select! {
            Token::LowerId(name) => name,
        })
        .then_ignore(just(Token::Colon))
        .then(type_parser())
        .map_with(|(name, signature), e| RelationDecl {
            name,
            signature,
            span: to_span(e.span()),
        })
}

/// Parse a fact
/// Example: parent(person("Alice", 45), person("Bob", 20)).
pub fn fact_parser<'a>() -> impl Parser<'a, &'a [Token], Fact, extra::Err<Simple<'a, Token>>> + Clone
{
    select! {
        Token::LowerId(name) => name,
    }
    .then(
        term_parser()
            .separated_by(just(Token::Comma))
            .collect()
            .delimited_by(just(Token::LParen), just(Token::RParen)),
    )
    .then_ignore(just(Token::Dot))
    .map_with(|(relation, args), e| Fact {
        relation,
        args,
        span: to_span(e.span()),
    })
}

/// Parse a rule
/// Example: ancestor(X, Y) :- parent(X, Y).
pub fn rule_parser<'a>() -> impl Parser<'a, &'a [Token], Rule, extra::Err<Simple<'a, Token>>> + Clone
{
    atom_parser()
        .then_ignore(just(Token::ColonDash))
        .then(goal_parser().separated_by(just(Token::Comma)).collect())
        .then_ignore(just(Token::Dot))
        .map_with(|(head, body), e| Rule {
            head,
            body,
            span: to_span(e.span()),
        })
}

/// Parse a query
/// Example: ?- ancestor(person("Alice", 45), X).
pub fn query_parser<'a>(
) -> impl Parser<'a, &'a [Token], Query, extra::Err<Simple<'a, Token>>> + Clone {
    just(Token::QuestionDash)
        .ignore_then(goal_parser().separated_by(just(Token::Comma)).collect())
        .then_ignore(just(Token::Dot))
        .map_with(|goals, e| Query {
            goals,
            span: to_span(e.span()),
        })
}

/// Parse a top-level item
pub fn item_parser<'a>() -> impl Parser<'a, &'a [Token], Item, extra::Err<Simple<'a, Token>>> + Clone
{
    choice((
        type_def_parser().map(Item::TypeDef),
        relation_decl_parser().map(Item::RelationDecl),
        query_parser().map(Item::Query),
        rule_parser().map(Item::Rule),
        fact_parser().map(Item::Fact),
    ))
}

/// Parse a complete program
pub fn program_parser<'a>(
) -> impl Parser<'a, &'a [Token], Program, extra::Err<Simple<'a, Token>>> + Clone {
    item_parser()
        .repeated()
        .collect()
        .then_ignore(end())
        .map(|items| Program { items })
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;

    fn lex(source: &str) -> Vec<Token> {
        Token::lexer(source).filter_map(|tok| tok.ok()).collect()
    }

    #[test]
    fn test_parse_type_named() {
        let tokens = lex("Person");
        let result = type_parser().parse(&tokens).into_result();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::Named("Person".to_string()));
    }

    #[test]
    fn test_parse_type_product() {
        let tokens = lex("Person × Int");
        let result = type_parser().parse(&tokens).into_result();
        assert!(result.is_ok());
        match result.unwrap() {
            Type::Product(types) => {
                assert_eq!(types.len(), 2);
                assert_eq!(types[0], Type::Named("Person".to_string()));
                assert_eq!(types[1], Type::Named("Int".to_string()));
            }
            _ => panic!("Expected product type"),
        }
    }

    #[test]
    fn test_parse_term_var() {
        let tokens = lex("X");
        let result = term_parser().parse(&tokens).into_result();
        assert!(result.is_ok());
        match result.unwrap() {
            Term::Var(name, _) => assert_eq!(name, "X"),
            _ => panic!("Expected variable"),
        }
    }

    #[test]
    fn test_parse_term_int() {
        let tokens = lex("42");
        let result = term_parser().parse(&tokens).into_result();
        assert!(result.is_ok());
        match result.unwrap() {
            Term::Int(n, _) => assert_eq!(n, 42),
            _ => panic!("Expected integer"),
        }
    }

    #[test]
    fn test_parse_term_constructor() {
        let tokens = lex(r#"person("Alice", 45)"#);
        let result = term_parser().parse(&tokens).into_result();
        assert!(result.is_ok());
        match result.unwrap() {
            Term::Construct {
                constructor, args, ..
            } => {
                assert_eq!(constructor, "person");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected constructor"),
        }
    }

    #[test]
    fn test_parse_term_binop() {
        let tokens = lex("X + Y");
        let result = term_parser().parse(&tokens).into_result();
        assert!(result.is_ok());
        match result.unwrap() {
            Term::BinOp { op, .. } => assert_eq!(op, BinOp::Add),
            _ => panic!("Expected binary operation"),
        }
    }

    #[test]
    fn test_parse_atom() {
        let tokens = lex("parent(X, Y)");
        let result = atom_parser().parse(&tokens).into_result();
        assert!(result.is_ok());
        let atom = result.unwrap();
        assert_eq!(atom.relation, "parent");
        assert_eq!(atom.args.len(), 2);
    }

    #[test]
    fn test_parse_type_def() {
        let tokens = lex("type Person = person(name: String, age: Int)");
        let result = type_def_parser().parse(&tokens).into_result();
        assert!(result.is_ok());
        let typedef = result.unwrap();
        assert_eq!(typedef.name, "Person");
        match typedef.def {
            TypeDefKind::Product {
                constructor,
                fields,
            } => {
                assert_eq!(constructor, "person");
                assert_eq!(fields.len(), 2);
            }
            _ => panic!("Expected product type"),
        }
    }

    #[test]
    fn test_parse_relation_decl() {
        let tokens = lex("rel parent: Person × Person");
        let result = relation_decl_parser().parse(&tokens).into_result();
        assert!(result.is_ok());
        let decl = result.unwrap();
        assert_eq!(decl.name, "parent");
    }

    #[test]
    fn test_parse_fact() {
        let tokens = lex(r#"parent(person("Alice", 45), person("Bob", 20))."#);
        let result = fact_parser().parse(&tokens).into_result();
        assert!(result.is_ok());
        let fact = result.unwrap();
        assert_eq!(fact.relation, "parent");
        assert_eq!(fact.args.len(), 2);
    }

    #[test]
    fn test_parse_rule() {
        let tokens = lex("ancestor(X, Y) :- parent(X, Y).");
        let result = rule_parser().parse(&tokens).into_result();
        assert!(result.is_ok());
        let rule = result.unwrap();
        assert_eq!(rule.head.relation, "ancestor");
        assert_eq!(rule.body.len(), 1);
    }

    #[test]
    fn test_parse_query() {
        let tokens = lex("?- ancestor(X, Y).");
        let result = query_parser().parse(&tokens).into_result();
        assert!(result.is_ok());
        let query = result.unwrap();
        assert_eq!(query.goals.len(), 1);
    }
}
