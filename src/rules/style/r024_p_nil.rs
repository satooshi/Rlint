use crate::diagnostic::{Diagnostic, Severity};
use crate::lexer::TokenKind;
use crate::rules::{LintContext, Rule};

pub struct PNilRule;

impl Rule for PNilRule {
    fn name(&self) -> &'static str {
        "R024"
    }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let tokens = ctx.tokens;

        // R024: `puts` with no args should use `puts` not `p nil`
        for (i, tok) in tokens.iter().enumerate() {
            if tok.kind == TokenKind::Ident && tok.text == "p" {
                if let Some(next) = tokens.get(i + 1) {
                    let is_nil = if next.kind == TokenKind::Nil {
                        // `p nil`
                        true
                    } else if next.kind == TokenKind::Whitespace {
                        // `p nil` (with space) or `p (nil)` (space then paren)
                        match tokens.get(i + 2) {
                            Some(t) if t.kind == TokenKind::Nil => true,
                            Some(t) if t.kind == TokenKind::LParen => {
                                // `p (nil)`: skip LParen, check for Nil
                                tokens
                                    .get(i + 3)
                                    .is_some_and(|t2| t2.kind == TokenKind::Nil)
                            }
                            _ => false,
                        }
                    } else if next.kind == TokenKind::LParen {
                        // `p(nil)`: LParen directly after `p`
                        tokens.get(i + 2).is_some_and(|t| t.kind == TokenKind::Nil)
                    } else {
                        false
                    };
                    if is_nil {
                        diags.push(Diagnostic::new(
                            ctx.file,
                            tok.line,
                            tok.col,
                            "R024",
                            "Use `puts` instead of `p nil` to print a blank line",
                            Severity::Info,
                        ));
                    }
                }
            }
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
        PNilRule.check(&ctx)
    }

    fn has_rule(diags: &[Diagnostic], rule: &str) -> bool {
        diags.iter().any(|d| d.rule == rule)
    }

    #[test]
    fn violation_p_nil() {
        let diags = check("p nil");
        assert!(has_rule(&diags, "R024"), "{diags:?}");
    }

    #[test]
    fn no_violation_p_with_value() {
        let diags = check("p some_object");
        assert!(!has_rule(&diags, "R024"));
    }

    #[test]
    fn no_violation_puts() {
        let diags = check("puts");
        assert!(!has_rule(&diags, "R024"));
    }

    #[test]
    fn violation_p_paren_nil() {
        // p(nil) should trigger R024
        let diags = check("p(nil)");
        assert!(has_rule(&diags, "R024"), "{diags:?}");
    }

    #[test]
    fn violation_p_space_paren_nil() {
        // p (nil) should trigger R024
        let diags = check("p (nil)");
        assert!(has_rule(&diags, "R024"), "{diags:?}");
    }
}
