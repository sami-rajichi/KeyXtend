//! The keyboard module: the board and its geometry, the keyboard's size and place, typing with sticky
//! modifiers, and later the language key. It talks to the OS only through ports, so it builds and tests everywhere.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod geom;
pub mod latch;
pub mod legend;
mod manifest;
pub mod place;
pub mod plan;
pub mod run;
mod settings;
pub mod sizer;
pub mod spec;

pub use geom::{Area, Board, KeyBox, board};
pub use latch::Latches;
pub use legend::{Legend, LegendMods};
pub use manifest::{ID, MANIFEST};
pub use place::Placer;
pub use plan::{Plan, Route};
pub use run::{Io, Ports, Tapped, Typing};
pub use settings::{Corner, KeyboardSettings, LangKey, Modifiers, SPEC};
pub use sizer::{Limits, Resize, SizeError, Sizer};
pub use spec::{BoardError, BoardSpec, KeyKind, Latch, Role};

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
        BoardSpec::shipped()?;
        // The keyboard face will own the ports with a `Typing`; until then the start checks they are there.
        Ports::from_cx(cx)?;
        Ok(())
    }
}
