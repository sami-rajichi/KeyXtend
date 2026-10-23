//! Shortcuts by id: the compiled-in tables, and each send planned as key events in the app's layout or an OS call.
//! The plan is pure over the injector's `Os`; only `win` calls Windows.
#![cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "Only the Windows service runs this logic; tests cover it everywhere."
    )
)]

use std::collections::BTreeMap;
use std::sync::LazyLock;

use kx_module_api::ports::{
    InjectError, KeyName, LayoutId, Scan, ShortcutError, ShortcutId, ShortcutInfo, Step,
};
use serde::Deserialize;
use thiserror::Error;

use crate::inject::{Os, deliver};
use crate::keys::EXTENDED_PREFIXES;
use crate::strokes::{self, Key, KeyEvent};

#[cfg(windows)]
mod win;

#[cfg(windows)]
pub use win::WinShortcuts;

/// Each shortcut id with its steps, compiled in so it is never read from disk.
const SHORTCUTS: &str = include_str!("shortcuts.toml");
/// Each key name with its virtual key.
const KEYNAMES: &str = include_str!("keynames.toml");

/// The shipped tables, parsed once; tests prove they parse, so the empty fallback never runs.
static TABLE: LazyLock<Table> =
    LazyLock::new(|| Table::parse(SHORTCUTS, KEYNAMES).unwrap_or_default());

/// The shipped tables.
pub(crate) fn table() -> &'static Table {
    &TABLE
}

/// Why a shortcut table is unusable.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub(crate) enum TableError {
    /// A table is not TOML of the expected shape, or holds a bad id or name.
    #[error("a shortcut table does not parse: {0}")]
    Parse(String),
    /// A shortcut has no steps, or a step no keys.
    #[error("a shortcut or one of its steps is empty")]
    Empty,
    /// A step names a key the key-name table lacks.
    #[error("a shortcut names a key with no virtual key")]
    UnknownName,
    /// An id is listed twice.
    #[error("a shortcut id is listed twice")]
    Duplicate,
}

/// An OS call that does a shortcut in place of its keys.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Call {
    /// Locks the session (`LockWorkStation`), as Windows ignores an injected Win+L.
    Lock,
}

/// What a send does: key events in one batch, or an OS call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Plan {
    /// Every step's events, in order.
    Keys(Vec<KeyEvent>),
    /// The call.
    Call(Call),
}

/// The shortcut table as TOML holds it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTable {
    shortcuts: Vec<RawEntry>,
}

/// One shortcut as TOML holds it: each step's key names, the last tapped, and its call.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEntry {
    id: ShortcutId,
    steps: Vec<Vec<KeyName>>,
    call: Option<Call>,
}

/// One shortcut: its steps by name for the tips, by virtual key for the send, and its call.
struct Entry {
    info: ShortcutInfo,
    vks: Vec<Vec<u16>>,
    call: Option<Call>,
}

impl Entry {
    /// Checks `raw` and maps its names with `names`.
    fn new(raw: RawEntry, names: &BTreeMap<KeyName, u16>) -> Result<Self, TableError> {
        if raw.steps.is_empty() {
            return Err(TableError::Empty);
        }
        let (mut steps, mut vks) = (Vec::new(), Vec::new());
        for keys in raw.steps {
            let (key, held) = keys.split_last().ok_or(TableError::Empty)?;
            let vk = |n: &KeyName| names.get(n).copied().ok_or(TableError::UnknownName);
            vks.push(keys.iter().map(vk).collect::<Result<_, _>>()?);
            steps.push(Step {
                held: held.to_vec(),
                key: key.clone(),
            });
        }
        let info = ShortcutInfo { id: raw.id, steps };
        Ok(Self {
            info,
            vks,
            call: raw.call,
        })
    }
}

/// The parsed tables: each shortcut in table order.
#[derive(Default)]
pub(crate) struct Table {
    entries: Vec<Entry>,
}

impl Table {
    /// Reads the shortcut table `shortcuts` against the key-name table `names`.
    pub(crate) fn parse(shortcuts: &str, names: &str) -> Result<Self, TableError> {
        let names: BTreeMap<KeyName, u16> = toml::from_str(names).map_err(|e| broken(&e))?;
        let raw: RawTable = toml::from_str(shortcuts).map_err(|e| broken(&e))?;
        let mut entries: Vec<Entry> = Vec::with_capacity(raw.shortcuts.len());
        for entry in raw.shortcuts {
            if entries.iter().any(|e| e.info.id == entry.id) {
                return Err(TableError::Duplicate);
            }
            entries.push(Entry::new(entry, &names)?);
        }
        Ok(Self { entries })
    }

    /// Every shortcut with its steps, in table order.
    pub(crate) fn known(&self) -> Vec<ShortcutInfo> {
        self.entries.iter().map(|e| e.info.clone()).collect()
    }

    /// The shortcut `id`.
    fn find(&self, id: &ShortcutId) -> Option<&Entry> {
        self.entries.iter().find(|e| e.info.id == *id)
    }
}

/// A TOML error as a table error.
fn broken(e: &toml::de::Error) -> TableError {
    TableError::Parse(e.to_string())
}

/// Plans `id` with the latched `held` scans, in the layout of the app in front, else the keyboard's own.
pub(crate) fn plan<O: Os>(
    os: &O,
    table: &Table,
    id: &ShortcutId,
    held: &[Scan],
) -> Result<Plan, ShortcutError> {
    let layout = os.layout().unwrap_or_else(|| os.own_layout());
    plan_in(os, layout, table, id, held)
}

/// Plans `id` with the latched `held` scans, its keys mapped in `layout`.
pub(crate) fn plan_in<O: Os>(
    os: &O,
    layout: LayoutId,
    table: &Table,
    id: &ShortcutId,
    held: &[Scan],
) -> Result<Plan, ShortcutError> {
    let entry = table.find(id).ok_or(ShortcutError::Unknown)?;
    if let Some(call) = entry.call {
        return Ok(Plan::Call(call));
    }
    let latched: Vec<Key> = held.iter().map(|&s| Key::Scan(s)).collect();
    let step = |vks: &Vec<u16>| {
        let keys = vks.iter().map(|&vk| Key::vk(vk, os.scan_of(vk, layout)));
        strokes::combo(&once_each(latched.iter().copied().chain(keys)))
    };
    Ok(Plan::Keys(entry.vks.iter().flat_map(step).collect()))
}

/// `keys` in order, each left out when a key already in place presses the same key, so a latch is pressed once.
fn once_each(keys: impl Iterator<Item = Key>) -> Vec<Key> {
    let mut kept: Vec<Key> = Vec::new();
    for key in keys {
        let at = place(key);
        if !kept
            .iter()
            .any(|&k| k == key || (at.is_some() && place(k) == at))
        {
            kept.push(key);
        }
    }
    kept
}

/// Where `key` sits: its scan code with the prefix it goes with; `None` for a key with no scan code.
fn place(key: Key) -> Option<u16> {
    match key {
        Key::Scan(s) => Some(s.code()),
        Key::Vk { scan, extended, .. } if scan != 0 => {
            let prefix = if extended { EXTENDED_PREFIXES[0] } else { 0 };
            Some(u16::from_be_bytes([prefix, scan]))
        }
        Key::Vk { .. } | Key::Unit(_) => None,
    }
}

/// Sends `events` in one batch; after a partial send every held key is let go and the counts come back.
pub(crate) fn deliver_keys<O: Os>(os: &O, events: &[KeyEvent]) -> Result<(), ShortcutError> {
    deliver(os, events).map_err(|e| match e {
        InjectError::Refused { sent, total } => ShortcutError::Refused { sent, total },
        InjectError::Unmappable => ShortcutError::Failed,
    })
}

#[cfg(test)]
mod tests;
