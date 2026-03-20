use crate::diagnostic::{Diagnostic, Severity};
use colored::Colorize;

#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
    Github, // GitHub Actions annotation format
}

pub struct Reporter {
    pub format: OutputFormat,
    pub show_fixes: bool,
}

impl Reporter {
    pub fn print(&self, diags: &[Diagnostic]) {
        match self.format {
            OutputFormat::Text => self.print_text(diags),
            OutputFormat::Json => self.print_json(diags),
            OutputFormat::Github => self.print_github(diags),
        }
    }

    fn print_text(&self, diags: &[Diagnostic]) {
        let mut current_file = "";
        for d in diags {
            if d.file != current_file {
                println!("\n{}", d.file.bold().underline());
                current_file = &d.file;
            }

            let loc = format!("{}:{}", d.line, d.col);
            let rule = format!("[{}]", d.rule).dimmed();
            let msg = match d.severity {
                Severity::Error => d.message.as_str().red().bold().to_string(),
                Severity::Warning => d.message.as_str().yellow().to_string(),
                Severity::Info => d.message.as_str().cyan().to_string(),
            };
            let sev = match d.severity {
                Severity::Error => "error  ".red().bold().to_string(),
                Severity::Warning => "warning".yellow().to_string(),
                Severity::Info => "info   ".cyan().to_string(),
            };

            println!("  {} {} {} {}", loc.dimmed(), sev, msg, rule);

            if self.show_fixes {
                if let Some(fix) = &d.fix {
                    println!("    {} {}", "fix:".green().bold(), fix.dimmed());
                }
            }
        }
    }

    fn print_json(&self, diags: &[Diagnostic]) {
        let json = serde_json::to_string_pretty(diags).unwrap_or_else(|_| "[]".to_string());
        println!("{}", json);
    }

    fn print_github(&self, diags: &[Diagnostic]) {
        for d in diags {
            let level = match d.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "notice",
            };
            println!(
                "::{} file={},line={},col={},title={}::{}",
                level, d.file, d.line, d.col, d.rule, d.message
            );
        }
    }

    pub fn print_summary(&self, diags: &[Diagnostic], files_checked: usize, elapsed_ms: u128) {
        if self.format != OutputFormat::Text {
            return;
        }

        let errors = diags
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count();
        let warnings = diags
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count();
        let infos = diags
            .iter()
            .filter(|d| d.severity == Severity::Info)
            .count();

        println!();
        println!(
            "{} {} {} in {} {} ({} ms)",
            format!("{} error{}", errors, if errors == 1 { "" } else { "s" })
                .red()
                .bold(),
            format!(
                "{} warning{}",
                warnings,
                if warnings == 1 { "" } else { "s" }
            )
            .yellow(),
            format!("{} info", infos).cyan(),
            files_checked.to_string().bold(),
            if files_checked == 1 { "file" } else { "files" },
            elapsed_ms,
        );

        if diags.is_empty() {
            println!("{}", "All checks passed!".green().bold());
        }
    }
}
