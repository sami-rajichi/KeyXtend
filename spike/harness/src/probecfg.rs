//! Settings of the mouse-helper probes G17, G18 and G6, from `[probes]` and `[g6]`.

use serde::Deserialize;

/// What the probes open and how the simulated user moves; times in ms, distances in px.
#[derive(Debug, Clone, Deserialize)]
pub struct Probes {
    /// Known text for the selection and modifier cases.
    pub text: String,
    /// Title of the Chrome probe page, used to find its window.
    pub page_title: String,
    /// Lines in the scroll documents.
    pub scroll_lines: usize,
    /// Characters per line in the scroll documents.
    pub scroll_cols: usize,
    /// Files in the Explorer scroll folder.
    pub scroll_files: usize,
    /// Name start of those files, before their number.
    pub scroll_prefix: String,
    /// The file Grab moves into `drop_dir`.
    pub move_file: String,
    /// The file Grab carries, then drops with Esc, so it stays.
    pub keep_file: String,
    /// The folder files are dropped into.
    pub drop_dir: String,
    /// The two files of the Ctrl+click case.
    pub ctrl_files: [String; 2],
    /// Style of the Chrome probe page's text block.
    pub page_css: String,
    /// Font of Word's RTF file.
    pub rtf_font: String,
    /// Font size of Word's RTF file, in half points.
    pub rtf_half_points: u32,
    /// Moves from the grab point to the drop point.
    pub move_steps: i32,
    /// Wait after a drop, click or scroll before checking.
    pub settle_ms: u64,
    /// A text point sits this far inside a line's left edge.
    pub inset_px: i32,
}

/// The simulated user's timing, shared by G5 and the probes; in ms.
#[derive(Debug, Clone, Deserialize)]
pub struct Sim {
    /// Pause between steps.
    pub step_ms: u64,
    /// How long a short click is held.
    pub click_ms: u64,
    /// A long hold lasts the hold time plus this.
    pub hold_margin_ms: u64,
}

/// Scroll probe settings.
#[derive(Debug, Clone, Deserialize)]
pub struct G6 {
    /// One wheel notch, as Windows counts it.
    pub wheel_delta: i32,
    /// Notches, or UIA small steps, per scroll.
    pub notches: u32,
    /// The stand-in face: a square this big at the desktop's top-left corner, in px.
    pub face_px: i32,
    /// A picture counts as changed when more than this share of its pixels changed.
    pub min_changed: f64,
}
