//! The target-window log format, shared by its writer (target-window) and its reader (the harness).

/// Starts comment lines; data lines are `<µs>\t<UTF-16 unit in hex>`.
pub const COMMENT: &str = "#";
/// First line of every run: `# start\t<µs>`.
pub const START: &str = "# start";
/// The text box lost the keyboard focus: `# focus-lost\t<µs>`.
pub const FOCUS_LOST: &str = "# focus-lost";
/// Field separator.
pub const SEP: char = '\t';

/// A data line for UTF-16 `unit` received at `us`.
pub fn data(us: i64, unit: u16) -> String {
    format!("{us}{SEP}{unit:04X}")
}

/// A comment line `mark` at `us`.
pub fn mark(mark: &str, us: i64) -> String {
    format!("{mark}{SEP}{us}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_have_the_documented_shape() {
        assert_eq!(data(12, 0x0644), "12\t0644");
        assert_eq!(mark(FOCUS_LOST, 7), "# focus-lost\t7");
        assert!(START.starts_with(COMMENT) && FOCUS_LOST.starts_with(COMMENT));
    }
}
