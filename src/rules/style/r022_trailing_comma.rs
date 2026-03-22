use crate::diagnostic::{Diagnostic, Severity};
use crate::lexer::TokenKind;
use crate::rules::{LintContext, Rule};

pub struct TrailingCommaRule;

impl Rule for TrailingCommaRule {
    fn name(&self) -> &'static str {
        "R022"
    }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let tokens = ctx.tokens;

        // R022: Trailing comma in method definition parameters
        let mut i = 1;
        while i < tokens.len() {
            let tok = &tokens[i];

            if tok.kind == TokenKind::Comma {
                if let Some(next) = tokens.get(i + 1) {
                    let real_next = if next.kind == TokenKind::Whitespace {
                        tokens.get(i + 2)
                    } else {
                        Some(next)
                    };
                    if let Some(rn) = real_next {
                        if rn.kind == TokenKind::RParen {
                            diags.push(Diagnostic::new(
                                ctx.file,
                                tok.line,
                                tok.col,
                                "R022",
                                "Avoid trailing comma before closing parenthesis",
                                Severity::Warning,
                            ));
                        }
                    }
                }
            }

            i += 1;
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn check(source: &str) -> Vec<Diagnostic> {
        let lines: Vec<&str> = source.lines().collect();
        let tokens = Lexer::new(source).tokenize();
        let ctx = LintContext::new("test.rb", source, &lines, &tokens);
        TrailingCommaRule.check(&ctx)
    }

    fn has_rule(diags: &[Diagnostic], rule: &str) -> bool {
        diags.iter().any(|d| d.rule == rule)
    }

    #[test]
    fn violation_trailing_comma_before_rparen() {
        let diags = check("foo(a, b,)");
        assert!(has_rule(&diags, "R022"), "{diags:?}");
    }

    #[test]
    fn no_violation_no_trailing_comma() {
        let diags = check("foo(a, b)");
        assert!(!has_rule(&diags, "R022"));
    }
}
