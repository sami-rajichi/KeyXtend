//! The shortcuts layer in the face's state (spec §4.2): its pages from the settings and the shortcuts the platform
//! knows, showing and hiding it, its pages, and the board it draws.

use super::Inner;
use crate::face::REDRAW;
use crate::layer::Layer;
use crate::run::Ports;
use crate::settings::KeyboardSettings;
use crate::spec::BoardSpec;
use crate::view::{BoardView, board_view, with_layer};
use kx_module_api::ports::{FaceReply, ShortcutId, ShortcutInfo};
use std::collections::HashSet;

/// A hidden layer over the pages of `settings` that the shortcuts of `ports` know, and what they know.
pub(super) fn hidden(
    spec: &BoardSpec,
    ports: &Ports,
    settings: &KeyboardSettings,
) -> (Layer, Vec<ShortcutInfo>) {
    let known = ports.shortcuts.known();
    let layer = Layer::new(spec.layer_grid(), &settings.shortcut_pages, &ids(&known));
    (layer, known)
}

impl Inner {
    /// Takes the pages of `settings` and what the shortcuts of `ports` know; a shown layer stays, on a page in range.
    pub(super) fn relayer(&mut self, ports: &Ports, settings: &KeyboardSettings) {
        self.known = ports.shortcuts.known();
        self.layer
            .renew(&settings.shortcut_pages, &ids(&self.known));
    }

    /// Shows or hides the layer; false, changing nothing, when it has nothing to show. Showing it hides the pad, so a
    /// pointer resting there is off it.
    pub(in crate::face) fn flip(&mut self) -> bool {
        let any = self.layer.any();
        if any {
            self.layer.toggle();
            self.hovered(false);
        }
        any
    }

    /// True while the layer shows.
    pub(in crate::face) fn layered(&self) -> bool {
        self.layer.shown()
    }

    /// Turns the layer to its next page, the first after the last, redrawing both when it turned.
    pub(in crate::face) fn next_page(&mut self) -> FaceReply {
        let before = self.layer.page();
        self.layer.next_page();
        if self.layer.page() == before {
            FaceReply::default()
        } else {
            REDRAW
        }
    }

    /// True when the layer draws key `id` now.
    pub(in crate::face) fn drawn(&self, id: &str) -> bool {
        self.layer.cells().iter().any(|c| c.key.id() == id)
    }

    /// The board at its size now, with the layer's keys while it shows.
    pub(in crate::face) fn board(&self, spec: &BoardSpec) -> BoardView {
        let mut view = board_view(spec, &self.settings, self.sizer.size());
        with_layer(&mut view, spec, &self.layer, &self.known);
        view
    }
}

/// The ids in `known`.
fn ids(known: &[ShortcutInfo]) -> HashSet<ShortcutId> {
    known.iter().map(|k| k.id.clone()).collect()
}
