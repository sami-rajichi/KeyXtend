//! The keyboard module: the board and its geometry, the keyboard's size and place, and later typing
//! and the language key. It talks to the OS only through ports, so it builds and tests everywhere.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod geom;
mod manifest;
pub mod place;
mod settings;
pub mod sizer;
pub mod spec;

pub use geom::{Area, Board, KeyBox, board};
pub use manifest::{ID, MANIFEST};
pub use place::{Placer, Screen};
pub use settings::{Corner, KeyboardSettings, LangKey, Modifiers, SPEC};
pub use sizer::{Limits, Resize, SizeError, Sizer};
pub use spec::{BoardError, BoardSpec, KeyKind, Latch, Role};

use kx_module_api::{Manifest, Module, ModuleCx, ModuleError};

/// Why the module could not start: its compiled-in board is broken.
const BOARD_BROKEN: &str = "the board is broken";

/// The keyboard module; the app registers it behind the `mod-keyboard` feature.
#[derive(Debug, Default)]
pub struct KeyboardModule;

impl Module for KeyboardModule {
    fn manifest(&self) -> &'static Manifest {
        &MANIFEST
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        cx.settings::<KeyboardSettings>()?;
        BoardSpec::shipped().map_err(|_| ModuleError::Start(BOARD_BROKEN))?;
        Ok(())
    }
}
