//! Boot: load settings, plan the order, block what cannot start (quietly when it is off),
//! skip what is off, start the rest.

use super::{KERNEL, Kernel, KernelError, Phase, SPEC, switched_off};
use crate::lifecycle::Step;
use crate::order::{self, Node, Plan, Reason};
use kx_module_api::{ModuleId, ModuleState, Notice};
use kx_settings::{Files, Store};
use std::collections::BTreeSet;
use std::iter;

/// Why a module whose settings spec is broken cannot start.
const SPEC_BROKEN: &str = "its settings spec is broken";

impl Kernel {
    /// Loads the settings and starts every module it can, in dependency order.
    /// Every notice it returns also went on the bus, as did every state change.
    ///
    /// # Errors
    /// `WrongPhase` unless the kernel is in setup, so it boots once.
    pub fn boot(&mut self) -> Result<Vec<Notice>, KernelError> {
        self.expect(Phase::Setup)?;
        self.phase = Phase::Running;
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
        Ok(self.told.take().unwrap_or_default())
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

    /// Blocks, before any start, each module with a broken spec or a plan failure; it never starts.
    /// One that is switched off, or blocked only by such modules, stops quietly; the rest fail.
    fn block(&mut self, broken: &[ModuleId]) {
        if broken.contains(&KERNEL) {
            tracing::error!("the kernel's settings spec is broken");
        }
        let specs = broken.iter().map(|&id| (id, SPEC_BROKEN.to_owned()));
        let planned = self.plan.failed().iter();
        let plan = planned.map(|(id, reason)| (*id, format!("{reason:?}")));
        let found = |(id, why)| Some((self.find(id)?, why));
        let blocked: Vec<(usize, String)> = specs.chain(plan).filter_map(found).collect();
        for &(at, _) in &blocked {
            self.slots[at].blocked = true;
        }
        self.skip_quiet(&self.inherited(broken));
        for (at, why) in blocked {
            if self.slots[at].state == ModuleState::Pending {
                self.fail(at, &why);
            }
        }
    }

    /// The modules whose only problem is a provider that cannot start: the plan says so and their spec is sound.
    fn inherited(&self, broken: &[ModuleId]) -> BTreeSet<usize> {
        let failed = self.plan.failed().iter();
        let heirs = failed.filter(|(id, reason)| {
            matches!(reason, Reason::DependsOnFailed(_)) && !broken.contains(id)
        });
        heirs.filter_map(|(id, _)| self.find(*id)).collect()
    }

    /// Skips, until none is left, each blocked `Pending` module that is switched off or blocked
    /// only by modules that stopped quietly.
    fn skip_quiet(&mut self, inherited: &BTreeSet<usize>) {
        loop {
            let quiet: Vec<usize> = (0..self.slots.len())
                .filter(|&at| self.is_quiet(at, inherited))
                .collect();
            if quiet.is_empty() {
                return;
            }
            for at in quiet {
                self.step(at, Step::Skip);
            }
        }
    }

    /// True when blocked module `at` still waits and should stop without a notice.
    fn is_quiet(&self, at: usize, inherited: &BTreeSet<usize>) -> bool {
        let slot = &self.slots[at];
        let waits = slot.blocked && slot.state == ModuleState::Pending;
        let off = self.is_disabled(slot.manifest.id);
        waits && (off || (inherited.contains(&at) && self.blocked_only_by_quiet(at)))
    }

    /// The switched-off modules and every started module that needs one of them.
    fn skipped(&self) -> BTreeSet<ModuleId> {
        let disabled = self.disabled();
        let mut skip = BTreeSet::new();
        for &id in self.plan.start() {
            if switched_off(&disabled, id) {
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
