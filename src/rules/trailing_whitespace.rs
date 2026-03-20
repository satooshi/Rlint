use crate::diagnostic::{Diagnostic, Severity};
use super::{LintContext, Rule};

/// R002 - No trailing whitespace
pub struct TrailingWhitespaceRule;

impl Rule for TrailingWhitespaceRule {
    fn name(&self) -> &'static str { "R002" }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, line) in ctx.lines.iter().enumerate() {
            if line.ends_with(' ') || line.ends_with('\t') {
                let trailing = line.len() - line.trim_end().len();
                diags.push(
                    Diagnostic::new(
                        ctx.file,
                        i + 1,
                        line.trim_end().len() + 1,
                        "R002",
                        format!("Trailing whitespace ({} character{})", trailing, if trailing == 1 { "" } else { "s" }),
                        Severity::Warning,
                    )
                    .with_fix(line.trim_end().to_string()),
                );
            }
        }
        diags
    }
}
