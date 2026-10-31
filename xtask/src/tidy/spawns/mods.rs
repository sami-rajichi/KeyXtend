//! F4's reader of test-only modules: a `#[cfg(test)]` or `#[cfg(all(test, ...))]` module, inline or in a file of its
//! own, read from the tokens of scanned code.

use std::ops::RangeInclusive;

use crate::tidy::bridges::part;

/// The attribute that builds an item only under a condition.
const CFG: &str = "cfg";
/// The condition of test builds.
const TEST: &str = "test";
/// The condition list under which `test` still means test builds only.
const ALL: &str = "all";
/// The keyword that declares a module.
const MOD: &str = "mod";
/// The keyword that opens a visibility.
const PUB: &str = "pub";

/// The test-only modules of one file.
#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Mods {
    /// Their names; each one's file and folder hold test code, even beside an inline module.
    pub names: Vec<String>,
    /// Line spans, from 0, of those written inline.
    pub spans: Vec<RangeInclusive<usize>>,
}

/// A name or one other character of scanned code, with its line from 0.
#[derive(Debug, Clone, Copy)]
struct Token<'a> {
    text: &'a str,
    line: usize,
}

/// The test-only modules declared in `code`.
pub(super) fn test_mods(code: &[String]) -> Mods {
    let tokens = tokens(code);
    let mut mods = Mods::default();
    let mut i = 0;
    while i < tokens.len() {
        i = match test_attr(&tokens, i) {
            Some(next) => {
                take_mod(&tokens, next, &mut mods);
                next
            }
            None => i + 1,
        };
    }
    mods
}

/// The tokens of `code`, without spaces.
fn tokens(code: &[String]) -> Vec<Token<'_>> {
    let mut out = Vec::new();
    for (line, text) in code.iter().enumerate() {
        let mut rest = text.as_str();
        while let Some(c) = rest.chars().next() {
            let len = if part(c) {
                rest.find(|d: char| !part(d)).unwrap_or(rest.len())
            } else {
                c.len_utf8()
            };
            if !c.is_whitespace() {
                out.push(Token {
                    text: &rest[..len],
                    line,
                });
            }
            rest = &rest[len..];
        }
    }
    out
}

/// Where the code after a test-only `#[cfg(...)]` at `i` goes on; `None` for any other token or attribute.
fn test_attr(tokens: &[Token<'_>], i: usize) -> Option<usize> {
    let text = |at: usize| tokens.get(at).map(|t| t.text);
    if (text(i)?, text(i + 1)?, text(i + 2)?, text(i + 3)?) != ("#", "[", CFG, "(") {
        return None;
    }
    let close = matching(tokens, i + 3)?;
    let only = test_only(&tokens[i + 4..close]) && text(close + 1) == Some("]");
    only.then_some(close + 2)
}

/// True when a cfg condition holds in test builds only: `test`, or `all(...)` with `test` among its own items.
fn test_only(cond: &[Token<'_>]) -> bool {
    match cond {
        [one] => one.text == TEST,
        [all, open, items @ .., close]
            if all.text == ALL && open.text == "(" && close.text == ")" =>
        {
            let mut depth = 0usize;
            items.iter().any(|t| {
                match t.text {
                    "(" => depth += 1,
                    ")" => depth = depth.saturating_sub(1),
                    _ => {}
                }
                depth == 0 && t.text == TEST
            })
        }
        _ => false,
    }
}

/// Notes the module declared at `i`, past attributes and a visibility, if one is.
fn take_mod(tokens: &[Token<'_>], i: usize, mods: &mut Mods) {
    let i = skip_prefix(tokens, i);
    let text = |at: usize| tokens.get(at).map(|t| t.text);
    if text(i) != Some(MOD) {
        return;
    }
    let Some(name) = text(i + 1).filter(|n| n.starts_with(part)) else {
        return;
    };
    match text(i + 2) {
        Some(";") => mods.names.push(name.to_string()),
        Some("{") => {
            if let Some(close) = matching(tokens, i + 2) {
                mods.names.push(name.to_string());
                mods.spans.push(tokens[i + 2].line..=tokens[close].line);
            }
        }
        _ => {}
    }
}

/// The first token at or after `i` past outer attributes and a visibility such as `pub(crate)`.
fn skip_prefix(tokens: &[Token<'_>], mut i: usize) -> usize {
    loop {
        let text = |at: usize| tokens.get(at).map(|t| t.text);
        match (text(i), text(i + 1)) {
            (Some("#"), Some("[")) | (Some(PUB), Some("(")) => match matching(tokens, i + 1) {
                Some(close) => i = close + 1,
                None => return i,
            },
            (Some(PUB), _) => i += 1,
            _ => return i,
        }
    }
}

/// The token that closes the bracket at `open`; `None` if none does.
fn matching(tokens: &[Token<'_>], open: usize) -> Option<usize> {
    let (o, c) = match tokens.get(open)?.text {
        "(" => ("(", ")"),
        "[" => ("[", "]"),
        "{" => ("{", "}"),
        _ => return None,
    };
    let mut depth = 0usize;
    for (at, t) in tokens.iter().enumerate().skip(open) {
        if t.text == o {
            depth += 1;
        } else if t.text == c {
            depth -= 1;
            if depth == 0 {
                return Some(at);
            }
        }
    }
    None
}
