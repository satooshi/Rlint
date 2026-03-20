use super::{LintContext, Rule};
use crate::diagnostic::{Diagnostic, Severity};

/// R003 - Files should have `# frozen_string_literal: true` magic comment
pub struct FrozenStringLiteralRule;

impl Rule for FrozenStringLiteralRule {
    fn name(&self) -> &'static str {
        "R003"
    }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        // Skip empty files
        if ctx.lines.is_empty() {
            return vec![];
        }

        // Check first two lines (allow shebang on line 1)
        let has_frozen = ctx
            .lines
            .iter()
            .take(3)
            .any(|l| l.contains("frozen_string_literal: true"));

        if !has_frozen {
            // If line 1 is a shebang, insert after it (before line 2) to avoid
            // moving the shebang off the first line and breaking executability.
            let has_shebang = ctx.lines.first().is_some_and(|l| l.starts_with("#!"));
            let insert_line = if has_shebang { 2 } else { 1 };
            vec![Diagnostic::new(
                ctx.file,
                insert_line,
                1,
                "R003",
                "Missing `# frozen_string_literal: true` magic comment",
                Severity::Warning,
            )
            .with_insert_before_fix("# frozen_string_literal: true")]
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn check(source: &str) -> Vec<Diagnostic> {
        let lines: Vec<&str> = source.lines().collect();
        let tokens = Lexer::new(source).tokenize();
        let ctx = LintContext {
            file: "test.rb",
            source,
            lines: &lines,
            tokens: &tokens,
        };
        FrozenStringLiteralRule.check(&ctx)
    }

    #[test]
    fn no_violation_magic_comment_first_line() {
        assert!(check("# frozen_string_literal: true\nx = 1").is_empty());
    }

    #[test]
    fn no_violation_shebang_plus_magic_comment() {
        let src = "#!/usr/bin/env ruby\n# frozen_string_literal: true\nx = 1";
        assert!(check(src).is_empty());
    }

    #[test]
    fn no_violation_magic_comment_on_line_3() {
        let src = "# encoding: utf-8\n# typed: strict\n# frozen_string_literal: true";
        assert!(check(src).is_empty());
    }

    #[test]
    fn violation_no_magic_comment() {
        let diags = check("x = 1\nputs x");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule, "R003");
        assert_eq!(diags[0].line, 1);
    }

    #[test]
    fn violation_fix_is_magic_comment() {
        let diags = check("x = 1");
        assert_eq!(
            diags[0].fix.as_deref(),
            Some("# frozen_string_literal: true")
        );
    }

    #[test]
    fn no_violation_empty_file() {
        assert!(check("").is_empty());
    }

    #[test]
    fn violation_magic_comment_on_line_4() {
        let src = "# a\n# b\n# c\n# frozen_string_literal: true";
        assert_eq!(check(src).len(), 1);
    }

    #[test]
    fn violation_shebang_inserts_after_shebang() {
        // File has shebang but no frozen comment; fix should target line 2
        // so the shebang stays on line 1.
        let src = "#!/usr/bin/env ruby\nx = 1\n";
        let diags = check(src);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule, "R003");
        assert_eq!(
            diags[0].line, 2,
            "fix insertion point should be after the shebang"
        );
        assert_eq!(
            diags[0].fix.as_deref(),
            Some("# frozen_string_literal: true")
        );
    }
}
