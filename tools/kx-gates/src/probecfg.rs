//! Settings of the mouse-helper probes G17, G18 and G6, and of the ring check G12, from `[probes]`, `[g6]` and `[g12]`.
#![cfg(windows)]

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
    /// Largest clipboard content saved and put back around a probe, in bytes.
    pub keep_max_bytes: usize,
    /// Wait before removing test copies from Windows clipboard history, in ms.
    pub history_settle_ms: u64,
    /// A history entry that is the start of `text`, at least this long, is a test copy.
    pub forget_min_chars: usize,
    /// History entries longer than this, in characters, are never read.
    pub forget_max_chars: usize,
}

/// The simulated user's timing, shared by G5 and the probes; in ms.
#[derive(Debug, Clone, Deserialize)]
#[allow(
    clippy::struct_field_names,
    reason = "The `_ms` names are the config keys."
)]
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

/// Ring overlay check settings (gate G12); times in ms, distances in px.
#[derive(Debug, Clone, Deserialize)]
pub struct G12 {
    /// Click grid columns and rows over target-window's text box; each point is clicked once.
    pub grid: [u32; 2],
    /// The grid stays this far inside the text box's edges.
    pub inset_px: i32,
    /// Longest wait after a move for the ring to follow the pointer.
    pub follow_ms: u64,
    /// Wait after each click, and after the last before reading the log.
    pub settle_ms: u64,
    /// A logged press may be this far from where we clicked.
    pub slack_px: i32,
    /// Least frames a second the ring must draw.
    pub min_fps: f64,
    /// 99 in 100 frame gaps must be this long or shorter.
    pub p99_ms: f64,
    /// Longest wait for the ring window to open.
    pub open_ms: u64,
}

impl G12 {
    /// Refuses an empty grid, a negative inset or slack, or limits that are not above 0.
    pub fn check(&self) -> Result<(), String> {
        let grid = self.grid.iter().all(|&n| n > 0);
        let limits = self.min_fps > 0.0 && self.p99_ms > 0.0;
        let room = self.inset_px >= 0 && self.slack_px >= 0;
        if !grid || !limits || !room {
            return Err(
                "g12: grid, min_fps and p99_ms must be above 0, inset_px and slack_px 0 or more"
                    .into(),
            );
        }
        Ok(())
    }
}
