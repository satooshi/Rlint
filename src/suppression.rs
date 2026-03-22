use crate::diagnostic::Diagnostic;
use crate::lexer::{Token, TokenKind};
use crate::linter::parse_rule_list;

/// Suppression entry: (optional rule list, start line, end line inclusive)
pub(crate) struct Suppression {
    rules: Option<Vec<String>>,
    start: usize,
    end: usize,
}

impl Suppression {
    fn suppresses(&self, diag: &Diagnostic) -> bool {
        if diag.line < self.start || diag.line > self.end {
            return false;
        }
        match &self.rules {
            None => true,
            Some(rs) => rs.iter().any(|r| diag.rule.starts_with(r.as_str())),
        }
    }
}

pub(crate) fn apply_suppressions(diags: &mut Vec<Diagnostic>, tokens: &[Token]) {
    // Fast path: skip full parsing when no rlint directives exist in the file.
    let has_directives = tokens
        .iter()
        .any(|t| t.kind == TokenKind::Comment && t.text.contains("rlint:"));
    if !has_directives {
        return;
    }

    let mut suppressions: Vec<Suppression> = Vec::new();
    // Active disable block: (rules, start_line)
    let mut active: Option<(Option<Vec<String>>, usize)> = None;

    for token in tokens {
        if token.kind != TokenKind::Comment {
            continue;
        }
        // Strip leading `#` and whitespace
        let text = token.text.trim_start_matches('#').trim();

        if let Some(rest) = text.strip_prefix("rlint:disable-next-line") {
            let rules = parse_rule_list(rest.trim());
            let next_line = token.line + 1;
            suppressions.push(Suppression {
                rules,
                start: next_line,
                end: next_line,
            });
        } else if let Some(rest) = text.strip_prefix("rlint:enable") {
            let enable_rules = parse_rule_list(rest.trim());
            if let Some((active_rules, start)) = active.take() {
                let end = token.line.saturating_sub(1);
                match (enable_rules, active_rules) {
                    // rlint:enable (no rules) → close the entire active block
                    (None, ar) => {
                        suppressions.push(Suppression {
                            rules: ar,
                            start,
                            end,
                        });
                    }
                    // rlint:enable Rxx after rlint:disable (all rules): close the
                    // entire block. The current rule structure cannot represent
                    // "suppress all except Rxx", so a targeted enable after a global
                    // disable re-enables all rules. Document this in user-facing help.
                    (Some(_), None) => {
                        suppressions.push(Suppression {
                            rules: None,
                            start,
                            end,
                        });
                    }
                    // rlint:enable R001 with rlint:disable R001,R002 → partial close
                    (Some(en_rules), Some(ac_rules)) => {
                        // Close the entire original block up to the enable line.
                        suppressions.push(Suppression {
                            rules: Some(ac_rules.clone()),
                            start,
                            end,
                        });
                        // Re-open only the rules that were NOT enabled.
                        // Use the same prefix semantics as Suppression::suppresses():
                        // an ac_rule is considered enabled if any en_rule is a prefix of it.
                        let remaining: Vec<String> = ac_rules
                            .iter()
                            .filter(|r| !en_rules.iter().any(|en| r.starts_with(en.as_str())))
                            .cloned()
                            .collect();
                        if !remaining.is_empty() {
                            active = Some((Some(remaining), token.line + 1));
                        }
                    }
                }
            }
        } else if let Some(rest) = text.strip_prefix("rlint:disable") {
            // Close any previously open block before starting a new one
            if let Some((rules, start)) = active.take() {
                suppressions.push(Suppression {
                    rules,
                    start,
                    end: token.line.saturating_sub(1),
                });
            }
            let rules = parse_rule_list(rest.trim());
            active = Some((rules, token.line));
        }
    }

    // Close any still-open suppression at end of file
    if let Some((rules, start)) = active {
        suppressions.push(Suppression {
            rules,
            start,
            end: usize::MAX,
        });
    }

    if suppressions.is_empty() {
        return;
    }

    diags.retain(|d| !suppressions.iter().any(|s| s.suppresses(d)));
}
