mod line_length;
mod trailing_whitespace;
mod frozen_string_literal;
mod missing_frozen_literal;
mod naming;
mod style;
mod syntax;
mod complexity;

use crate::diagnostic::Diagnostic;
use crate::lexer::Token;

pub use line_length::LineLengthRule;
pub use trailing_whitespace::TrailingWhitespaceRule;
pub use frozen_string_literal::FrozenStringLiteralRule;
pub use naming::NamingRule;
pub use style::StyleRule;
pub use syntax::SyntaxRule;
pub use complexity::ComplexityRule;

/// Context passed to each rule
pub struct LintContext<'a> {
    pub file: &'a str,
    pub source: &'a str,
    pub lines: &'a [&'a str],
    pub tokens: &'a [Token],
}

/// Trait implemented by every lint rule
pub trait Rule {
    fn name(&self) -> &'static str;
    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic>;
}

/// Returns all built-in rules
pub fn all_rules() -> Vec<Box<dyn Rule + Send + Sync>> {
    vec![
        Box::new(LineLengthRule::default()),
        Box::new(TrailingWhitespaceRule),
        Box::new(FrozenStringLiteralRule),
        Box::new(NamingRule),
        Box::new(StyleRule),
        Box::new(SyntaxRule),
        Box::new(ComplexityRule),
    ]
}
