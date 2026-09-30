//! A fake layouts service that shows one layout table, lists them all and records each step and pick.

use crate::layout_file::{LayoutFile, LayoutFileError};
use crate::lock;
use kx_module_api::ports::{LangLabel, LayoutId, LayoutTable, Layouts, SwitchError};
use std::sync::{Arc, Mutex};

/// A switch the fake layouts service was asked for.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    /// `step(by)`.
    Step(i32),
    /// `pick(id)`.
    Pick(LayoutId),
}

/// A layouts service over the tables a test gives, listed in that order, with the first one shown.
/// A step or pick only records the ask, as an app that has not switched yet; `show` switches.
#[derive(Debug, Default)]
pub struct FakeLayouts {
    tables: Vec<Arc<LayoutTable>>,
    shown: Mutex<usize>,
    asked: Mutex<Vec<Ask>>,
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

    /// Every step and pick so far, oldest first.
    #[must_use]
    pub fn asked(&self) -> Vec<Ask> {
        lock(&self.asked).clone()
    }

    fn ask(&self, ask: Ask) {
        lock(&self.asked).push(ask);
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

    /// Nothing changes on its own; a test switches with `show`.
    fn refresh(&self) -> bool {
        false
    }

    fn step(&self, by: i32) -> Result<(), SwitchError> {
        if self.tables.is_empty() {
            return Err(SwitchError::NotInstalled);
        }
        self.ask(Ask::Step(by));
        Ok(())
    }

    fn pick(&self, id: LayoutId) -> Result<(), SwitchError> {
        if !self.tables.iter().any(|t| t.label.id == id) {
            return Err(SwitchError::NotInstalled);
        }
        self.ask(Ask::Pick(id));
        Ok(())
    }
}

/// A layout file as the layouts service serves it; its label holds only the id and the direction,
/// since naming layouts is the layouts module's work.
impl From<&LayoutFile> for LayoutTable {
    fn from(file: &LayoutFile) -> Self {
        let label = LangLabel {
            id: file.id,
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
