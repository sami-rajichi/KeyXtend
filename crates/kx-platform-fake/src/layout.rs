//! A fake layout source that serves layout tables and records switch requests.

use crate::layout_file::{LayoutFile, LayoutFileError};
use crate::lock;
use kx_module_api::ports::{KeyChars, LangFacts, LayoutId, LayoutSource, Scan, Slot, SwitchError};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// Left and right Ctrl, by scan code.
const CTRL: [Scan; 2] = [Scan::new(0x1D), Scan::new(0xE01D)];
/// Left and right Alt, by scan code; with Ctrl they read another level, so the Ctrl rule stands aside.
const ALT: [Scan; 2] = [Scan::new(0x38), Scan::new(0xE038)];

/// True for a letter key: its base is letters that Shift capitalises, or letters without case, as Arabic's are.
fn is_letter(k: &KeyChars) -> bool {
    let upper = k.base.to_uppercase();
    let caseless = upper == k.base.to_lowercase();
    let letters = !k.base.is_empty() && k.base.chars().all(char::is_alphabetic);
    letters && (caseless || k.shift == upper)
}

/// True when `held` has a Ctrl and no Alt, so only controls and letter keys type.
fn ctrl_only(held: &[Scan]) -> bool {
    let any = |set: &[Scan]| held.iter().any(|h| set.contains(h));
    any(&CTRL) && !any(&ALT)
}

/// A layout source over tables the test gives.
/// A request switches the app in front at once, unless the test turns following off.
#[derive(Debug, Default)]
pub struct FakeLayoutSource {
    tables: Mutex<Vec<LayoutFile>>,
    front: Mutex<Option<LayoutId>>,
    caps: AtomicBool,
    requests: Mutex<Vec<LayoutId>>,
    error: Mutex<Option<SwitchError>>,
    ignore: AtomicBool,
}

impl FakeLayoutSource {
    /// A source with `tables` installed in that order, the first one in front.
    #[must_use]
    pub fn new(tables: Vec<LayoutFile>) -> Self {
        let front = tables.first().map(|t| t.id);
        Self {
            tables: Mutex::new(tables),
            front: Mutex::new(front),
            ..Self::default()
        }
    }

    /// A source over the golden tables `names`, such as `["en-us", "fr-fr", "ar-tn"]`, installed in that order.
    ///
    /// # Errors
    /// The `LayoutFileError` of the first table that cannot be read.
    pub fn golden(names: &[&str]) -> Result<Self, LayoutFileError> {
        let tables = names.iter().map(|n| LayoutFile::golden(n));
        Ok(Self::new(tables.collect::<Result<_, _>>()?))
    }

    /// Installs `table` after the others, as when the user adds a layout.
    pub fn add(&self, table: LayoutFile) {
        lock(&self.tables).push(table);
    }

    /// Uninstalls layout `id`; the app in front keeps its layout until the test sets another.
    pub fn remove(&self, id: LayoutId) {
        lock(&self.tables).retain(|t| t.id != id);
    }

    /// Sets the layout of the app in front.
    pub fn set_foreground(&self, id: Option<LayoutId>) {
        *lock(&self.front) = id;
    }

    /// Turns Caps Lock on or off.
    pub fn set_caps(&self, on: bool) {
        self.caps.store(on, Ordering::SeqCst);
    }

    /// Refuses every later request with `error`, or takes them again with `None`.
    pub fn set_error(&self, error: Option<SwitchError>) {
        *lock(&self.error) = error;
    }

    /// With `false`, the app in front records requests but keeps its layout, as a slow or unwilling app does.
    pub fn set_follow(&self, on: bool) {
        self.ignore.store(!on, Ordering::SeqCst);
    }

    /// Every switch request that went through, oldest first.
    #[must_use]
    pub fn requests(&self) -> Vec<LayoutId> {
        lock(&self.requests).clone()
    }

    /// `read` of the table of layout `id`, or `None` when it is not installed.
    fn with<T>(&self, id: LayoutId, read: impl FnOnce(&LayoutFile) -> T) -> Option<T> {
        lock(&self.tables).iter().find(|t| t.id == id).map(read)
    }
}

impl LayoutSource for FakeLayoutSource {
    fn installed(&self) -> Vec<LayoutId> {
        lock(&self.tables).iter().map(|t| t.id).collect()
    }

    fn foreground(&self) -> Option<LayoutId> {
        *lock(&self.front)
    }

    fn keys(&self, layout: LayoutId, scans: &[Scan]) -> Vec<KeyChars> {
        let read = |t: &LayoutFile| {
            let chars = |s| t.keys.get(s).cloned().unwrap_or_default();
            scans.iter().map(chars).collect()
        };
        self.with(layout, read)
            .unwrap_or_else(|| vec![KeyChars::default(); scans.len()])
    }

    fn facts(&self, layout: LayoutId) -> LangFacts {
        self.with(layout, |t| t.facts.clone()).unwrap_or_default()
    }

    fn caps_on(&self) -> bool {
        self.caps.load(Ordering::SeqCst)
    }

    /// A control or a key with any character types, as `ToUnicodeEx` reads the captured tables with Win, Shift or Alt
    /// held too; with Ctrl and no Alt held, only controls and letter keys type.
    fn types_any(&self, layout: LayoutId, key: Scan, held: &[Scan]) -> bool {
        let any = |k: &KeyChars| Slot::ALL.iter().any(|&s| !k.get(s).is_empty());
        let chars: fn(&KeyChars) -> bool = if ctrl_only(held) { is_letter } else { any };
        let read =
            |t: &LayoutFile| t.controls.contains(&key) || t.keys.get(&key).is_some_and(chars);
        self.with(layout, read).unwrap_or(false)
    }

    fn request(&self, layout: LayoutId) -> Result<(), SwitchError> {
        if let Some(error) = *lock(&self.error) {
            return Err(error);
        }
        if self.foreground().is_none() {
            return Err(SwitchError::NoTarget);
        }
        self.with(layout, |_| ()).ok_or(SwitchError::NotInstalled)?;
        lock(&self.requests).push(layout);
        if !self.ignore.load(Ordering::SeqCst) {
            *lock(&self.front) = Some(layout);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
