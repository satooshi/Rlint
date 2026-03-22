use crate::diagnostic::{Diagnostic, Severity};
use crate::rules::{LintContext, Rule};

pub struct FinalNewlineRule;

impl Rule for FinalNewlineRule {
    fn name(&self) -> &'static str {
        "R025"
    }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        // R025: Missing final newline
        if !ctx.source.is_empty() && !ctx.source.ends_with('\n') {
            let last_line = ctx.lines.len();
            let last_line_content = ctx.lines.last().copied().unwrap_or("").to_string();
            // Preserve the file's existing line ending style.
            // Detect CRLF by finding the first \n in the source and checking if it's
            // preceded by \r. This avoids false-positives from literal \r\n in strings
            // (which would not appear at a raw line boundary).
            let line_ending = {
                let bytes = ctx.source.as_bytes();
                let has_crlf = bytes
                    .iter()
                    .position(|&b| b == b'\n')
                    .is_some_and(|pos| pos > 0 && bytes[pos - 1] == b'\r');
                if has_crlf {
                    "\r\n"
                } else {
                    "\n"
                }
            };
            let fixed = format!("{}{}", last_line_content, line_ending);
            diags.push(
                Diagnostic::new(
                    ctx.file,
                    last_line,
                    1,
                    "R025",
                    "Missing final newline at end of file",
                    Severity::Warning,
                )
                .with_fix(fixed),
            );
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
        FinalNewlineRule.check(&ctx)
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
    fn violation_missing_final_newline() {
        let diags = check("x = 1");
        assert!(has_rule(&diags, "R025"), "{diags:?}");
    }

    #[test]
    fn no_violation_has_final_newline() {
        let diags = check("x = 1\n");
        assert!(!has_rule(&diags, "R025"));
    }

    #[test]
    fn no_violation_empty_source() {
        let diags = check("");
        assert!(!has_rule(&diags, "R025"));
    }

    #[test]
    fn fix_r025_appends_newline() {
        let diags = check("x = 1");
        let fix = fix_for_rule(&diags, "R025").expect("should have fix");
        assert!(fix.ends_with('\n'), "fix should end with newline: {fix:?}");
        assert!(fix.starts_with("x = 1"));
    }

    #[test]
    fn fix_r025_applied() {
        use crate::fixer::apply_fixes;
        let source = "x = 1";
        let diags = check(source);
        let (fixed, count) = apply_fixes(source, &diags);
        assert!(count > 0);
        assert!(fixed.ends_with('\n'));
    }
}
