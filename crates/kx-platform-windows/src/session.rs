//! The session watch on Windows (ADR-0042 decision 9): a hidden top-level window on its own thread hears locks,
//! unlocks, sleeps and wakes, and tells each change once. The table and the message rule are pure; `win` calls Windows.
#![cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "Only the Windows watch runs these rules; tests cover them everywhere."
    )
)]

#[cfg(windows)]
mod win;

#[cfg(windows)]
pub use win::WinSessionWatch;

use std::sync::LazyLock;

use kx_module_api::ports::SessionNote;
use serde::Deserialize;

/// The session table, compiled in so it is never read from disk.
const TABLE: &str = include_str!("session.toml");

/// The shipped table; tests prove it parses, and one that does not makes the watch unavailable.
static SHIPPED: LazyLock<Option<Table>> = LazyLock::new(|| {
    let table = toml::from_str(TABLE).ok();
    if table.is_none() {
        tracing::warn!("the session table does not parse");
    }
    table
});

/// A message and its wParam, as the table lists them.
type Pair = (u32, u32);

/// The messages that tell each session change.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Table {
    locked: Vec<Pair>,
    unlocked: Vec<Pair>,
    sleeping: Vec<Pair>,
    woke: Vec<Pair>,
}

impl Table {
    /// The change message `msg` with `wparam` tells; `None` for any other message.
    pub(crate) fn note(&self, msg: u32, wparam: usize) -> Option<SessionNote> {
        let pair = (msg, u32::try_from(wparam).ok()?);
        let lists = [
            (&self.locked, SessionNote::Locked),
            (&self.unlocked, SessionNote::Unlocked),
            (&self.sleeping, SessionNote::Sleeping),
            (&self.woke, SessionNote::Woke),
        ];
        lists
            .into_iter()
            .find(|(list, _)| list.contains(&pair))
            .map(|(_, note)| note)
    }

    /// True when `msg` is one the table names, whatever its wParam.
    pub(crate) fn names(&self, msg: u32) -> bool {
        let all = [&self.locked, &self.unlocked, &self.sleeping, &self.woke];
        all.into_iter().flatten().any(|(m, _)| *m == msg)
    }
}

/// The shipped table.
pub(crate) fn table() -> Option<&'static Table> {
    SHIPPED.as_ref()
}

/// The last change told, so a change Windows tells twice is told once.
#[derive(Debug, Default)]
pub(crate) struct Last(Option<SessionNote>);

impl Last {
    /// True when `note` differs from the last change told, which it then becomes.
    pub(crate) fn fresh(&mut self, note: SessionNote) -> bool {
        let fresh = self.0 != Some(note);
        self.0 = Some(note);
        fresh
    }
}

#[cfg(test)]
mod tests;
