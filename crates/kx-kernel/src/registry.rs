//! The service registry: each service with its owner and the capability that gates it.

use crate::grants::Grants;
use kx_module_api::{Capability, ModuleId, ServiceError, ServiceId};
use std::any::Any;
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

/// A stored service, boxed as the host receives it.
pub type Service = Box<dyn Any + Send + Sync>;

/// Who registered a service.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Owner {
    /// The platform adapter, before any module starts.
    Platform,
    /// A module, whose services go when it stops or fails.
    Module(ModuleId),
}

/// One service with its owner and gating capability.
struct Slot {
    owner: Owner,
    cap: Option<Capability>,
    value: Service,
}

/// Every provided service by id; the kernel owns the one registry.
#[derive(Default)]
pub struct Registry {
    slots: BTreeMap<ServiceId, Slot>,
}

impl Registry {
    /// A registry with no services.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Stores `value` under `id`, gated by `cap`.
    ///
    /// # Errors
    /// `AlreadyProvided` when `id` has a provider, which keeps it.
    pub fn provide(
        &mut self,
        owner: Owner,
        id: ServiceId,
        cap: Option<Capability>,
        value: Service,
    ) -> Result<(), ServiceError> {
        match self.slots.entry(id) {
            Entry::Occupied(_) => Err(ServiceError::AlreadyProvided(id)),
            Entry::Vacant(slot) => {
                slot.insert(Slot { owner, cap, value });
                Ok(())
            }
        }
    }

    /// Service `id` for a requester holding `grants`, gated on the stored capability.
    ///
    /// # Errors
    /// `Missing`, `CapabilityMismatch` or `NotGranted`, checked in that order.
    pub fn lookup(
        &self,
        id: ServiceId,
        cap: Option<Capability>,
        grants: &Grants,
    ) -> Result<&(dyn Any + Send + Sync), ServiceError> {
        let slot = self.slots.get(&id).ok_or(ServiceError::Missing(id))?;
        if slot.cap != cap {
            return Err(ServiceError::CapabilityMismatch(id));
        }
        match slot.cap {
            Some(cap) if !grants.has(cap) => Err(ServiceError::NotGranted(id, cap)),
            _ => Ok(&*slot.value),
        }
    }

    /// Removes every service `module` provided and hands them back for the caller to drop.
    pub fn remove_owned(&mut self, module: ModuleId) -> Vec<Service> {
        let owner = Owner::Module(module);
        self.slots
            .extract_if(.., |_, slot| slot.owner == owner)
            .map(|(_, slot)| slot.value)
            .collect()
    }
}
