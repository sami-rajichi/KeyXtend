//! Boot: load settings, plan the order, fail what cannot start, skip what is off, start the rest.

use super::{KERNEL, Kernel, SPEC};
use crate::lifecycle::Step;
use crate::order::{self, Node, Plan};
use kx_module_api::{ModuleId, ModuleState, Notice};
use kx_settings::{Files, Store};
use std::collections::BTreeSet;
use std::iter;

/// Why a module whose settings spec is broken cannot start.
const SPEC_BROKEN: &str = "its settings spec is broken";

impl Kernel {
    /// Loads the settings and starts every module it can, in dependency order; call it once.
    /// Every notice it returns also went on the bus, as did every state change.
    pub fn boot(&mut self) -> Vec<Notice> {
        self.told = Some(Vec::new());
        let broken = self.load();
        self.plan = self.plan_order();
        self.block(&broken);
        let skip = self.skipped();
        for id in self.plan.start().to_vec() {
            let Some(at) = self.find(id) else { continue };
            if self.slots[at].state != ModuleState::Pending {
                continue;
            }
            if skip.contains(&id) {
                self.step(at, Step::Skip);
            } else {
                self.start(at);
            }
        }
        self.save();
        self.told.take().unwrap_or_default()
    }

    /// Loads the store, kernel section first, tells its notices and returns the broken specs.
    fn load(&mut self) -> Vec<ModuleId> {
        let files = Files::new(self.platform.dirs().data_dir().path());
        let modules = self.slots.iter();
        let specs: Vec<_> = iter::once((KERNEL, &SPEC))
            .chain(modules.filter_map(|s| Some((s.manifest.id, s.manifest.settings?))))
            .collect();
        let (store, report) = Store::load(files, &specs);
        self.store = Some(store);
        for notice in report.notices {
            self.notify(notice);
        }
        report.broken
    }

    fn plan_order(&self) -> Plan {
        let node = |s: &super::Slot| Node {
            id: s.manifest.id,
            requires: s.manifest.requires,
            provides: s.manifest.provides,
        };
        let nodes: Vec<Node<'_>> = self.slots.iter().map(node).collect();
        order::plan(&nodes, &self.platform_ids)
    }

    /// Fails, before any start, each module with a broken spec or a plan failure; they never start.
    fn block(&mut self, broken: &[ModuleId]) {
        if broken.contains(&KERNEL) {
            tracing::error!("the kernel's settings spec is broken");
        }
        let specs = broken.iter().map(|&id| (id, SPEC_BROKEN.to_owned()));
        let planned = self.plan.failed().iter();
        let plan = planned.map(|(id, reason)| (*id, format!("{reason:?}")));
        let blocked: Vec<_> = specs.chain(plan).collect();
        for (id, why) in blocked {
            let Some(at) = self.find(id) else { continue };
            self.slots[at].blocked = true;
            if self.slots[at].state == ModuleState::Pending {
                self.fail(at, &why);
            }
        }
    }

    /// The switched-off modules and every started module that needs one of them.
    fn skipped(&self) -> BTreeSet<ModuleId> {
        let disabled = self.disabled();
        let mut skip = BTreeSet::new();
        for &id in self.plan.start() {
            if disabled.iter().any(|d| d == id.as_str()) {
                skip.insert(id);
                skip.extend(order::dependents(&self.plan, id));
            }
        }
        skip
    }

    /// Saves when the file is behind; the first failed save since a success tells the user.
    pub(super) fn save(&mut self) {
        let Some(store) = self.store.as_mut() else {
            return;
        };
        if !store.needs_save() {
            return;
        }
        if let Err(unsaved) = store.save() {
            tracing::warn!(error = %unsaved.error, "settings not saved");
            if let Some(notice) = unsaved.notice {
                self.notify(notice);
            }
        }
    }
}
