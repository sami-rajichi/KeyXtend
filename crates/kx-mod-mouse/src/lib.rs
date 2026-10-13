//! The mouse module: Right-click by holding still, Grab, and modifier + click (spec §5.1). Its hold engine runs on
//! the platform's pointer hook (ADR-0027), so the module never talks to the OS.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod manifest;
mod settings;

pub use manifest::{ID, MANIFEST};
pub use settings::MouseSettings;

use kx_module_api::ports::{PointerHook, PointerHookKey};
use kx_module_api::{Manifest, Module, ModuleCx, ModuleError};
use std::sync::Arc;

/// The mouse module: it reads `[mouse]` and keeps the pointer hook for the engine.
#[derive(Default)]
pub struct MouseModule {
    /// The pointer hook, which the engine runs on while a mode needs it.
    hook: Option<Arc<dyn PointerHook>>,
}

impl Module for MouseModule {
    fn manifest(&self) -> &'static Manifest {
        &MANIFEST
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        cx.settings::<MouseSettings>()?;
        self.hook = Some(cx.service::<PointerHookKey>()?);
        Ok(())
    }
}
