//! The layouts module: reads the OS keyboard layouts through `LayoutSource` and provides `Layouts`,
//! the table of the layout shown, the language list in the OS order, and the switch requests.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod manifest;
mod settings;

pub use manifest::{ID, MANIFEST};
pub use settings::{LayoutsSettings, SPEC};

use kx_module_api::{Manifest, Module, ModuleCx, ModuleError};

/// The layouts module; the app registers it behind the `mod-layouts` feature.
#[derive(Debug, Default)]
pub struct LayoutsModule;

impl Module for LayoutsModule {
    fn manifest(&self) -> &'static Manifest {
        &MANIFEST
    }

    fn start(&mut self, _cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        Ok(())
    }
}
