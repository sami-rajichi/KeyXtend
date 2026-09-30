//! A fake layout source that serves layout tables and records switch requests.

use crate::layout_file::{LayoutFile, LayoutFileError};
use crate::lock;
use kx_module_api::ports::{KeyChars, LangFacts, LayoutId, LayoutSource, Scan, Slot, SwitchError};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

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

    /// A source over the golden tables `names`, such as `["en", "fr", "ar"]`, installed in that order.
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

    /// The fake ignores `held`: a key types something when it is a control or has any character.
    fn types_any(&self, layout: LayoutId, key: Scan, _held: &[Scan]) -> bool {
        let any = |k: &KeyChars| Slot::ALL.iter().any(|&s| !k.get(s).is_empty());
        let read = |t: &LayoutFile| t.controls.contains(&key) || t.keys.get(&key).is_some_and(any);
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
mod tests {
    use super::*;

    const EXAMPLE: &str = "example";
    const ID: LayoutId = LayoutId::new(0x040C_040C);
    const OTHER: LayoutId = LayoutId::new(0x0409_0409);
    const Q: Scan = Scan::new(0x10);
    const ENTER: Scan = Scan::new(0x1C);
    const MISSING: Scan = Scan::new(0x59);

    fn example() -> FakeLayoutSource {
        FakeLayoutSource::golden(&[EXAMPLE]).unwrap()
    }

    fn other() -> LayoutFile {
        LayoutFile {
            id: OTHER,
            ..example_file()
        }
    }

    fn example_file() -> LayoutFile {
        LayoutFile::golden(EXAMPLE).unwrap()
    }

    #[test]
    fn it_serves_the_example_table() {
        let fake = example();
        assert_eq!(fake.installed(), [ID]);
        assert_eq!(fake.foreground(), Some(ID));
        assert_eq!(fake.facts(ID).iso, "fr");
        let keys = fake.keys(ID, &[Q, MISSING]);
        assert_eq!((keys[0].base.as_str(), keys[0].shift.as_str()), ("a", "A"));
        assert_eq!(keys[1], KeyChars::default());
    }

    #[test]
    fn an_unknown_layout_types_nothing_and_has_no_facts() {
        let fake = example();
        assert_eq!(fake.keys(OTHER, &[Q]), [KeyChars::default()]);
        assert_eq!(fake.facts(OTHER), LangFacts::default());
        assert!(!fake.types_any(OTHER, Q, &[]));
    }

    #[test]
    fn controls_and_characters_type_something() {
        let fake = example();
        assert!(fake.types_any(ID, ENTER, &[]));
        assert!(fake.types_any(ID, Q, &[]));
        assert!(!fake.types_any(ID, MISSING, &[]));
    }

    #[test]
    fn a_request_switches_the_app_in_front_and_is_recorded() {
        let fake = example();
        fake.add(other());
        assert_eq!(fake.installed(), [ID, OTHER]);
        assert_eq!(fake.request(OTHER), Ok(()));
        assert_eq!(fake.foreground(), Some(OTHER));
        assert_eq!(fake.requests(), [OTHER]);
    }

    #[test]
    fn with_no_app_in_front_a_request_finds_no_target() {
        let fake = example();
        fake.set_foreground(None);
        assert_eq!(fake.request(ID), Err(SwitchError::NoTarget));
        assert_eq!(fake.foreground(), None);
        assert!(fake.requests().is_empty());
    }

    #[test]
    fn an_app_that_does_not_follow_keeps_its_layout_until_the_test_switches_it() {
        let fake = example();
        fake.add(other());
        fake.set_follow(false);
        assert_eq!(fake.request(OTHER), Ok(()));
        assert_eq!(fake.foreground(), Some(ID), "asked, not switched");
        assert_eq!(fake.requests(), [OTHER]);
        fake.set_foreground(Some(OTHER));
        assert_eq!(fake.foreground(), Some(OTHER));
    }

    #[test]
    fn a_request_can_fail() {
        let fake = example();
        assert_eq!(fake.request(OTHER), Err(SwitchError::NotInstalled));
        fake.set_error(Some(SwitchError::NoTarget));
        assert_eq!(fake.request(ID), Err(SwitchError::NoTarget));
        assert!(fake.requests().is_empty());
    }

    #[test]
    fn a_removed_layout_leaves_the_list() {
        let fake = example();
        fake.add(other());
        fake.remove(ID);
        assert_eq!(fake.installed(), [OTHER]);
        assert_eq!(fake.keys(ID, &[Q]), [KeyChars::default()]);
    }

    #[test]
    fn caps_and_the_app_in_front_follow_the_test() {
        let fake = example();
        assert!(!fake.caps_on());
        fake.set_caps(true);
        assert!(fake.caps_on());
        fake.set_foreground(None);
        assert_eq!(fake.foreground(), None);
    }

    #[test]
    fn the_captured_tables_serve_english_french_and_right_to_left_arabic() {
        let fake = FakeLayoutSource::golden(&["en-us", "fr-fr", "ar-tn"]).unwrap();
        let facts: Vec<LangFacts> = fake
            .installed()
            .into_iter()
            .map(|id| fake.facts(id))
            .collect();
        let isos: Vec<&str> = facts.iter().map(|f| f.iso.as_str()).collect();
        assert_eq!(isos, ["en", "fr", "ar"]);
        let rtl: Vec<bool> = facts.iter().map(|f| f.rtl).collect();
        assert_eq!(rtl, [false, false, true]);
    }

    #[test]
    fn a_missing_golden_table_is_an_error() {
        assert!(FakeLayoutSource::golden(&["no-such-layout"]).is_err());
    }
}
