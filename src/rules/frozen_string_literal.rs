use crate::diagnostic::{Diagnostic, Severity};
use super::{LintContext, Rule};

/// R003 - Files should have `# frozen_string_literal: true` magic comment
pub struct FrozenStringLiteralRule;

impl Rule for FrozenStringLiteralRule {
    fn name(&self) -> &'static str { "R003" }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        // Skip empty files
        if ctx.lines.is_empty() { return vec![]; }

        // Check first two lines (allow shebang on line 1)
        let has_frozen = ctx.lines.iter().take(3).any(|l| {
            l.contains("frozen_string_literal: true")
        });

        if !has_frozen {
            vec![Diagnostic::new(
                ctx.file,
                1,
                1,
                "R003",
                "Missing `# frozen_string_literal: true` magic comment",
                Severity::Warning,
            )
            .with_fix("# frozen_string_literal: true".to_string())]
        } else {
            vec![]
        }
    }
}
