/// Lexer for leem using logos
///
/// Converts source text into tokens.

use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(skip r"//[^\n]*")]
#[logos(skip r"/\*([^*]|\*[^/])*\*/")]
pub enum Token {
    // Keywords
    #[token("type")]
    Type,

    #[token("rel")]
    Rel,

    // Literals
    #[regex(r"[0-9]+", |lex| lex.slice().parse().ok())]
    Int(i64),

    #[regex(r#""([^"\\]|\\["\\bnfrt])*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    String(String),

    // Identifiers
    #[regex(r"[a-z][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    LowerId(String),

    #[regex(r"[A-Z][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    UpperId(String),

    // Operators
    #[token(":-")]
    ColonDash,

    #[token("?-")]
    QuestionDash,

    #[token("=")]
    Eq,

    #[token("==")]
    EqEq,

    #[token("!=")]
    Ne,

    #[token("<")]
    Lt,

    #[token("<=")]
    Le,

    #[token(">")]
    Gt,

    #[token(">=")]
    Ge,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("%")]
    Percent,

    #[token("×")]
    Times,

    // Delimiters
    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token(",")]
    Comma,

    #[token(".")]
    Dot,

    #[token(":")]
    Colon,

    #[token("|")]
    Pipe,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_keywords() {
        let mut lex = Token::lexer("type rel");
        assert_eq!(lex.next(), Some(Ok(Token::Type)));
        assert_eq!(lex.next(), Some(Ok(Token::Rel)));
    }

    #[test]
    fn test_lex_identifiers() {
        let mut lex = Token::lexer("person Alice");
        assert_eq!(lex.next(), Some(Ok(Token::LowerId("person".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::UpperId("Alice".to_string()))));
    }

    #[test]
    fn test_lex_numbers() {
        let mut lex = Token::lexer("42 0 999");
        assert_eq!(lex.next(), Some(Ok(Token::Int(42))));
        assert_eq!(lex.next(), Some(Ok(Token::Int(0))));
        assert_eq!(lex.next(), Some(Ok(Token::Int(999))));
    }

    #[test]
    fn test_lex_strings() {
        let mut lex = Token::lexer(r#""hello" "world""#);
        assert_eq!(lex.next(), Some(Ok(Token::String("hello".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::String("world".to_string()))));
    }

    #[test]
    fn test_lex_operators() {
        let mut lex = Token::lexer(":- ?- = == != < <= > >=");
        assert_eq!(lex.next(), Some(Ok(Token::ColonDash)));
        assert_eq!(lex.next(), Some(Ok(Token::QuestionDash)));
        assert_eq!(lex.next(), Some(Ok(Token::Eq)));
        assert_eq!(lex.next(), Some(Ok(Token::EqEq)));
        assert_eq!(lex.next(), Some(Ok(Token::Ne)));
        assert_eq!(lex.next(), Some(Ok(Token::Lt)));
        assert_eq!(lex.next(), Some(Ok(Token::Le)));
        assert_eq!(lex.next(), Some(Ok(Token::Gt)));
        assert_eq!(lex.next(), Some(Ok(Token::Ge)));
    }

    #[test]
    fn test_skip_comments() {
        let mut lex = Token::lexer("type // comment\nrel /* block */ ×");
        assert_eq!(lex.next(), Some(Ok(Token::Type)));
        assert_eq!(lex.next(), Some(Ok(Token::Rel)));
        assert_eq!(lex.next(), Some(Ok(Token::Times)));
    }
}
