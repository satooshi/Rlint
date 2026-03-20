use crate::diagnostic::{Diagnostic, Severity};
use super::{LintContext, Rule};

/// R001 - Lines should not exceed max_length characters
pub struct LineLengthRule {
    pub max_length: usize,
}

impl Default for LineLengthRule {
    fn default() -> Self {
        LineLengthRule { max_length: 120 }
    }
}

impl Rule for LineLengthRule {
    fn name(&self) -> &'static str { "R001" }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, line) in ctx.lines.iter().enumerate() {
            let len = line.chars().count();
            if len > self.max_length {
                diags.push(Diagnostic::new(
                    ctx.file,
                    i + 1,
                    self.max_length + 1,
                    "R001",
                    format!("Line too long ({} > {} characters)", len, self.max_length),
                    Severity::Warning,
                ));
            }
        }
        diags
    }
}
