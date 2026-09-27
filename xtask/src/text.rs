//! Small text helpers shared by the report summary lines.

/// `word`, pluralized with a trailing `s` unless `n` is exactly one.
pub(crate) fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        word.to_string()
    } else {
        format!("{word}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_is_singular_and_other_counts_are_plural() {
        assert_eq!(plural(0, "error"), "errors");
        assert_eq!(plural(1, "error"), "error");
        assert_eq!(plural(2, "error"), "errors");
    }
}
