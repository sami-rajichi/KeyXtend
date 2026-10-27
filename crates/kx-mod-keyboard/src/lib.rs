//! The keyboard module: the board and its geometry, the keyboard's size and place, typing with sticky modifiers, the
//! language key, the shortcuts layer, the `KeyboardFace` the UI draws, and the pieces it typed and the words it types
//! for word prediction (ADR-0033). It talks to the OS only through ports.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod face;
pub mod geom;
pub mod langkey;
pub mod latch;
pub mod layer;
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
pub use layer::{Layer, LayerCell, LayerKey};
pub use legend::{Legend, LegendMods};
pub use manifest::{ID, MANIFEST};
pub use panel::{LangList, LangRow, ListView, Moved};
pub use place::Placer;
pub use plan::{Plan, Route};
pub use repeat::Repeat;
pub use run::{Io, Part, Pause, Ports, Sleep, TapError, Tapped, Typing, Word, drive};
pub use settings::{
    Corner, ERASE_GAP_MS, KeyboardSettings, LangKey, MAX_PAGES, Modifiers, SPEC, TEXT_CAP,
};
pub use sizer::{Limits, Resize, SizeError, Sizer};
pub use spec::{Assist, BoardError, BoardSpec, KeyKind, Latch, LatchRole, Nav, Role, Tell};
pub use view::{BoardView, KeyState, KeyView, StateView};

use kx_module_api::ports::KeyboardFaceKey;
use kx_module_api::{Manifest, Module, ModuleCx, ModuleError};
use std::fmt;
use std::sync::Arc;

/// The keyboard module; the app registers it behind the `mod-keyboard` feature.
/// It keeps its face across a restart, so the latches survive a settings change.
pub struct KeyboardModule {
    face: Option<Arc<Face>>,
    /// The pause after each Backspace of a picked word.
    pause: Arc<dyn Pause>,
}

impl KeyboardModule {
    /// A module whose picked words wait through `pause`, such as a test's fake that never sleeps.
    #[must_use]
    pub fn with_pause(pause: Arc<dyn Pause>) -> Self {
        Self { face: None, pause }
    }
}

/// A module whose picked words wait by sleeping.
impl Default for KeyboardModule {
    fn default() -> Self {
        Self::with_pause(Arc::new(Sleep))
    }
}

/// Shows only whether the face was made.
impl fmt::Debug for KeyboardModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyboardModule")
            .field("face", &self.face.is_some())
            .finish_non_exhaustive()
    }
}

impl Module for KeyboardModule {
    fn manifest(&self) -> &'static Manifest {
        &MANIFEST
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        let settings = cx.settings::<KeyboardSettings>()?;
        let spec = BoardSpec::shipped()?;
        let ports = Ports::from_cx(cx, Arc::clone(&self.pause))?;
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
        face.set_running(true);
        self.face = Some(face);
        Ok(())
    }

    /// The face stops typing picked words, as a handler may still run on the sender's thread; the UI keeps it.
    fn stop(&mut self) {
        if let Some(face) = &self.face {
            face.set_running(false);
        }
    }
}
