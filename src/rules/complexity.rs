use crate::diagnostic::{Diagnostic, Severity};
use crate::lexer::TokenKind;
use super::{LintContext, Rule};

/// Complexity rules
pub struct ComplexityRule;

const MAX_METHOD_LINES: usize = 30;
const MAX_CLASS_LINES: usize = 300;

impl Rule for ComplexityRule {
    fn name(&self) -> &'static str { "R040" }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let tokens = ctx.tokens;

        // R040: Method too long
        let mut i = 0;
        while i < tokens.len() {
            if tokens[i].kind == TokenKind::Def {
                let def_line = tokens[i].line;
                // Find method name
                let name = tokens.iter().skip(i + 1)
                    .find(|t| !matches!(t.kind, TokenKind::Whitespace))
                    .map(|t| t.text.clone())
                    .unwrap_or_else(|| "<unknown>".into());

                // Find matching `end`
                let mut depth = 1usize;
                let mut j = i + 1;
                while j < tokens.len() && depth > 0 {
                    match tokens[j].kind {
                        TokenKind::Def | TokenKind::Class | TokenKind::Module
                        | TokenKind::Do | TokenKind::Begin
                        | TokenKind::If | TokenKind::Unless
                        | TokenKind::While | TokenKind::Until
                        | TokenKind::For => depth += 1,
                        TokenKind::End => depth -= 1,
                        _ => {}
                    }
                    j += 1;
                }

                if j <= tokens.len() {
                    let end_line = tokens[j.saturating_sub(1)].line;
                    let method_lines = end_line.saturating_sub(def_line) + 1;
                    if method_lines > MAX_METHOD_LINES {
                        diags.push(Diagnostic::new(
                            ctx.file,
                            def_line,
                            tokens[i].col,
                            "R040",
                            format!(
                                "Method `{}` is too long ({} lines, max {})",
                                name, method_lines, MAX_METHOD_LINES
                            ),
                            Severity::Warning,
                        ));
                    }
                }
            }
            i += 1;
        }

        // R041: Class too long
        let mut i = 0;
        while i < tokens.len() {
            if tokens[i].kind == TokenKind::Class {
                let class_line = tokens[i].line;
                let name = tokens.iter().skip(i + 1)
                    .find(|t| !matches!(t.kind, TokenKind::Whitespace))
                    .map(|t| t.text.clone())
                    .unwrap_or_else(|| "<unknown>".into());

                let mut depth = 1usize;
                let mut j = i + 1;
                while j < tokens.len() && depth > 0 {
                    match tokens[j].kind {
                        TokenKind::Def | TokenKind::Class | TokenKind::Module
                        | TokenKind::Do | TokenKind::Begin => depth += 1,
                        TokenKind::End => depth -= 1,
                        _ => {}
                    }
                    j += 1;
                }

                if j <= tokens.len() {
                    let end_line = tokens[j.saturating_sub(1)].line;
                    let class_lines = end_line.saturating_sub(class_line) + 1;
                    if class_lines > MAX_CLASS_LINES {
                        diags.push(Diagnostic::new(
                            ctx.file,
                            class_line,
                            tokens[i].col,
                            "R041",
                            format!(
                                "Class `{}` is too long ({} lines, max {})",
                                name, class_lines, MAX_CLASS_LINES
                            ),
                            Severity::Warning,
                        ));
                    }
                }
            }
            i += 1;
        }

        // R042: Cyclomatic complexity (count branching keywords)
        // Per method: count if/unless/elsif/while/until/for/rescue/when
        let mut i = 0;
        while i < tokens.len() {
            if tokens[i].kind == TokenKind::Def {
                let def_line = tokens[i].line;
                let name = tokens.iter().skip(i + 1)
                    .find(|t| !matches!(t.kind, TokenKind::Whitespace))
                    .map(|t| t.text.clone())
                    .unwrap_or_else(|| "<unknown>".into());

                let mut complexity = 1usize;
                let mut depth = 1usize;
                let mut j = i + 1;
                while j < tokens.len() && depth > 0 {
                    match tokens[j].kind {
                        TokenKind::Def | TokenKind::Class | TokenKind::Module
                        | TokenKind::Do | TokenKind::Begin => depth += 1,
                        TokenKind::End => depth -= 1,
                        TokenKind::If | TokenKind::Unless | TokenKind::Elsif
                        | TokenKind::While | TokenKind::Until | TokenKind::For
                        | TokenKind::Rescue | TokenKind::When if depth == 1 => {
                            complexity += 1;
                        }
                        TokenKind::And2 | TokenKind::Or2 => {
                            complexity += 1;
                        }
                        _ => {}
                    }
                    j += 1;
                }

                if complexity > 10 {
                    diags.push(Diagnostic::new(
                        ctx.file,
                        def_line,
                        tokens[i].col,
                        "R042",
                        format!(
                            "Method `{}` has high cyclomatic complexity ({})",
                            name, complexity
                        ),
                        Severity::Warning,
                    ));
                }
            }
            i += 1;
        }

        diags
    }
}
