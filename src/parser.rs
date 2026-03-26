/// Ruby parser wrapper around lib-ruby-parser.
///
/// Provides a simple `parse()` function that converts Ruby source code
/// into an AST node, used by AST-based linting rules.
pub use lib_ruby_parser::Node;

use lib_ruby_parser::{Parser, ParserOptions};

/// Parse Ruby source code and return the root AST node.
///
/// Returns `None` when the parser produces no AST (e.g. empty source).
pub fn parse(source: &str) -> Option<Node> {
    let options = ParserOptions {
        buffer_name: "(input)".to_string(),
        ..Default::default()
    };
    let parser = Parser::new(source.as_bytes().to_vec(), options);
    let result = parser.do_parse();
    result.ast.map(|boxed| *boxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_assignment() {
        let node = parse("x = 1").expect("should parse assignment");
        match &node {
            Node::Lvasgn(_) => {} // expected
            other => panic!("expected Lvasgn, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_method_def() {
        let source = "def foo\n  42\nend";
        let node = parse(source).expect("should parse method def");
        match &node {
            Node::Def(_) => {} // expected
            other => panic!("expected Def, got {:?}", other),
        }
    }

    #[test]
    fn test_invalid_syntax_does_not_panic() {
        // Invalid syntax should not panic; it may return Some or None
        // depending on how much the parser can recover.
        let _result = parse("def end end end @@@ !!!");
    }

    #[test]
    fn test_empty_source() {
        let result = parse("");
        assert!(result.is_none(), "empty source should produce no AST");
    }
}
