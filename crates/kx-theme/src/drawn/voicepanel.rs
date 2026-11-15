//! What the Voice panel draws beyond the clipboard's pairs (VoiceMark.qml, VoiceSwitch.qml, VoiceProgress.qml,
//! VoiceBand.qml): a mark on its chip on any row, the lit and the outlined marks, a switch on and off, the progress line
//! and the band above a list. A ticked row's text and the ghost buttons are the clipboard's pairs.

use super::{Drawn, worst};
use crate::contrast::{WCAG_AA, WCAG_SHAPES};
use crate::rgba::Rgba;
use crate::view::LookView;

/// A plain mark's word on its chip.
pub(super) const MARK: &str = "voice mark on its chip";
/// A plain mark's icon on its chip.
pub(super) const MARK_ICON: &str = "voice mark icon on its chip";
/// In use, in the badge colours.
pub(super) const MARK_LIT: &str = "voice lit mark";
/// An outlined mark's icon, on a row at rest; on a lit row it takes `sel_ink`.
pub(super) const MARK_OUT: &str = "voice outlined mark icon";
/// An outlined mark's icon, or an off switch, on a row under the pointer or ticked.
const LIT_ROW: &str = "voice outlined mark or off switch on a lit row";
/// A switch that is on: its knob on its track. The track may melt into the row, as ET66 dark's badge does, since the
/// On or Off word beside it also says the state (WCAG 1.4.11).
pub(super) const SWITCH_ON: &str = "voice switch on";
/// A switch that is off: its outline and knob on a row at rest.
pub(super) const SWITCH_OFF: &str = "voice switch off";
/// A download's or a check's progress on its track, on a row at rest.
pub(super) const PROGRESS: &str = "voice progress on its track";
/// The progress on a row under the pointer or ticked.
pub(super) const LIT_PROGRESS: &str = "voice progress on a lit row";
/// A band's line above a list.
pub(super) const BAND: &str = "voice band line";
/// A band's icon.
pub(super) const BAND_ICON: &str = "voice band icon";

/// One pair: its name, the ink, the backgrounds it sits on and the least contrast it needs.
type Pair = (&'static str, Rgba, Vec<Rgba>, f32);

/// The panel's own pairs; high contrast draws a chip's mark in `ButtonText`, a lit row's parts in `HighlightText` and
/// the progress track in `Window`, as system colours.
pub(super) fn panel(v: &LookView, out: &mut Vec<Drawn>) {
    for (name, ink, bgs, min) in marks(v).into_iter().chain(controls(v)) {
        out.push(worst(name, ink, 1.0, &bgs, min));
    }
}

/// A row at rest, under the pointer, ticked, and both.
fn rows(v: &LookView) -> [Rgba; 4] {
    let p = &v.palette;
    let ticked = p.pop_sel.on(p.pop_bg);
    [
        p.pop_bg,
        p.pop_hover.on(p.pop_bg),
        ticked,
        p.pop_hover.on(ticked),
    ]
}

/// A mark's pairs: on its chip on every row, lit, and outlined at rest or on a lit row.
fn marks(v: &LookView) -> [Pair; 5] {
    let p = &v.palette;
    let [rest, hot, ticked, both] = rows(v);
    let chips: Vec<Rgba> = rows(v).iter().map(|&r| p.chip_bg.on(r)).collect();
    let (word, icon) = if v.contrast {
        (p.legend, p.legend)
    } else {
        (p.pop_ink, p.ring)
    };
    [
        (MARK, word, chips.clone(), WCAG_AA),
        (MARK_ICON, icon, chips, WCAG_SHAPES),
        (MARK_LIT, p.badge_ink, vec![p.badge_bg], WCAG_AA),
        (MARK_OUT, p.pop_muted, vec![rest], WCAG_SHAPES),
        (LIT_ROW, v.sel_ink, vec![hot, ticked, both], WCAG_SHAPES),
    ]
}

/// The switch's, the progress line's and the band's pairs. High contrast draws the track in `Window` on any row; else a
/// lit row's progress takes `sel_ink`, as `ring` fades on ET66's ticked row.
fn controls(v: &LookView) -> [Pair; 6] {
    let p = &v.palette;
    let [rest, hot, ticked, both] = rows(v).map(|r| p.pop_hover.on(r));
    let (lit_ink, tracks, lit_tracks) = if v.contrast {
        (p.ring, vec![p.pop_bg], vec![p.pop_bg])
    } else {
        (v.sel_ink, vec![rest], vec![hot, ticked, both])
    };
    let band = p.pop_hover.on(p.pop_bg);
    let band_icon = if v.contrast { v.sel_ink } else { p.ring };
    [
        (SWITCH_ON, p.badge_ink, vec![p.badge_bg], WCAG_SHAPES),
        (SWITCH_OFF, p.pop_muted, vec![p.pop_bg], WCAG_SHAPES),
        (PROGRESS, p.ring, tracks, WCAG_SHAPES),
        (LIT_PROGRESS, lit_ink, lit_tracks, WCAG_SHAPES),
        (BAND, v.sel_ink, vec![band], WCAG_AA),
        (BAND_ICON, band_icon, vec![band], WCAG_SHAPES),
    ]
}
