use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error   => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info    => write!(f, "info"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub rule: &'static str,
    pub message: String,
    pub severity: Severity,
    /// Optional auto-fix suggestion
    pub fix: Option<String>,
}

impl Diagnostic {
    pub fn new(
        file: impl Into<String>,
        line: usize,
        col: usize,
        rule: &'static str,
        message: impl Into<String>,
        severity: Severity,
    ) -> Self {
        Diagnostic {
            file: file.into(),
            line,
            col,
            rule,
            message: message.into(),
            severity,
            fix: None,
        }
    }

    pub fn with_fix(mut self, fix: impl Into<String>) -> Self {
        self.fix = Some(fix.into());
        self
    }
}
