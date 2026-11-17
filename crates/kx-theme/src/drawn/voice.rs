//! What the dictation bar draws beyond the shared panel pairs (VoiceBar.qml, VoiceWave.qml): the red dot, the plain
//! meter's fill and the language chip. The wave's bars are `ring on pop_bg`, and Stop's icon is the main button's.

use super::{Drawn, worst};
use crate::contrast::{WCAG_AA, WCAG_SHAPES};
use crate::view::LookView;

/// The red dot while the microphone listens.
pub(super) const DOT: &str = "voice red dot";
/// The plain meter's fill on its track, under reduced motion.
pub(super) const METER: &str = "voice meter on its track";
/// The language's name on its chip.
pub(super) const CHIP: &str = "voice language chip";

/// The bar's own pairs; high contrast draws the dot in Highlight, the meter's track in Window and the chip's name in
/// `ButtonText`, as system colours.
pub(super) fn voice(v: &LookView, out: &mut Vec<Drawn>) {
    let p = &v.palette;
    let (dot, track, name) = if v.contrast {
        (p.ring, p.pop_bg, p.legend)
    } else {
        (v.common.rec_dot, p.pop_hover.on(p.pop_bg), p.pop_ink)
    };
    let pairs = [
        (DOT, dot, p.pop_bg, WCAG_SHAPES),
        (METER, p.ring, track, WCAG_SHAPES),
        (CHIP, name, p.chip_bg.on(p.pop_bg), WCAG_AA),
    ];
    for (name, ink, bg, min) in pairs {
        out.push(worst(name, ink, 1.0, &[bg], min));
    }
}
