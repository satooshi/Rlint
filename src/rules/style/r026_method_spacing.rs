use crate::diagnostic::{Diagnostic, Severity};
use crate::lexer::TokenKind;
use crate::rules::{LintContext, Rule};

/// Returns the leading whitespace of a line (spaces/tabs before first non-ws char).
pub fn leading_whitespace(line: &str) -> &str {
    let trimmed = line.trim_start();
    &line[..line.len() - trimmed.len()]
}

pub struct MethodSpacingRule;

impl Rule for MethodSpacingRule {
    fn name(&self) -> &'static str {
        "R026"
    }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let tokens = ctx.tokens;

        // R026: Missing blank line between method definitions
        // Track a stack of block-opening keywords so we know whether an `end` closes a `def`.
        // Only fire when a method-closing `end` is immediately followed by `def` with no
        // blank line in between.
        {
            // Build a set of lines that have a method-closing `end` by scanning tokens
            // with a simple keyword stack (def/class/module/do/if/unless/while/until/for/begin/case).
            let mut block_stack: Vec<TokenKind> = Vec::new();
            let mut method_end_lines = std::collections::HashSet::new();
            for tok in tokens {
                match tok.kind {
                    TokenKind::Def
                    | TokenKind::Class
                    | TokenKind::Module
                    | TokenKind::Do
                    | TokenKind::If
                    | TokenKind::Unless
                    | TokenKind::While
                    | TokenKind::Until
                    | TokenKind::For
                    | TokenKind::Begin
                    | TokenKind::Case => {
                        // Only push if the keyword starts a block. For if/unless/while/until,
                        // modifier forms (e.g. `x = 1 if cond`) don't have a matching `end`,
                        // so only push when the keyword is the first non-whitespace token on
                        // the line.
                        let is_modifier_capable = matches!(
                            tok.kind,
                            TokenKind::If | TokenKind::Unless | TokenKind::While | TokenKind::Until
                        );
                        if is_modifier_capable {
                            let line_idx = tok.line.saturating_sub(1);
                            let line = ctx.lines.get(line_idx).copied().unwrap_or("");
                            let first_non_ws = line.trim_start();
                            // Check if this keyword is at the start of the line
                            if !first_non_ws.starts_with(tok.text.as_str()) {
                                // Modifier form — skip pushing
                                continue;
                            }
                        }
                        block_stack.push(tok.kind.clone());
                    }
                    TokenKind::End => {
                        if let Some(opener) = block_stack.pop() {
                            if opener == TokenKind::Def {
                                method_end_lines.insert(tok.line);
                            }
                        }
                    }
                    _ => {}
                }
            }

            for i in 0..ctx.lines.len().saturating_sub(1) {
                let line_no = i + 1; // 1-based
                if !method_end_lines.contains(&line_no) {
                    continue;
                }
                let current = ctx.lines[i];
                let next = ctx.lines[i + 1];
                let current_trimmed = current.trim();
                let next_trimmed = next.trim();
                if (current_trimmed == "end"
                    || current_trimmed.starts_with("end ")
                    || current_trimmed.starts_with("end#"))
                    && (next_trimmed.starts_with("def ") || next_trimmed == "def")
                {
                    let end_indent = leading_whitespace(current);
                    let def_indent = leading_whitespace(next);
                    if end_indent != def_indent {
                        continue;
                    }
                    diags.push(
                        Diagnostic::new(
                            ctx.file,
                            i + 2,
                            1,
                            "R026",
                            "Missing blank line between method definitions",
                            Severity::Warning,
                        )
                        // Empty string = blank line (the fixer inserts it as a new line
                        // in the lines vector, producing a blank line in the output).
                        .with_insert_before_fix(String::new()),
                    );
                }
            }
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::FixKind;
    use crate::lexer::Lexer;

    fn check(source: &str) -> Vec<Diagnostic> {
        let lines: Vec<&str> = source.lines().collect();
        let tokens = Lexer::new(source).tokenize();
        let ctx = LintContext::new("test.rb", source, &lines, &tokens);
        MethodSpacingRule.check(&ctx)
    }

    fn has_rule(diags: &[Diagnostic], rule: &str) -> bool {
        diags.iter().any(|d| d.rule == rule)
    }

    #[test]
    fn violation_no_blank_line_between_methods() {
        let source = "def foo\n  1\nend\ndef bar\n  2\nend\n";
        let diags = check(source);
        assert!(has_rule(&diags, "R026"), "{diags:?}");
    }

    #[test]
    fn no_violation_r026_after_non_method_end() {
        // `end` closing an `if` block should not trigger R026
        let source = "if true\n  1\nend\ndef bar\n  2\nend\n";
        let diags = check(source);
        assert!(!has_rule(&diags, "R026"), "{diags:?}");
    }

    #[test]
    fn no_violation_blank_line_between_methods() {
        let source = "def foo\n  1\nend\n\ndef bar\n  2\nend\n";
        let diags = check(source);
        assert!(!has_rule(&diags, "R026"), "{diags:?}");
    }

    #[test]
    fn fix_r026_insert_before() {
        let source = "def foo\n  1\nend\ndef bar\n  2\nend\n";
        let diags = check(source);
        let diag = diags.iter().find(|d| d.rule == "R026").expect("R026 diag");
        assert_eq!(diag.fix_kind, FixKind::InsertBefore);
    }

    #[test]
    fn fix_r026_applied() {
        use crate::fixer::apply_fixes;
        let source = "def foo\n  1\nend\ndef bar\n  2\nend\n";
        let diags = check(source);
        let (fixed, count) = apply_fixes(source, &diags);
        assert!(count > 0);
        // There should now be a blank line between methods
        assert!(fixed.contains("end\n\ndef"), "fixed: {fixed:?}");
    }
}
