//! The scroll module: the pad scrolls what the user last pointed at, gliding at two speeds until a stop (spec §5.2).
//! It moves targets only through the platform's `ScrollTarget` port (ADR-0029), so the module never talks to the OS.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod manifest;
mod settings;

pub use manifest::{ID, MANIFEST};
pub use settings::{SPEC, ScrollSettings};

use kx_module_api::ports::{ScrollTarget, ScrollTargetKey};
use kx_module_api::{Manifest, Module, ModuleCx, ModuleError};
use std::sync::Arc;

/// The scroll module: it reads `[scroll]` and keeps the scroll target for its runtime.
#[derive(Default)]
pub struct ScrollModule {
    /// The settings and the target, from start to stop.
    live: Option<(ScrollSettings, Arc<dyn ScrollTarget>)>,
}

impl Module for ScrollModule {
    fn manifest(&self) -> &'static Manifest {
        &MANIFEST
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        let settings = cx.settings::<ScrollSettings>()?;
        let target = cx.service::<ScrollTargetKey>()?;
        self.live = Some((settings, target));
        Ok(())
    }

    /// Lets go of the settings and the target.
    fn stop(&mut self) {
        self.live.take();
    }
}
