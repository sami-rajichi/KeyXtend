//! The keyboard module: the board and its geometry, the keyboard's size and place, and later typing
//! and the language key. It talks to the OS only through ports, so it builds and tests everywhere.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod manifest;
mod settings;

pub use manifest::{ID, MANIFEST};
pub use settings::{Corner, KeyboardSettings, LangKey, Modifiers, SPEC};

use kx_module_api::{Manifest, Module, ModuleCx, ModuleError};

/// The keyboard module; the app registers it behind the `mod-keyboard` feature.
#[derive(Debug, Default)]
pub struct KeyboardModule;

impl Module for KeyboardModule {
    fn manifest(&self) -> &'static Manifest {
        &MANIFEST
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        cx.settings::<KeyboardSettings>()?;
        Ok(())
    }
}
