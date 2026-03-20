use crate::diagnostic::{Diagnostic, Severity};
use crate::lexer::TokenKind;
use super::{LintContext, Rule};

/// Syntax-level rules
pub struct SyntaxRule;

impl Rule for SyntaxRule {
    fn name(&self) -> &'static str { "R030" }

    fn check(&self, ctx: &LintContext<'_>) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let tokens = ctx.tokens;

        // R030: Unbalanced brackets/parens/braces
        let mut paren_stack: Vec<(char, usize, usize)> = Vec::new();
        for tok in tokens {
            match tok.kind {
                TokenKind::LParen   => paren_stack.push(('(', tok.line, tok.col)),
                TokenKind::LBracket => paren_stack.push(('[', tok.line, tok.col)),
                TokenKind::LBrace   => paren_stack.push(('{', tok.line, tok.col)),
                TokenKind::RParen => {
                    if paren_stack.last().map(|p| p.0) == Some('(') {
                        paren_stack.pop();
                    } else {
                        diags.push(Diagnostic::new(
                            ctx.file, tok.line, tok.col, "R030",
                            "Unexpected `)` — no matching `(`",
                            Severity::Error,
                        ));
                    }
                }
                TokenKind::RBracket => {
                    if paren_stack.last().map(|p| p.0) == Some('[') {
                        paren_stack.pop();
                    } else {
                        diags.push(Diagnostic::new(
                            ctx.file, tok.line, tok.col, "R030",
                            "Unexpected `]` — no matching `[`",
                            Severity::Error,
                        ));
                    }
                }
                TokenKind::RBrace => {
                    if paren_stack.last().map(|p| p.0) == Some('{') {
                        paren_stack.pop();
                    } else {
                        diags.push(Diagnostic::new(
                            ctx.file, tok.line, tok.col, "R030",
                            "Unexpected `}` — no matching `{`",
                            Severity::Error,
                        ));
                    }
                }
                _ => {}
            }
        }
        for (ch, line, col) in paren_stack {
            diags.push(Diagnostic::new(
                ctx.file, line, col, "R030",
                format!("Unclosed `{ch}` — missing closing bracket"),
                Severity::Error,
            ));
        }

        // R031: `end` without matching `def`/`class`/`module`/`do`/`if`
        let mut block_stack: Vec<(&'static str, usize, usize)> = Vec::new();
        let mut i = 0;
        while i < tokens.len() {
            let tok = &tokens[i];
            match tok.kind {
                TokenKind::Def | TokenKind::Class | TokenKind::Module
                | TokenKind::Do | TokenKind::Begin => {
                    let label: &'static str = match tok.kind {
                        TokenKind::Def    => "def",
                        TokenKind::Class  => "class",
                        TokenKind::Module => "module",
                        TokenKind::Do     => "do",
                        TokenKind::Begin  => "begin",
                        _ => unreachable!(),
                    };
                    block_stack.push((label, tok.line, tok.col));
                }
                // inline if/unless don't need `end` — only block form does
                // This is a heuristic: if `if` is at start of expression (preceded by newline/nothing)
                TokenKind::If | TokenKind::Unless | TokenKind::While
                | TokenKind::Until | TokenKind::For => {
                    // Block form: the previous non-whitespace token is a newline (or start of file)
                    let prev_non_ws = (0..i).rev()
                        .find(|&j| tokens[j].kind != TokenKind::Whitespace)
                        .map(|j| &tokens[j]);
                    let at_statement_start = match prev_non_ws {
                        None => true,
                        Some(p) => matches!(p.kind, TokenKind::Newline),
                    };
                    if at_statement_start {
                        let label: &'static str = match tok.kind {
                            TokenKind::If      => "if",
                            TokenKind::Unless  => "unless",
                            TokenKind::While   => "while",
                            TokenKind::Until   => "until",
                            TokenKind::For     => "for",
                            _ => unreachable!(),
                        };
                        block_stack.push((label, tok.line, tok.col));
                    }
                }
                TokenKind::End => {
                    if block_stack.pop().is_none() {
                        diags.push(Diagnostic::new(
                            ctx.file, tok.line, tok.col, "R031",
                            "Unexpected `end` — no matching block opener",
                            Severity::Error,
                        ));
                    }
                }
                _ => {}
            }
            i += 1;
        }
        for (label, line, col) in block_stack {
            diags.push(Diagnostic::new(
                ctx.file, line, col, "R031",
                format!("Missing `end` for `{label}` block"),
                Severity::Error,
            ));
        }

        // R032: Redundant `return` on last line of method
        // Heuristic: `return expr` immediately before `end`
        let mut i = 0;
        while i < tokens.len() {
            if tokens[i].kind == TokenKind::Return {
                // Look ahead: skip whitespace/newline, find `end`
                let mut j = i + 1;
                while j < tokens.len() && matches!(tokens[j].kind, TokenKind::Whitespace | TokenKind::Newline) {
                    j += 1;
                }
                // Skip the return expression (everything until newline)
                while j < tokens.len() && tokens[j].kind != TokenKind::Newline {
                    j += 1;
                }
                // Skip more blank lines
                while j < tokens.len() && matches!(tokens[j].kind, TokenKind::Whitespace | TokenKind::Newline) {
                    j += 1;
                }
                if j < tokens.len() && tokens[j].kind == TokenKind::End {
                    diags.push(Diagnostic::new(
                        ctx.file,
                        tokens[i].line,
                        tokens[i].col,
                        "R032",
                        "Redundant `return` on last line of method (Ruby returns the last expression implicitly)",
                        Severity::Info,
                    ));
                }
            }
            i += 1;
        }

        diags
    }
}
