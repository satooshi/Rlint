use crate::diagnostic::{Diagnostic, Severity};
use crate::lexer::TokenKind;
use super::{LintContext, Rule};

/// Style rules: spacing, semicolons, parentheses, etc.
pub struct StyleRule;

impl Rule for StyleRule {
    fn name(&self) -> &'static str { "R020" }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let tokens = ctx.tokens;

        // R020: No semicolons at end of line (use newline instead)
        for tok in tokens {
            if tok.kind == TokenKind::Semicolon {
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

        // R021: Space around operators (=, ==, !=, <, >, etc.)
        // Check: token is operator, previous non-ws token exists, check spacing
        let mut i = 1;
        while i < tokens.len() {
            let tok = &tokens[i];

            let is_binary_op = matches!(
                tok.kind,
                TokenKind::EqEq | TokenKind::NotEq | TokenKind::Lt | TokenKind::Gt
                | TokenKind::LtEq | TokenKind::GtEq | TokenKind::And2 | TokenKind::Or2
                | TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash
            );

            if is_binary_op {
                let prev = &tokens[i - 1];
                let next = tokens.get(i + 1);

                // No space before operator
                if prev.kind != TokenKind::Whitespace && prev.kind != TokenKind::Newline {
                    diags.push(Diagnostic::new(
                        ctx.file,
                        tok.line,
                        tok.col,
                        "R021",
                        format!("Missing space before `{}`", tok.text),
                        Severity::Warning,
                    ));
                }

                // No space after operator
                if let Some(next) = next {
                    if next.kind != TokenKind::Whitespace && next.kind != TokenKind::Newline {
                        diags.push(Diagnostic::new(
                            ctx.file,
                            tok.line,
                            tok.col + tok.text.len(),
                            "R021",
                            format!("Missing space after `{}`", tok.text),
                            Severity::Warning,
                        ));
                    }
                }
            }

            // R022: Trailing comma in method definition parameters
            if tok.kind == TokenKind::Comma {
                if let Some(next) = tokens.get(i + 1) {
                    let real_next = if next.kind == TokenKind::Whitespace { tokens.get(i + 2) } else { Some(next) };
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

        // R023: Multiple empty lines (> 2 consecutive blank lines)
        let mut blank_count = 0usize;
        for (i, line) in ctx.lines.iter().enumerate() {
            if line.trim().is_empty() {
                blank_count += 1;
                if blank_count > 2 {
                    diags.push(Diagnostic::new(
                        ctx.file,
                        i + 1,
                        1,
                        "R023",
                        "Too many consecutive blank lines (maximum 2)",
                        Severity::Warning,
                    ));
                }
            } else {
                blank_count = 0;
            }
        }

        // R024: `puts` with no args should use `puts` not `p nil`
        for (i, tok) in tokens.iter().enumerate() {
            if tok.kind == TokenKind::Ident && tok.text == "p" {
                if let Some(next) = tokens.get(i + 1) {
                    let is_nil = if next.kind == TokenKind::Whitespace {
                        tokens.get(i + 2).map_or(false, |t| t.kind == TokenKind::Nil)
                    } else {
                        next.kind == TokenKind::Nil
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
