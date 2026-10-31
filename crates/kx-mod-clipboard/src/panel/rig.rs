//! What the panel and view tests share: the page size, a history of text copies and an open panel.

use kx_module_api::Redacted;
use kx_module_api::ports::{LocalTime, PanelKey};

use super::{Panel, PanelConfig};
use crate::fixture::{CFG, at, copied, date};
use crate::order::History;

/// The ui's `page_rows` and the plan's `search_chars`.
pub(crate) const PANEL: PanelConfig = PanelConfig {
    rows: 5,
    query_chars: CFG.search_chars,
};

/// 2026-10-31 at 16:32 in Paris.
pub(crate) fn now() -> LocalTime {
    at(date(2026, 10, 31), 16, 32)
}

/// A history of text copies of `texts`, the first copied first, so the last is at the top.
pub(crate) fn history(texts: &[&str]) -> History {
    let mut history = History::default();
    for t in texts {
        history.add(copied(t, &now()));
    }
    history
}

/// The id of the entry titled `title`.
pub(crate) fn id(history: &History, title: &str) -> u64 {
    let found = history.entries().iter().find(|e| e.title.expose() == title);
    found.map(|e| e.id).unwrap()
}

/// A panel opened with `cfg`.
pub(crate) fn opened_with(cfg: PanelConfig) -> Panel {
    let mut panel = Panel::new(cfg);
    panel.open();
    panel
}

/// A panel opened with the plan's page.
pub(crate) fn opened() -> Panel {
    opened_with(PANEL)
}

/// The key that types `s` into the search.
pub(crate) fn chars(s: &str) -> PanelKey {
    PanelKey::Text(Redacted::new(s.to_owned()))
}
