//! A fake layouts service that shows one layout table, lists them all and records each step and pick.

use crate::layout_file::{LayoutFile, LayoutFileError};
use crate::lock;
use kx_module_api::ports::{LangLabel, LayoutId, LayoutTable, Layouts, SwitchError};
use std::fmt;
use std::sync::{Arc, Mutex};

/// A switch the fake layouts service was asked for.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    /// `step(by)`.
    Step(i32),
    /// `pick(id)`.
    Pick(LayoutId),
}

/// What `refresh` runs once it changed the layout shown, as the real service publishes `LayoutChanged`.
#[derive(Clone, Default)]
struct Listener(Option<Arc<dyn Fn() + Send + Sync>>);

impl fmt::Debug for Listener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Listener")
    }
}

/// A layouts service over the tables a test gives, listed in that order, with the first one shown.
/// A step or pick only records the ask, as an app that has not switched yet; `show` switches at once, and
/// `switch` on the next `refresh`, as the real service sees an app switch.
#[derive(Debug, Default)]
pub struct FakeLayouts {
    tables: Vec<Arc<LayoutTable>>,
    shown: Mutex<usize>,
    pending: Mutex<Option<usize>>,
    refreshes: Mutex<usize>,
    error: Mutex<Option<SwitchError>>,
    asked: Mutex<Vec<Ask>>,
    listener: Mutex<Listener>,
}

impl FakeLayouts {
    /// A service listing `tables` in that order, showing the first.
    #[must_use]
    pub fn new(tables: Vec<LayoutTable>) -> Self {
        Self {
            tables: tables.into_iter().map(Arc::new).collect(),
            ..Self::default()
        }
    }

    /// A service over the golden tables `names`, such as `["en-us", "fr-fr"]`.
    ///
    /// # Errors
    /// The `LayoutFileError` of the first table that cannot be read.
    pub fn golden(names: &[&str]) -> Result<Self, LayoutFileError> {
        let files = names.iter().map(|n| LayoutFile::golden(n));
        let files: Vec<LayoutFile> = files.collect::<Result<_, _>>()?;
        Ok(Self::new(files.iter().map(LayoutTable::from).collect()))
    }

    /// Shows layout `id`, as when the app in front switched; false when it is not listed.
    pub fn show(&self, id: LayoutId) -> bool {
        let at = self.tables.iter().position(|t| t.label.id == id);
        if let Some(at) = at {
            *lock(&self.shown) = at;
        }
        at.is_some()
    }

    /// The app in front switched to layout `id`, which the next `refresh` sees; false when it is not listed.
    pub fn switch(&self, id: LayoutId) -> bool {
        let at = self.tables.iter().position(|t| t.label.id == id);
        if at.is_some() {
            *lock(&self.pending) = at;
        }
        at.is_some()
    }

    /// Runs `listener` after each `refresh` that changes the layout shown, with no lock held, as a bus event does.
    pub fn on_change(&self, listener: impl Fn() + Send + Sync + 'static) {
        *lock(&self.listener) = Listener(Some(Arc::new(listener)));
    }

    /// Refuses every later step and pick with `error`, or takes them again with `None`.
    pub fn set_error(&self, error: Option<SwitchError>) {
        *lock(&self.error) = error;
    }

    /// How many times `refresh` ran.
    #[must_use]
    pub fn refreshes(&self) -> usize {
        *lock(&self.refreshes)
    }

    /// Every step and pick so far, oldest first.
    #[must_use]
    pub fn asked(&self) -> Vec<Ask> {
        lock(&self.asked).clone()
    }

    /// Records `ask`, unless the test set an error.
    fn ask(&self, ask: Ask) -> Result<(), SwitchError> {
        if let Some(error) = *lock(&self.error) {
            return Err(error);
        }
        lock(&self.asked).push(ask);
        Ok(())
    }
}

impl Layouts for FakeLayouts {
    fn current(&self) -> Arc<LayoutTable> {
        let shown = *lock(&self.shown);
        self.tables.get(shown).cloned().unwrap_or_default()
    }

    fn list(&self) -> Vec<LangLabel> {
        self.tables.iter().map(|t| t.label.clone()).collect()
    }

    /// Shows the layout a test `switch`ed to, if any, then runs the listener; true when that changed the layout shown.
    fn refresh(&self) -> bool {
        *lock(&self.refreshes) += 1;
        let Some(at) = lock(&self.pending).take() else {
            return false;
        };
        let moved = std::mem::replace(&mut *lock(&self.shown), at) != at;
        let listener = lock(&self.listener).clone();
        if let (true, Some(run)) = (moved, listener.0) {
            run();
        }
        moved
    }

    fn step(&self, by: i32) -> Result<(), SwitchError> {
        if self.tables.is_empty() {
            return Err(SwitchError::NotInstalled);
        }
        self.ask(Ask::Step(by))
    }

    fn pick(&self, id: LayoutId) -> Result<(), SwitchError> {
        if !self.tables.iter().any(|t| t.label.id == id) {
            return Err(SwitchError::NotInstalled);
        }
        self.ask(Ask::Pick(id))
    }
}

/// A layout file as the layouts service serves it; its label holds only the id, the language code and the
/// direction, since naming layouts is the layouts module's work.
impl From<&LayoutFile> for LayoutTable {
    fn from(file: &LayoutFile) -> Self {
        let label = LangLabel {
            id: file.id,
            iso: file.facts.iso.to_ascii_lowercase(),
            rtl: file.facts.rtl,
            ..LangLabel::default()
        };
        Self {
            label,
            keys: file.keys.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kx_module_api::ports::Scan;

    const Q: Scan = Scan::new(0x10);

    fn fake() -> FakeLayouts {
        FakeLayouts::golden(&["en-us", "fr-fr", "ar-tn"]).unwrap()
    }

    fn ids(fake: &FakeLayouts) -> Vec<LayoutId> {
        fake.list().into_iter().map(|l| l.id).collect()
    }

    #[test]
    fn it_shows_the_first_table_and_lists_all_in_order() {
        let fake = fake();
        let files = ["en-us", "fr-fr", "ar-tn"].map(|n| LayoutFile::golden(n).unwrap());
        assert_eq!(ids(&fake), files.each_ref().map(|f| f.id));
        let shown = fake.current();
        assert_eq!(*shown, LayoutTable::from(&files[0]));
        assert_eq!(shown.keys[&Q].base, "q");
        assert!(fake.list()[2].rtl, "Arabic is right to left");
        let isos: Vec<String> = fake.list().into_iter().map(|l| l.iso).collect();
        assert_eq!(isos, ["en", "fr", "ar"], "each label carries its language");
    }

    #[test]
    fn a_step_or_pick_only_asks_until_the_test_shows_the_layout() {
        let fake = fake();
        let fr = ids(&fake)[1];
        assert_eq!((fake.step(-1), fake.pick(fr)), (Ok(()), Ok(())));
        assert_eq!(fake.asked(), [Ask::Step(-1), Ask::Pick(fr)]);
        assert!(!fake.refresh());
        assert_eq!(fake.current().keys[&Q].base, "q", "not switched yet");
        assert!(fake.show(fr));
        assert_eq!(fake.current().keys[&Q].base, "a", "AZERTY");
    }

    #[test]
    fn a_switch_lands_on_the_next_refresh_which_counts_its_calls() {
        let fake = fake();
        let fr = ids(&fake)[1];
        assert!(fake.switch(fr));
        assert_eq!(fake.current().keys[&Q].base, "q", "not seen yet");
        assert!(fake.refresh(), "the refresh sees the switch");
        assert_eq!(fake.current().keys[&Q].base, "a");
        assert!(!fake.refresh(), "nothing more changed");
        assert_eq!(fake.refreshes(), 2);
        assert!(!fake.switch(LayoutId::new(1)));
    }

    #[test]
    fn a_refresh_that_changes_the_layout_runs_the_listener_once_it_shows() {
        let fake = Arc::new(fake());
        let seen = Arc::new(Mutex::new(Vec::new()));
        let (weak, log) = (Arc::downgrade(&fake), Arc::clone(&seen));
        fake.on_change(move || {
            let shown = weak.upgrade().map(|f| f.current().label.id);
            log.lock().unwrap().extend(shown);
        });
        let fr = ids(&fake)[1];
        assert!(!fake.refresh());
        fake.switch(fr);
        assert!(fake.refresh());
        assert_eq!(*seen.lock().unwrap(), [fr], "it reads the new layout");
    }

    #[test]
    fn an_error_the_test_sets_refuses_steps_and_picks_until_cleared() {
        let fake = fake();
        let fr = ids(&fake)[1];
        fake.set_error(Some(SwitchError::Refused));
        assert_eq!(fake.step(1), Err(SwitchError::Refused));
        assert_eq!(fake.pick(fr), Err(SwitchError::Refused));
        assert!(fake.asked().is_empty());
        fake.set_error(None);
        assert_eq!(fake.step(1), Ok(()));
    }

    #[test]
    fn an_unknown_layout_or_an_empty_list_is_refused() {
        let fake = fake();
        let none = LayoutId::new(1);
        assert_eq!(fake.pick(none), Err(SwitchError::NotInstalled));
        assert!(!fake.show(none));
        let empty = FakeLayouts::new(Vec::new());
        assert_eq!(empty.step(1), Err(SwitchError::NotInstalled));
        assert_eq!(*empty.current(), LayoutTable::default());
        assert!(fake.asked().is_empty() && empty.asked().is_empty());
    }
}
