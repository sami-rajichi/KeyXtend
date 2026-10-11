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

/// G8's churn: ms between steps, theme rounds, Caps Lock changes and size changes. Each round shows every theme once
/// and comes back to the first; Caps Lock comes back to off, and the sizes go up by half of them and down again.
const STEP_MS: u32 = 400;
const ROUNDS: u32 = 1;
const CAPS: u32 = 20;
const SIZES: u32 = 24;

/// The plan as JSON, for a theme list of `themes`, when `value`, the variable's, is set and not empty; else an empty
/// string.
pub fn plan(value: Option<&OsStr>, themes: usize) -> String {
    match value {
        Some(v) if !v.is_empty() => {
            let count = u32::try_from(themes).unwrap_or(u32::MAX);
            let plan = Plan {
                step_ms: STEP_MS,
                themes: ROUNDS.saturating_mul(count),
                caps: CAPS,
                sizes: SIZES,
            };
            serde_json::to_string(&plan).unwrap_or_default()
        }
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
        assert_eq!(plan(None, 3), "");
        assert_eq!(plan(Some(OsStr::new("")), 3), "");
        let on: serde_json::Value = serde_json::from_str(&plan(Some(OsStr::new("1")), 3)).unwrap();
        assert_eq!(on["caps"], 20);
        assert_eq!(on["sizes"], 24);
        assert!(on["step_ms"].as_u64().is_some_and(|ms| ms > 0));
    }

    #[test]
    fn the_plan_has_one_theme_step_per_theme_per_round() {
        for themes in [3_u8, 4] {
            let on: serde_json::Value =
                serde_json::from_str(&plan(Some(OsStr::new("1")), usize::from(themes))).unwrap();
            assert_eq!(on["themes"], ROUNDS * u32::from(themes));
        }
    }
}
