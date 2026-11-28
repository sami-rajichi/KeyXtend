//! Colours every theme shares, as the mock-up writes them once for all themes.

use serde::{Deserialize, Serialize};

use crate::palette::Shadow;
use crate::rgba::Rgba;

/// The shared colours, from `[common]` in `themes/common.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Common {
    /// Laid, through each palette's press, over a pressed key or D-pad arrow (mock-up `brightness(.94)`).
    pub press: Rgba,
    /// The recording dot on the dictation bar.
    pub rec_dot: Rgba,
    /// Laid over the frozen screen outside the snip region.
    pub snip_dim: Rgba,
    /// The snip region's edge.
    pub snip_edge: Rgba,
    /// Behind the snip region's size, the hint and the countdown.
    pub snip_tag: Rgba,
    /// The text on `snip_tag`.
    pub snip_tag_ink: Rgba,
    /// The red edge round a part that records.
    pub snip_rec: Rgba,
    /// The editor's inks, which the picture keeps in every theme and in high contrast.
    pub snip_ink_red: Rgba,
    /// Yellow ink.
    pub snip_ink_yellow: Rgba,
    /// Green ink.
    pub snip_ink_green: Rgba,
    /// Blue ink.
    pub snip_ink_blue: Rgba,
    /// Black ink.
    pub snip_ink_black: Rgba,
    /// White ink.
    pub snip_ink_white: Rgba,
    /// A step's number on red ink, at text contrast on it.
    pub snip_num_red: Rgba,
    /// A step's number on yellow ink.
    pub snip_num_yellow: Rgba,
    /// A step's number on green ink.
    pub snip_num_green: Rgba,
    /// A step's number on blue ink.
    pub snip_num_blue: Rgba,
    /// A step's number on black ink.
    pub snip_num_black: Rgba,
    /// A step's number on white ink.
    pub snip_num_white: Rgba,
    /// The mode badge's shadow (mock-up `.badge` box-shadow). Each palette takes it as `badge_shadow`, so a face reads
    /// it there, where high contrast drops it.
    #[serde(skip_serializing)]
    pub badge_shadow: Shadow,
}

impl Common {
    /// Every shared colour of our own windows with its name in `themes/common.toml`; high contrast maps each one.
    #[must_use]
    pub fn named(&self) -> [(&'static str, Rgba); 7] {
        [
            ("press", self.press),
            ("rec_dot", self.rec_dot),
            ("snip_dim", self.snip_dim),
            ("snip_edge", self.snip_edge),
            ("snip_tag", self.snip_tag),
            ("snip_tag_ink", self.snip_tag_ink),
            ("snip_rec", self.snip_rec),
        ]
    }

    /// The editor's inks by Snip's ink id, in the editor's order, each with its step number's colour.
    #[must_use]
    pub fn inks(&self) -> [(&'static str, Rgba, Rgba); 6] {
        [
            ("red", self.snip_ink_red, self.snip_num_red),
            ("yellow", self.snip_ink_yellow, self.snip_num_yellow),
            ("green", self.snip_ink_green, self.snip_num_green),
            ("blue", self.snip_ink_blue, self.snip_num_blue),
            ("black", self.snip_ink_black, self.snip_num_black),
            ("white", self.snip_ink_white, self.snip_num_white),
        ]
    }
}
