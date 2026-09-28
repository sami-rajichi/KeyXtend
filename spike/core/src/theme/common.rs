//! Colours every theme shares, as the mock-up writes them once for all themes.

use serde::{Deserialize, Serialize};

use super::Rgba;

/// The shared colours, from `[common]` in `themes.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Common {
    /// Laid over a pressed key (mock-up `brightness(.94)`).
    pub press: Rgba,
    /// The recording dot in the caption bar.
    pub rec_dot: Rgba,
    /// The Copy pill's button under the pointer.
    pub pill_hover: Rgba,
    /// Laid over the frozen screen outside the snip region.
    pub snip_dim: Rgba,
    /// The snip region's edge and its size text.
    pub snip_edge: Rgba,
    /// Behind the snip region's size text.
    pub snip_tag: Rgba,
}

impl Common {
    /// Every shared colour with its name in `themes.toml`.
    pub fn named(&self) -> [(&'static str, Rgba); 6] {
        [
            ("press", self.press),
            ("rec_dot", self.rec_dot),
            ("pill_hover", self.pill_hover),
            ("snip_dim", self.snip_dim),
            ("snip_edge", self.snip_edge),
            ("snip_tag", self.snip_tag),
        ]
    }
}
