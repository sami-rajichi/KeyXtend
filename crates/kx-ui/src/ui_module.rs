//! The `ui` module: it reads the `[ui]` settings, takes the keyboard face and hands both to the hub, with its
//! settings handle for the theme button and the light or dark switch. It holds `KeyboardUi`, the face's gate,
//! `ReadFocusedText`, the chip lane's (ADR-0033), `ReadClipboard`, the clipboard panel's (ADR-0034), `Vault`,
//! Quick-fill's event gate (ADR-0038), `Selection`, the selection chips' event gate (ADR-0041), `Voice`, voice
//! typing's event gate (ADR-0042), and `Snip`, Snip's event gate (ADR-0057).

use crate::hub::Hub;
use crate::settings::{SPEC, UiSettings};
use kx_module_api::ports::KeyboardFaceKey;
use kx_module_api::{Capability, Manifest, Module, ModuleCx, ModuleError, ModuleId, ServiceKey};
use std::sync::Arc;

/// The module's id, which also names its settings section.
pub const ID: ModuleId = ModuleId::new("ui");

/// The module's manifest: it needs the keyboard face, behind `KeyboardUi`, hears the chip lane, behind
/// `ReadFocusedText`, and draws the clipboard panel, behind `ReadClipboard`, Quick-fill's, behind `Vault`, the
/// selection chips, behind `Selection`, the dictation bar and Voice panel, behind `Voice`, and Snip's windows, behind
/// `Snip`.
pub static MANIFEST: Manifest = Manifest {
    id: ID,
    version: env!("CARGO_PKG_VERSION"),
    requires: &[KeyboardFaceKey::ID],
    provides: &[],
    capabilities: &[
        Capability::KeyboardUi,
        Capability::ReadFocusedText,
        Capability::ReadClipboard,
        // An event gate only: the panel's views, clicks and label names need it, and no notebook service is asked.
        Capability::Vault,
        // An event gate only: the strip's views and clicks need it, and no selection or link service is asked.
        Capability::Selection,
        // An event gate only: the bar's and panel's views, clicks and open need it; no text is typed or heard typed.
        Capability::Voice,
        // An event gate only: Snip's views, clicks and aside need it; no screen is grabbed and no file is written.
        Capability::Snip,
    ],
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
