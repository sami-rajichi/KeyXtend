//! A fake layouts service that shows one layout table, lists them all, records each step and pick, and can lose one.

use crate::layout_file::{LayoutFile, LayoutFileError};
use crate::lock;
use kx_module_api::ports::{LangLabel, LayoutId, LayoutTable, Layouts, SwitchError, step_to};
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

/// A layouts service over the tables a test gives, listed in that order, with the first one shown; `remove`
/// uninstalls one. A step or pick only records the ask, as an app that has not switched yet; `show` switches at
/// once, and `switch` on the next `refresh`, as the real service sees an app switch.
#[derive(Debug, Default)]
pub struct FakeLayouts {
    tables: Mutex<Vec<Arc<LayoutTable>>>,
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
            tables: Mutex::new(tables.into_iter().map(Arc::new).collect()),
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

    /// Where layout `id` is listed, if it is.
    fn find(&self, id: LayoutId) -> Option<usize> {
        lock(&self.tables).iter().position(|t| t.label.id == id)
    }

    /// Shows layout `id`, as when the app in front switched; false when it is not listed.
    pub fn show(&self, id: LayoutId) -> bool {
        let at = self.find(id);
        if let Some(at) = at {
            *lock(&self.shown) = at;
        }
        at.is_some()
    }

    /// The app in front switched to layout `id`, which the next `refresh` sees; false when it is not listed.
    pub fn switch(&self, id: LayoutId) -> bool {
        let at = self.find(id);
        if at.is_some() {
            *lock(&self.pending) = at;
        }
        at.is_some()
    }

    /// Uninstalls layout `id` at once, dropping a pending switch; the first layout shows if `id` was shown.
    /// False when it is not listed.
    pub fn remove(&self, id: LayoutId) -> bool {
        let mut tables = lock(&self.tables);
        let Some(at) = tables.iter().position(|t| t.label.id == id) else {
            return false;
        };
        tables.remove(at);
        drop(tables);
        *lock(&self.pending) = None;
        let mut shown = lock(&self.shown);
        *shown = match (*shown).cmp(&at) {
            std::cmp::Ordering::Less => *shown,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => *shown - 1,
        };
        true
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
        lock(&self.tables).get(shown).cloned().unwrap_or_default()
    }

    fn list(&self) -> Vec<LangLabel> {
        lock(&self.tables).iter().map(|t| t.label.clone()).collect()
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

    /// As the real service: a step that lands on the shown layout asks nothing.
    fn step(&self, by: i32) -> Result<(), SwitchError> {
        let ids: Vec<LayoutId> = self.list().iter().map(|l| l.id).collect();
        let shown = self.current().label.id;
        match step_to(&ids, Some(shown), by) {
            None => Err(SwitchError::NotInstalled),
            Some(to) if to == shown => Ok(()),
            Some(_) => self.ask(Ask::Step(by)),
        }
    }

    fn pick(&self, id: LayoutId) -> Result<(), SwitchError> {
        if self.find(id).is_none() {
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
mod tests;
