use crate::diagnostic::Diagnostic;
use crate::lexer::Lexer;
use crate::rules::{all_rules, LintContext};

pub struct Linter {
    rules: Vec<Box<dyn crate::rules::Rule + Send + Sync>>,
}

impl Linter {
    pub fn new() -> Self {
        Linter { rules: all_rules() }
    }

    pub fn lint_file(&self, path: &str, source: &str) -> Vec<Diagnostic> {
        let lines: Vec<&str> = source.lines().collect();
        let tokens = Lexer::new(source).tokenize();

        let ctx = LintContext {
            file: path,
            source,
            lines: &lines,
            tokens: &tokens,
        };

        let mut diags: Vec<Diagnostic> = self.rules.iter()
            .flat_map(|rule| rule.check(&ctx))
            .collect();

        diags.sort_by(|a, b| a.line.cmp(&b.line).then(a.col.cmp(&b.col)));
        diags
    }
}
