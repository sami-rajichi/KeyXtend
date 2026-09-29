//! Typed keys that name a service's API, id and needed capability.

use crate::{Capability, ServiceId};

/// The typed key a module names to get or provide a service, usually a unit struct.
pub trait ServiceKey: 'static {
    /// The service's API, usually a `dyn Trait`.
    type Api: ?Sized + Send + Sync + 'static;
    /// The service's registry name.
    const ID: ServiceId;
    /// The capability a consumer needs, if any.
    const CAPABILITY: Option<Capability>;
}
