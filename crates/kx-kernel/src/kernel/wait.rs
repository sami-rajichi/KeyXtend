//! Who waits for whom: a module's providers, one that is not running, and one the user switched off.

use super::{Kernel, switched_off};
use kx_module_api::ModuleState::{Active, Stopped};
use kx_module_api::ServiceId;
use std::collections::BTreeSet;

impl Kernel {
    /// Each service module `at` requires from a module, with the first module that provides it, if any.
    /// Platform services never wait, so they are left out.
    fn providers(&self, at: usize) -> impl Iterator<Item = (ServiceId, Option<usize>)> + '_ {
        let requires = self.slots[at].manifest.requires.iter().copied();
        requires
            .filter(|s| !self.platform_ids.contains(s))
            .map(|s| (s, self.slots.iter().position(|p| p.manifest.provides(s))))
    }

    /// The first service module `at` requires whose providing module is not `Active`.
    pub(super) fn waiting_on(&self, at: usize) -> Option<ServiceId> {
        let active = |p: Option<usize>| p.is_some_and(|p| self.slots[p].state == Active);
        self.providers(at)
            .find(|&(_, p)| !active(p))
            .map(|(s, _)| s)
    }

    /// True when module `at` waits for a switched-off provider, directly or through stopped providers.
    pub(super) fn held_off(&self, at: usize) -> bool {
        let disabled = self.disabled();
        let mut seen = BTreeSet::from([at]);
        let mut todo = vec![at];
        while let Some(m) = todo.pop() {
            for p in self.providers(m).filter_map(|(_, p)| p) {
                let id = self.slots[p].manifest.id;
                if switched_off(&disabled, id) {
                    return true;
                }
                if self.slots[p].state == Stopped && seen.insert(p) {
                    todo.push(p);
                }
            }
        }
        false
    }

    /// True when every blocked provider of module `at` has stopped quietly, and it has at least one.
    pub(super) fn blocked_only_by_quiet(&self, at: usize) -> bool {
        let mut blocked = (self.providers(at).filter_map(|(_, p)| p))
            .filter(|&p| self.slots[p].blocked)
            .peekable();
        blocked.peek().is_some() && blocked.all(|p| self.slots[p].state == Stopped)
    }
}
