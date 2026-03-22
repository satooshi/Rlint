use crate::diagnostic::{Diagnostic, Severity};
use crate::lexer::TokenKind;
use crate::rules::{LintContext, Rule};

/// Returns true if the character position `col` (0-based byte offset) in `line`
/// is inside a string literal.
pub fn is_inside_string(line: &str, col: usize) -> bool {
    let mut in_string = false;
    let mut quote_char = '"';
    let mut escaped = false;
    for (i, c) in line.char_indices() {
        if i >= col {
            break;
        }
        if escaped {
            escaped = false;
            continue;
        }
        if in_string {
            if c == '\\' {
                escaped = true;
            } else if c == quote_char {
                in_string = false;
            }
        } else if c == '"' || c == '\'' {
            in_string = true;
            quote_char = c;
        }
    }
    in_string
}

pub struct SemicolonRule;

impl Rule for SemicolonRule {
    fn name(&self) -> &'static str {
        "R020"
    }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let tokens = ctx.tokens;

        // R020: No semicolons at end of line (use newline instead)
        // Warning only (no auto-fix). Skip semicolons inside string literals.
        for tok in tokens {
            if tok.kind == TokenKind::Semicolon {
                // tok.col is 1-based character column; convert to 0-based byte offset
                let line_idx = tok.line.saturating_sub(1);
                let line = ctx.lines.get(line_idx).copied().unwrap_or("");
                // Convert 1-based char column to byte offset for is_inside_string
                let col_char = tok.col.saturating_sub(1);
                let col_byte = line
                    .char_indices()
                    .nth(col_char)
                    .map(|(b, _)| b)
                    .unwrap_or(line.len());
                if is_inside_string(line, col_byte) {
                    continue;
                }
                // No auto-fix for R020: splitting at semicolons requires multi-line
                // fix support which the fixer does not yet implement.
                diags.push(Diagnostic::new(
                    ctx.file,
                    tok.line,
                    tok.col,
                    "R020",
                    "Avoid using semicolons to separate statements; use a newline instead",
                    Severity::Warning,
                ));
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
        SemicolonRule.check(&ctx)
    }

    fn has_rule(diags: &[Diagnostic], rule: &str) -> bool {
        diags.iter().any(|d| d.rule == rule)
    }

    fn fix_for_rule<'a>(diags: &'a [Diagnostic], rule: &str) -> Option<&'a str> {
        diags
            .iter()
            .find(|d| d.rule == rule && d.fix.is_some())
            .and_then(|d| d.fix.as_deref())
    }

    #[test]
    fn violation_semicolon_between_statements() {
        let diags = check("x = 1; y = 2");
        assert!(has_rule(&diags, "R020"));
    }

    #[test]
    fn no_violation_no_semicolons() {
        let diags = check("x = 1\ny = 2");
        assert!(!has_rule(&diags, "R020"));
    }

    #[test]
    fn no_fix_for_semicolon() {
        // R020 does not provide auto-fix (multi-line fix not yet supported)
        let diags = check("x = 1; y = 2");
        assert!(fix_for_rule(&diags, "R020").is_none());
    }

    #[test]
    fn no_violation_semicolon_in_string() {
        // Semicolons inside string literals should not trigger R020
        let diags = check(r#"x = "a; b""#);
        assert!(!has_rule(&diags, "R020"));
    }
}
