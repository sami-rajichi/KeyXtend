//! The `ui` module: it reads the `[ui]` settings, takes the keyboard face and hands both to the hub, with its
//! settings handle for the theme button and the light or dark switch. It holds no capability.

use crate::hub::Hub;
use crate::settings::{SPEC, UiSettings};
use kx_module_api::ports::KeyboardFaceKey;
use kx_module_api::{Manifest, Module, ModuleCx, ModuleError, ModuleId, ServiceKey};
use std::sync::Arc;

/// The module's id, which also names its settings section.
pub const ID: ModuleId = ModuleId::new("ui");

/// The module's manifest: it needs the keyboard face and no capability.
pub static MANIFEST: Manifest = Manifest {
    id: ID,
    version: env!("CARGO_PKG_VERSION"),
    requires: &[KeyboardFaceKey::ID],
    provides: &[],
    capabilities: &[],
    settings: Some(&SPEC),
};

/// The `ui` module over the app's hub; a restart hands the hub new settings.
pub struct UiModule {
    hub: Arc<Hub>,
}

impl UiModule {
    /// The module that feeds `hub`.
    #[must_use]
    pub fn new(hub: Arc<Hub>) -> Self {
        Self { hub }
    }
}

impl Module for UiModule {
    fn manifest(&self) -> &'static Manifest {
        &MANIFEST
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        let settings = cx.settings::<UiSettings>()?;
        let face = cx.service::<KeyboardFaceKey>()?;
        self.hub.attach(settings, face, cx.settings_handle());
        Ok(())
    }

    fn stop(&mut self) {
        self.hub.detach();
    }
}
