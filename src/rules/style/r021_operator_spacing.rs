use crate::diagnostic::{Diagnostic, Severity};
use crate::lexer::TokenKind;
use crate::rules::{LintContext, Rule};

pub struct OperatorSpacingRule;

impl Rule for OperatorSpacingRule {
    fn name(&self) -> &'static str {
        "R021"
    }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let tokens = ctx.tokens;

        // R021: Space around operators (==, !=, <, >, etc.)
        // Fix: insert missing spaces around the operator on the whole line.
        // Exclude ** (exponent) — the lexer emits two Star tokens, so we skip
        // both stars when they appear consecutively.
        let mut i = 1;
        while i < tokens.len() {
            let tok = &tokens[i];

            let is_binary_op = matches!(
                tok.kind,
                TokenKind::EqEq
                    | TokenKind::NotEq
                    | TokenKind::Lt
                    | TokenKind::Gt
                    | TokenKind::LtEq
                    | TokenKind::GtEq
                    | TokenKind::And2
                    | TokenKind::Or2
                    | TokenKind::Plus
                    | TokenKind::Minus
                    | TokenKind::Star
                    | TokenKind::Slash
            );

            if is_binary_op {
                // Exclude `**` (exponent): Star preceded by another Star (possibly with whitespace).
                // Skip *both* stars so neither triggers a spacing diagnostic.
                if tok.kind == TokenKind::Star {
                    // Check if this star is the second of a `**` pair
                    let prev_real = tokens[..i]
                        .iter()
                        .rev()
                        .find(|t| t.kind != TokenKind::Whitespace);
                    if prev_real.is_some_and(|t| t.kind == TokenKind::Star) {
                        i += 1;
                        continue;
                    }
                    // Check if this star is the first of a `**` pair
                    let next_real = tokens[i + 1..]
                        .iter()
                        .find(|t| t.kind != TokenKind::Whitespace);
                    if next_real.is_some_and(|t| t.kind == TokenKind::Star) {
                        i += 1;
                        continue;
                    }
                }

                let prev = &tokens[i - 1];
                let next = tokens.get(i + 1);

                let missing_before =
                    prev.kind != TokenKind::Whitespace && prev.kind != TokenKind::Newline;
                let missing_after = next.is_some_and(|n| {
                    n.kind != TokenKind::Whitespace && n.kind != TokenKind::Newline
                });

                if missing_before || missing_after {
                    let line_idx = tok.line.saturating_sub(1);
                    let line = ctx.lines.get(line_idx).copied().unwrap_or("");
                    let fixed = fix_operator_spacing(line);

                    if missing_before {
                        diags.push(
                            Diagnostic::new(
                                ctx.file,
                                tok.line,
                                tok.col,
                                "R021",
                                format!("Missing space before `{}`", tok.text),
                                Severity::Warning,
                            )
                            .with_fix(fixed.clone()),
                        );
                    }

                    if missing_after {
                        diags.push(
                            Diagnostic::new(
                                ctx.file,
                                tok.line,
                                tok.col + tok.text.len(),
                                "R021",
                                format!("Missing space after `{}`", tok.text),
                                Severity::Warning,
                            )
                            .with_fix(fixed),
                        );
                    }
                }
            }

            i += 1;
        }

        diags
    }
}

/// Heuristic: add missing spaces around binary operators in a line of Ruby code.
/// This handles simple cases like `x==y` → `x == y`.
///
/// Uses a single left-to-right pass, matching the longest operator first at each
/// position. This avoids the quadratic cost of repeated `is_inside_string` calls
/// and prevents multi-pass corruption of operators.
pub fn fix_operator_spacing(line: &str) -> String {
    // Operators to fix, ordered longest-first to avoid partial matches.
    let ops: &[&str] = &[
        "<=>", "==", "!=", "<=", ">=", "&&", "||", "<", ">", "+", "-", "*", "/",
    ];

    // Multi-character operators that should NOT be touched by the fixer.
    // If we see one of these at position i, skip past it without adding spaces.
    // This prevents corrupting `=>`, `**`, `<<`, `>>`, `+=`, `-=`, `*=`, `/=`.
    let skip_ops: &[&str] = &[
        "**", "=>", "<<", ">>", "+=", "-=", "*=", "/=", "%=", "&=", "|=",
    ];

    let mut out = String::with_capacity(line.len() + 8);
    let bytes = line.as_bytes();
    let mut i = 0; // byte index
    let mut in_string = false;
    let mut quote_char = b'"';
    let mut escaped = false;

    while i < bytes.len() {
        let b = bytes[i];

        // Helper: decode one UTF-8 char at position i and push it to out
        let push_char = |out: &mut String, pos: usize| -> usize {
            let ch = &line[pos..];
            if let Some(c) = ch.chars().next() {
                out.push(c);
                c.len_utf8()
            } else {
                1
            }
        };

        // Track string state
        if escaped {
            escaped = false;
            i += push_char(&mut out, i);
            continue;
        }
        if in_string {
            if b == b'\\' {
                escaped = true;
            } else if b == quote_char {
                in_string = false;
            }
            i += push_char(&mut out, i);
            continue;
        }
        if b == b'"' || b == b'\'' {
            in_string = true;
            quote_char = b;
            out.push(b as char);
            i += 1;
            continue;
        }

        // First, check if we're at a multi-char operator that should be skipped entirely
        // (e.g. =>, **, <<, >>). If so, copy it verbatim and advance past it.
        let mut skipped = false;
        for skip in skip_ops {
            let sb = skip.as_bytes();
            if i + sb.len() <= bytes.len() && &bytes[i..i + sb.len()] == sb {
                out.push_str(skip);
                i += sb.len();
                skipped = true;
                break;
            }
        }
        if skipped {
            continue;
        }

        // Try to match the longest operator at position i
        let mut matched_op: Option<&str> = None;
        for op in ops {
            let op_bytes = op.as_bytes();
            if i + op_bytes.len() <= bytes.len() && &bytes[i..i + op_bytes.len()] == op_bytes {
                matched_op = Some(op);
                break; // ops are longest-first, so first match is longest
            }
        }

        if let Some(op) = matched_op {
            let op_len = op.len();
            let before = if i > 0 { bytes[i - 1] } else { 0 };
            let after = if i + op_len < bytes.len() {
                bytes[i + op_len]
            } else {
                0
            };

            let need_space_before = before != b' ' && before != b'\t' && before != 0;
            let need_space_after = after != b' ' && after != b'\t' && after != 0 && after != b'\n';

            if need_space_before {
                out.push(' ');
            }
            out.push_str(op);
            if need_space_after {
                out.push(' ');
            }
            i += op_len;
        } else {
            // Non-ASCII safe: decode one UTF-8 character
            i += push_char(&mut out, i);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn check(source: &str) -> Vec<Diagnostic> {
        let lines: Vec<&str> = source.lines().collect();
        let tokens = Lexer::new(source).tokenize();
        let ctx = LintContext::new("test.rb", source, &lines, &tokens);
        OperatorSpacingRule.check(&ctx)
    }

    fn has_rule(diags: &[Diagnostic], rule: &str) -> bool {
        diags.iter().any(|d| d.rule == rule)
    }

    fn count_rule(diags: &[Diagnostic], rule: &str) -> usize {
        diags.iter().filter(|d| d.rule == rule).count()
    }

    fn fix_for_rule<'a>(diags: &'a [Diagnostic], rule: &str) -> Option<&'a str> {
        diags
            .iter()
            .find(|d| d.rule == rule && d.fix.is_some())
            .and_then(|d| d.fix.as_deref())
    }

    #[test]
    fn no_violation_spaced_operator() {
        let diags = check("x == y");
        assert!(!has_rule(&diags, "R021"));
    }

    #[test]
    fn violation_no_space_before_eq_eq() {
        let diags = check("x== y");
        assert!(has_rule(&diags, "R021"));
    }

    #[test]
    fn violation_no_space_after_eq_eq() {
        let diags = check("x ==y");
        assert!(has_rule(&diags, "R021"));
    }

    #[test]
    fn violation_no_space_around_plus() {
        let diags = check("a+b");
        assert_eq!(count_rule(&diags, "R021"), 2); // before and after
    }

    #[test]
    fn no_violation_newline_before_operator() {
        // Operator at start of continuation line is fine
        let diags = check("x\n== y");
        assert!(!has_rule(&diags, "R021"));
    }

    #[test]
    fn fix_operator_spacing_eq_eq() {
        let diags = check("x==y");
        let fix = fix_for_rule(&diags, "R021").expect("should have fix");
        assert_eq!(fix, "x == y");
    }
}
