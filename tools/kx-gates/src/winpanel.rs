//! Windows' own clipboard panel, which an injected Win+V opens only from a uiAccess product: G20's and G25's parts that
//! need it are skipped on any other product, such as the test build, and neither fail nor pass their gate.
#![cfg(windows)]

use serde_json::{Value, json};

use crate::workertok::{PRODUCT, Rights};

/// Why the history-off part is skipped: Windows refuses an injected Win+V from a process without uiAccess.
pub const NO_UI_ACCESS: &str = "Windows' own panel needs a uiAccess product, so Win and V with the history key off \
                                is checked by hand on the dev install";
/// The key, and the console word, of a part that was skipped.
pub const SKIPPED: &str = "skipped";

/// The history-off part: skipped with its reason when the `product`'s token has no uiAccess, else what `run` gives;
/// a token that could not be read runs it.
pub fn history_off(product: Option<Rights>, run: impl FnOnce() -> Value) -> Value {
    match product {
        Some(p) if !p.ui_access => json!({ SKIPPED: true, "why": NO_UI_ACCESS, PRODUCT: p }),
        _ => run(),
    }
}

/// True when `part` was skipped by `history_off`.
pub fn skipped(part: &Value) -> bool {
    part[SKIPPED] == true
}

/// True when at least one of `parts` was judged and every judged one passed; a skipped part neither fails nor passes.
pub fn judged_ok<'a>(parts: impl IntoIterator<Item = &'a Value>) -> bool {
    let mut judged = parts.into_iter().filter(|p| !skipped(p)).peekable();
    judged.peek().is_some() && judged.all(|p| p["ok"] == true)
}
