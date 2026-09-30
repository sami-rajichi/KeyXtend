//! The layouts service over the OS layout source: one cached table per layout, the list, and the switch requests.

use crate::LayoutsSettings;
use crate::logic::follow::Follow;
use crate::logic::label::label;
use crate::logic::order;
use kx_module_api::Bus;
use kx_module_api::ports::{
    KeyChars, LangLabel, LayoutChanged, LayoutId, LayoutSource, LayoutTable, Layouts, Scan,
    SwitchError, step_to,
};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// What the service last read from the source.
#[derive(Default)]
struct State {
    /// Every installed layout's label, each layout once, in the source's order.
    list: Vec<LangLabel>,
    /// Which layout is shown.
    follow: Follow,
    /// One table per layout read so far; a table goes when its layout is uninstalled.
    tables: HashMap<LayoutId, Arc<LayoutTable>>,
    /// The table of the layout shown, or an empty one while nothing is installed.
    current: Arc<LayoutTable>,
}

/// The `Layouts` service: it reads through `source` and tells `bus` when the shown layout changes.
/// No lock is held while the bus runs its handlers, so a handler may call the service.
pub struct LayoutsService {
    source: Arc<dyn LayoutSource>,
    short: BTreeMap<String, String>,
    scans: Vec<Scan>,
    bus: Bus,
    state: Mutex<State>,
}

impl LayoutsService {
    /// Reads the installed layouts and the app in front; `scans` are the keys each table holds.
    #[must_use]
    pub fn new(
        source: Arc<dyn LayoutSource>,
        settings: LayoutsSettings,
        scans: Vec<Scan>,
        bus: Bus,
    ) -> Self {
        let service = Self {
            source,
            short: settings.short,
            scans,
            bus,
            state: Mutex::default(),
        };
        service.update(&mut service.state());
        service
    }

    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Reads the list and the app in front into `state`; says whether the list and the shown layout changed.
    fn update(&self, state: &mut State) -> (bool, bool) {
        let ids = order::dedupe(self.source.installed());
        let listed = !ids.iter().eq(state.list.iter().map(|l| &l.id));
        if listed {
            state.list = ids.iter().map(|&id| self.label(id)).collect();
            state.tables.retain(|id, _| ids.contains(id));
        }
        let moved = state.follow.see(self.source.foreground(), &ids);
        if listed || moved {
            state.current = self.table(state);
        }
        (listed, moved)
    }

    /// The table of the shown layout, read once per layout; empty while nothing is installed.
    fn table(&self, state: &mut State) -> Arc<LayoutTable> {
        let Some(id) = state.follow.shown() else {
            return Arc::default();
        };
        let list = &state.list;
        let table = state.tables.entry(id).or_insert_with(|| {
            let known = list.iter().find(|l| l.id == id).cloned();
            let label = known.unwrap_or_else(|| self.label(id));
            Arc::new(LayoutTable {
                label,
                keys: self.keys(id),
            })
        });
        Arc::clone(table)
    }

    /// How the keyboard names layout `id`.
    fn label(&self, id: LayoutId) -> LangLabel {
        label(id, &self.source.facts(id), &self.short)
    }

    /// What each of the table's keys types in layout `id`, leaving out keys that type nothing.
    fn keys(&self, id: LayoutId) -> BTreeMap<Scan, KeyChars> {
        let chars = self.source.keys(id, &self.scans);
        let typed = self.scans.iter().copied().zip(chars);
        typed.filter(|(_, k)| *k != KeyChars::default()).collect()
    }
}

impl Layouts for LayoutsService {
    fn current(&self) -> Arc<LayoutTable> {
        Arc::clone(&self.state().current)
    }

    fn list(&self) -> Vec<LangLabel> {
        self.state().list.clone()
    }

    fn refresh(&self) -> bool {
        let (changed, shown) = {
            let mut state = self.state();
            let (listed, moved) = self.update(&mut state);
            (listed || moved, state.follow.shown().filter(|_| moved))
        };
        if let Some(id) = shown {
            // `LayoutChanged` needs no capability, so the bus never refuses it.
            let _ = self.bus.publish(&LayoutChanged { id });
        }
        changed
    }

    /// A step that lands on the shown layout, such as `by` 0 or a lone layout, asks nothing.
    fn step(&self, by: i32) -> Result<(), SwitchError> {
        let (to, shown) = {
            let state = self.state();
            let ids: Vec<LayoutId> = state.list.iter().map(|l| l.id).collect();
            let shown = state.follow.shown();
            (step_to(&ids, shown, by), shown)
        };
        match to {
            None => Err(SwitchError::NotInstalled),
            Some(to) if Some(to) == shown => Ok(()),
            Some(to) => self.source.request(to),
        }
    }

    fn pick(&self, id: LayoutId) -> Result<(), SwitchError> {
        let known = self.state().list.iter().any(|l| l.id == id);
        if !known {
            return Err(SwitchError::NotInstalled);
        }
        self.source.request(id)
    }
}

#[cfg(test)]
mod tests {
    use kx_module_api::Event;
    use kx_module_api::ports::LayoutChanged;

    #[test]
    fn the_change_event_needs_no_capability_so_its_publish_cannot_fail() {
        assert_eq!(LayoutChanged::CAPABILITY, None);
    }
}
