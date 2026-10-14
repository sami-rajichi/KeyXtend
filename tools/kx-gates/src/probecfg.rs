//! Settings of the mouse-helper probes G17, G18 and G6, from `[probes]`, `[sim]` and `[g6]`.
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
    /// The stand-in face: a square this big, in px.
    pub face_px: i32,
    /// How far above the document window's lower edge the face's top sits, in px, clear of a status bar.
    pub face_overlap_px: i32,
    /// A picture counts as changed when more than this share of its pixels changed.
    pub min_changed: f64,
}

impl G6 {
    /// Refuses a face or an overlap that is not above 0, as the scroll point would then miss the document's body.
    pub fn check(&self) -> Result<(), String> {
        if self.face_px <= 0 || self.face_overlap_px <= 0 {
            return Err("g6.face_px and g6.face_overlap_px must be above 0".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g6() -> G6 {
        crate::config::load().expect("kx-gates.toml loads").g6
    }

    #[test]
    fn g6_checks_and_refuses_a_face_or_overlap_not_above_0() {
        assert_eq!(g6().check(), Ok(()));
        let bad: [fn(&mut G6); 4] = [
            |g| g.face_px = 0,
            |g| g.face_px = -1,
            |g| g.face_overlap_px = 0,
            |g| g.face_overlap_px = -1,
        ];
        for (i, f) in bad.iter().enumerate() {
            let mut g = g6();
            f(&mut g);
            assert!(g.check().is_err(), "case {i}");
        }
    }
}
