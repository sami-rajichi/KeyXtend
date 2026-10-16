//! The keyboard module: the board and its geometry, the keyboard's size and place, typing with sticky
//! modifiers, the language key and the `KeyboardFace` the UI draws. It talks to the OS only through ports.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod face;
pub mod geom;
pub mod langkey;
pub mod latch;
pub mod legend;
mod manifest;
pub mod panel;
pub mod place;
pub mod plan;
pub mod repeat;
pub mod run;
mod settings;
pub mod sizer;
pub mod spec;
pub mod view;

pub use face::Face;
pub use geom::{Area, Board, KeyBox, board};
pub use langkey::{LangClick, LangName, LangView, Turn};
pub use latch::Latches;
pub use legend::{Legend, LegendMods};
pub use manifest::{ID, MANIFEST};
pub use panel::{LangList, LangRow, ListView, Moved};
pub use place::Placer;
pub use plan::{Plan, Route};
pub use repeat::Repeat;
pub use run::{Io, Ports, TapError, Tapped, Typing};
pub use settings::{Corner, KeyboardSettings, LangKey, Modifiers, SPEC};
pub use sizer::{Limits, Resize, SizeError, Sizer};
pub use spec::{Assist, BoardError, BoardSpec, KeyKind, Latch, Nav, Role};
pub use view::{BoardView, KeyState, KeyView, StateView};

use kx_module_api::ports::KeyboardFaceKey;
use kx_module_api::{Manifest, Module, ModuleCx, ModuleError};
use std::sync::Arc;

/// The keyboard module; the app registers it behind the `mod-keyboard` feature.
/// It keeps its face across a restart, so the latches survive a settings change.
#[derive(Debug, Default)]
pub struct KeyboardModule {
    face: Option<Arc<Face>>,
}

impl Module for KeyboardModule {
    fn manifest(&self) -> &'static Manifest {
        &MANIFEST
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        let settings = cx.settings::<KeyboardSettings>()?;
        let spec = BoardSpec::shipped()?;
        let ports = Ports::from_cx(cx)?;
        let store = cx.settings_handle();
        let face = match &self.face {
            Some(face) => {
                face.renew(ports, settings, store);
                Arc::clone(face)
            }
            None => Arc::new(Face::new(spec, ports, settings, store)),
        };
        cx.provide::<KeyboardFaceKey>(face.clone())?;
        face::listen(cx, &face)?;
        self.face = Some(face);
        Ok(())
    }
}
