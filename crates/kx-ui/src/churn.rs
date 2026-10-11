//! Debug builds only: with `KX_UI_CHURN` set, the keyboard window churns through its own buttons' calls
//! (`qml/Churn.qml`), so a memory check can measure after it with no mouse and no key sent.

use serde::Serialize;
use std::ffi::OsStr;

/// The variable that turns the churn on.
pub const CHURN_VAR: &str = "KX_UI_CHURN";

/// The churn: ms between steps, then how many theme switches, Caps Lock changes and size changes, in that order.
#[derive(Serialize)]
struct Plan {
    step_ms: u32,
    themes: u32,
    caps: u32,
    sizes: u32,
}

/// G8's churn with a theme round and Caps Lock: the themes come back to the first, Caps Lock to off, and the sizes go
/// up by half of them and down again.
const PLAN: Plan = Plan {
    step_ms: 400,
    themes: 3,
    caps: 20,
    sizes: 24,
};

/// The plan as JSON when `value`, the variable's, is set and not empty; else an empty string.
pub fn plan(value: Option<&OsStr>) -> String {
    match value {
        Some(v) if !v.is_empty() => serde_json::to_string(&PLAN).unwrap_or_default(),
        _ => String::new(),
    }
}

/// Logs that the churn ran to its end, which the memory check waits for.
pub fn ended() {
    tracing::info!("the churn ended");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_churn_runs_only_while_its_variable_is_set() {
        assert_eq!(plan(None), "");
        assert_eq!(plan(Some(OsStr::new(""))), "");
        let on: serde_json::Value = serde_json::from_str(&plan(Some(OsStr::new("1")))).unwrap();
        assert_eq!(on["themes"], 3);
        assert_eq!(on["caps"], 20);
        assert_eq!(on["sizes"], 24);
        assert!(on["step_ms"].as_u64().is_some_and(|ms| ms > 0));
    }
}
