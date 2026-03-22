use crate::diagnostic::{Diagnostic, Severity};
use crate::lexer::TokenKind;
use crate::rules::{LintContext, Rule};

/// R033 - Redundant `self.` on method calls inside instance methods
pub struct RedundantSelfRule;

impl Rule for RedundantSelfRule {
    fn name(&self) -> &'static str {
        "R033"
    }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let tokens = ctx.tokens;

        // Track whether we are inside an instance method body
        // (i.e. inside a `def` that is NOT `def self.foo`)
        let mut method_depth = 0usize;
        let mut block_depth = 0usize; // overall block depth for nesting

        let mut i = 0;
        while i < tokens.len() {
            match &tokens[i].kind {
                TokenKind::Def => {
                    // Check if this is `def self.name` (class method) — skip those
                    let mut j = i + 1;
                    while j < tokens.len() && tokens[j].kind == TokenKind::Whitespace {
                        j += 1;
                    }
                    let is_class_method = j < tokens.len()
                        && tokens[j].kind == TokenKind::Self_
                        && tokens.get(j + 1).map(|t| t.kind == TokenKind::Dot) == Some(true);

                    block_depth += 1;
                    if !is_class_method {
                        method_depth += 1;
                    }
                }
                TokenKind::Class | TokenKind::Module | TokenKind::Do | TokenKind::Begin => {
                    block_depth += 1;
                }
                TokenKind::If
                | TokenKind::Unless
                | TokenKind::While
                | TokenKind::Until
                | TokenKind::For => {
                    // Only push if block form (at statement start)
                    let prev_non_ws = (0..i)
                        .rev()
                        .find(|&j| tokens[j].kind != TokenKind::Whitespace)
                        .map(|j| &tokens[j]);
                    let at_statement_start = match prev_non_ws {
                        None => true,
                        Some(p) => matches!(p.kind, TokenKind::Newline),
                    };
                    if at_statement_start {
                        block_depth += 1;
                    }
                }
                TokenKind::End => {
                    block_depth = block_depth.saturating_sub(1);
                    if method_depth > 0 && block_depth < method_depth {
                        method_depth -= 1;
                    }
                }
                TokenKind::Self_ => {
                    // Only flag inside instance method bodies
                    if method_depth == 0 {
                        i += 1;
                        continue;
                    }

                    // Check for `self.method_name` pattern
                    let j = i + 1;
                    if j >= tokens.len() || tokens[j].kind != TokenKind::Dot {
                        i += 1;
                        continue;
                    }

                    let k = j + 1;
                    if k >= tokens.len() {
                        i += 1;
                        continue;
                    }

                    let name_tok = &tokens[k];
                    // Skip if it's an assignment (setter): `self.attr = value`
                    if name_tok.kind == TokenKind::Ident || name_tok.kind == TokenKind::Constant {
                        // Check next non-whitespace token after name — if it's `=` (but not `==`),
                        // it's a setter method call which requires `self.`
                        let after_name = (k + 1..tokens.len())
                            .find(|&idx| tokens[idx].kind != TokenKind::Whitespace)
                            .unwrap_or(tokens.len());
                        let is_setter =
                            after_name < tokens.len() && tokens[after_name].kind == TokenKind::Eq;

                        if !is_setter {
                            // Build the fix: remove `self.` from the line
                            let line_text = ctx
                                .lines
                                .get(tokens[i].line.saturating_sub(1))
                                .copied()
                                .unwrap_or("");
                            let fix = line_text.replacen("self.", "", 1);

                            diags.push(
                                Diagnostic::new(
                                    ctx.file,
                                    tokens[i].line,
                                    tokens[i].col,
                                    "R033",
                                    format!(
                                        "Redundant `self.` — `self.{}` can be written as `{}`",
                                        name_tok.text, name_tok.text
                                    ),
                                    Severity::Warning,
                                )
                                .with_fix(fix),
                            );
                        }
                    }
                }
                _ => {}
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
        RedundantSelfRule.check(&ctx)
    }

    fn has_rule(diags: &[Diagnostic], rule: &str) -> bool {
        diags.iter().any(|d| d.rule == rule)
    }

    #[test]
    fn violation_redundant_self_method_call() {
        let src = "def foo\n  self.bar\nend\n";
        assert!(has_rule(&check(src), "R033"), "{:?}", check(src));
    }

    #[test]
    fn no_violation_self_setter() {
        // `self.name = value` is needed for setter methods
        let src = "def foo\n  self.name = 'value'\nend\n";
        assert!(!has_rule(&check(src), "R033"));
    }

    #[test]
    fn no_violation_outside_method() {
        // `self.foo` at class level is needed
        let src = "class MyClass\n  self.foo\nend\n";
        assert!(!has_rule(&check(src), "R033"));
    }

    #[test]
    fn fix_removes_self_dot() {
        let src = "def foo\n  self.bar\nend\n";
        let diags = check(src);
        let d = diags.iter().find(|d| d.rule == "R033").unwrap();
        let fix = d.fix.as_deref().unwrap_or("");
        assert!(fix.contains("bar") && !fix.contains("self."), "fix: {fix}");
    }
}
