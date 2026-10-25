//! Debug builds only: with `KX_UI_CHURN` set, the app churns through its own buttons' calls (`qml/Churn.qml`), so a
//! memory check can measure after it with no mouse and no key sent.

use serde::Serialize;
use std::ffi::OsStr;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;

/// The variable that turns the churn on.
pub const CHURN_VAR: &str = "KX_UI_CHURN";

/// Caps Lock reads the other way round while set: the app's debug layout source turns `caps_on` over (ADR-0033).
static CAPS_FLIPPED: AtomicBool = AtomicBool::new(false);

/// Turns the churn's Caps Lock flag over, so the next relabel shows Caps Lock changed with no key sent.
pub fn flip_caps() {
    CAPS_FLIPPED.fetch_xor(true, SeqCst);
}

/// True while the churn shows Caps Lock the other way round from the system's.
#[must_use]
pub fn caps_flipped() -> bool {
    CAPS_FLIPPED.load(SeqCst)
}

/// The churn: ms between steps, rounds and the theme switches in each, then how many Caps Lock changes and size
/// changes, in that order. Each round first shows a key's tip, the List design's language list and the bubble (`Churn.qml`).
#[derive(Serialize)]
struct Plan {
    step_ms: u32,
    rounds: u32,
    themes: u32,
    caps: u32,
    sizes: u32,
}

/// G8's churn: ms between steps, rounds, Caps Lock changes and size changes. Each round shows the pop-ups `Plan` names,
/// then every theme once, back to the first; Caps Lock comes back to off, and the sizes go up by half and down again.
const STEP_MS: u32 = 400;
const ROUNDS: u32 = 1;
const CAPS: u32 = 20;
const SIZES: u32 = 24;

/// The plan as JSON, for a theme list of `themes`, when `value`, the variable's, is set and not empty; else an empty
/// string.
pub fn plan(value: Option<&OsStr>, themes: usize) -> String {
    match value {
        Some(v) if !v.is_empty() => {
            let plan = Plan {
                step_ms: STEP_MS,
                rounds: ROUNDS,
                themes: u32::try_from(themes).unwrap_or(u32::MAX),
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
    fn the_caps_flag_turns_over_at_each_flip() {
        let before = caps_flipped();
        flip_caps();
        assert_eq!(caps_flipped(), !before);
        flip_caps();
        assert_eq!(caps_flipped(), before, "and back");
    }

    #[test]
    fn each_round_has_one_theme_step_per_theme() {
        for themes in [3_u8, 4] {
            let on: serde_json::Value =
                serde_json::from_str(&plan(Some(OsStr::new("1")), usize::from(themes))).unwrap();
            assert_eq!(on["rounds"], ROUNDS);
            assert_eq!(on["themes"], u32::from(themes));
        }
    }
}
