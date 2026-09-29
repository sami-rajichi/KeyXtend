//! G1 comparison: sent and received text, character by character.
#![cfg(windows)]

use serde::Serialize;

/// Sent and received text, compared character by character.
#[derive(Debug, Clone, Serialize)]
pub struct Diff {
    /// Characters sent.
    pub sent_len: usize,
    /// Characters received.
    pub received_len: usize,
    /// True when both texts are the same.
    pub equal: bool,
    /// Index of the first differing character.
    pub first_diff: Option<usize>,
    /// Sent code point at the first difference (`U+XXXX`, empty past the end).
    pub sent_at: String,
    /// Received code point at the first difference (`U+XXXX`, empty past the end).
    pub received_at: String,
    /// Sent characters around the first difference.
    pub sent_context: String,
    /// Received characters around the first difference.
    pub received_context: String,
    /// Positions that differ over the shorter length.
    pub mismatched: usize,
}

/// Compares `sent` and `received` exactly, with `context` characters each side.
pub fn compare(sent: &str, received: &str, context: usize) -> Diff {
    let (a, b): (Vec<char>, Vec<char>) = (sent.chars().collect(), received.chars().collect());
    let shared = a.len().min(b.len());
    let first = (0..shared)
        .find(|&i| a[i] != b[i])
        .or((a.len() != b.len()).then_some(shared));
    let around = |s: &[char], i: usize| -> String {
        let end = (i + context + 1).min(s.len());
        s[i.saturating_sub(context).min(end)..end].iter().collect()
    };
    let code = |s: &[char], i: usize| s.get(i).map(|c| format!("U+{:04X}", u32::from(*c)));
    Diff {
        sent_len: a.len(),
        received_len: b.len(),
        equal: first.is_none(),
        first_diff: first,
        sent_at: first.and_then(|i| code(&a, i)).unwrap_or_default(),
        received_at: first.and_then(|i| code(&b, i)).unwrap_or_default(),
        sent_context: first.map(|i| around(&a, i)).unwrap_or_default(),
        received_context: first.map(|i| around(&b, i)).unwrap_or_default(),
        mismatched: (0..shared).filter(|&i| a[i] != b[i]).count(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_finds_the_first_difference() {
        let d = compare("abcdef", "abXdef", 1);
        assert!(!d.equal);
        assert_eq!(d.first_diff, Some(2));
        assert_eq!(
            (d.sent_at.as_str(), d.received_at.as_str()),
            ("U+0063", "U+0058")
        );
        assert_eq!(
            (d.sent_context.as_str(), d.received_context.as_str()),
            ("bcd", "bXd")
        );
        assert_eq!(d.mismatched, 1);
    }

    #[test]
    fn compare_handles_equal_and_short_text() {
        assert!(compare("لا€", "لا€", 10).equal);
        let d = compare("abc", "ab", 10);
        assert_eq!((d.first_diff, d.sent_len, d.received_len), (Some(2), 3, 2));
        assert_eq!((d.sent_at.as_str(), d.received_at.as_str()), ("U+0063", ""));
        assert_eq!(d.received_context, "ab");
    }
}
