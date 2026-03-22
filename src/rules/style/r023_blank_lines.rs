use crate::diagnostic::{Diagnostic, Severity};
use crate::rules::{LintContext, Rule};

pub struct BlankLinesRule;

impl Rule for BlankLinesRule {
    fn name(&self) -> &'static str {
        "R023"
    }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        // R023: Multiple empty lines (> 2 consecutive blank lines)
        // Fix: delete the excess blank line.
        let mut blank_count = 0usize;
        for (i, line) in ctx.lines.iter().enumerate() {
            if line.trim().is_empty() {
                blank_count += 1;
                if blank_count > 2 {
                    diags.push(
                        Diagnostic::new(
                            ctx.file,
                            i + 1,
                            1,
                            "R023",
                            "Too many consecutive blank lines (maximum 2)",
                            Severity::Warning,
                        )
                        .with_delete_line_fix(),
                    );
                }
            } else {
                blank_count = 0;
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
        BlankLinesRule.check(&ctx)
    }

    fn has_rule(diags: &[Diagnostic], rule: &str) -> bool {
        diags.iter().any(|d| d.rule == rule)
    }

    #[test]
    fn no_violation_two_blank_lines() {
        let diags = check("a = 1\n\n\nb = 2");
        assert!(!has_rule(&diags, "R023"));
    }

    #[test]
    fn violation_three_blank_lines() {
        let diags = check("a = 1\n\n\n\nb = 2");
        assert!(has_rule(&diags, "R023"), "{diags:?}");
    }

    #[test]
    fn fix_r023_deletes_excess_blank_line() {
        let diags = check("a = 1\n\n\n\nb = 2");
        let diag = diags.iter().find(|d| d.rule == "R023").expect("R023 diag");
        assert_eq!(diag.fix_kind, FixKind::DeleteLine);
        // The fix field is set (`"<delete line>"` marker for DeleteLine)
        assert!(diag.fix.is_some());
    }

    #[test]
    fn fix_r023_applied_removes_line() {
        use crate::fixer::apply_fixes;
        let source = "a = 1\n\n\n\nb = 2\n";
        let diags = check(source);
        let (fixed, count) = apply_fixes(source, &diags);
        assert!(count > 0);
        // After fix: at most 2 consecutive blank lines
        let blank_runs: Vec<usize> = {
            let mut runs = Vec::new();
            let mut run = 0usize;
            for line in fixed.lines() {
                if line.trim().is_empty() {
                    run += 1;
                } else {
                    runs.extend((run > 0).then_some(run));
                    run = 0;
                }
            }
            runs
        };
        assert!(blank_runs.iter().all(|&r| r <= 2));
    }
}
