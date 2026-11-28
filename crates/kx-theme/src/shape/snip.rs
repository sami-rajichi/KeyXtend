//! Snip's sizes (`tokens/snip.toml`, design/snip-parts.css): the frozen screen's tag, outlines and frame handles, the
//! capture bar, the hint, the strip, the countdown, the red edge, the record bar and the editor with its steps.

use serde::{Deserialize, Serialize};

use crate::theme::{FONT_WEIGHTS, outside_unit};

/// A right angle, in degrees, which an arrow's head opens less than.
const RIGHT_DEG: f32 = 90.0;

/// Snip's windows, in logical pixels, never scaled with the keyboard; the editor's steps too, which it multiplies by the
/// capture screen's display scale into the picture's own pixels.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnipShape {
    /// Size tag text size.
    pub tag_px: f32,
    /// Size tag text weight.
    pub tag_weight: u16,
    /// Size tag padding: top and bottom, sides.
    pub tag_pad_px: [f32; 2],
    /// Size tag corner radius.
    pub tag_radius_px: f32,
    /// How far the size tag's top sits above the region.
    pub tag_gap_px: f32,
    /// The region's edge line.
    pub hole_px: f32,
    /// The ring round the window under the pointer.
    pub hover_px: f32,
    /// That ring's corner radius.
    pub hover_radius_px: f32,
    /// A frame handle's square.
    pub handle_px: f32,
    /// A frame handle's click area across, never smaller than its square.
    pub handle_hit_px: f32,
    /// The capture bar's gap from its screen's top.
    pub bar_top_px: f32,
    /// The plate's padding round its keycaps.
    pub plate_pad_px: f32,
    /// A keycap's height.
    pub key_px: f32,
    /// A keycap's side padding: with a word, then with an icon alone.
    pub key_pad_px: [f32; 2],
    /// A keycap's word size.
    pub word_px: f32,
    /// A keycap's icon size.
    pub icon_px: f32,
    /// The gap between a keycap's icon and word.
    pub icon_gap_px: f32,
    /// The gap between key groups.
    pub sep_px: f32,
    /// The ring inside the picked way or a sound that records.
    pub ring_px: f32,
    /// The hint's gap under the bar.
    pub hint_gap_px: f32,
    /// The hint's padding: top and bottom, sides.
    pub hint_pad_px: [f32; 2],
    /// The hint's text size.
    pub hint_px: f32,
    /// The gap between a part and the strip or the record bar.
    pub gap_px: f32,
    /// The countdown's disc.
    pub count_px: f32,
    /// The countdown's number size.
    pub count_num_px: f32,
    /// The countdown's number weight.
    pub count_weight: u16,
    /// The countdown's word size.
    pub count_word_px: f32,
    /// The countdown's word width.
    pub count_word_w_px: f32,
    /// The gap between the disc and its Cancel keycap.
    pub count_gap_px: f32,
    /// The red edge's line outside a recorded part.
    pub edge_px: f32,
    /// The glow fading out beyond that line.
    pub glow_px: f32,
    /// The glow's strength where it starts.
    pub glow_share: f32,
    /// The countdown's dashed outline.
    pub soon_px: f32,
    /// That outline's dash and gap.
    pub dash_px: [f32; 2],
    /// The record bar's height.
    pub rec_px: f32,
    /// The record bar's padding: at its end, at its start.
    pub rec_pad_px: [f32; 2],
    /// The gap between the record bar's parts.
    pub rec_gap_px: f32,
    /// A record bar key's height.
    pub rec_key_px: f32,
    /// A record bar key's side padding.
    pub rec_key_pad_px: f32,
    /// The record dot.
    pub rec_dot_px: f32,
    /// The time's text size.
    pub rec_time_px: f32,
    /// The time's weight.
    pub rec_time_weight: u16,
    /// The gap between the dot and the time.
    pub rec_time_gap_px: f32,
    /// How far a separator on the record bar stays from the bar's top and bottom.
    pub rec_sep_px: f32,
    /// The editor's default width and height (mock-up `ED`).
    pub editor_px: [f32; 2],
    /// The least the editor shrinks to, so its top row's keys still fit on one line.
    pub editor_min_px: [f32; 2],
    /// A pen's and a shape's line, per thickness step, thinnest first.
    pub pen_px: Vec<u16>,
    /// The Highlighter's width, per thickness step.
    pub marker_px: Vec<u16>,
    /// A step's disc, across, per thickness step.
    pub disc_px: Vec<u16>,
    /// How far an arrow's head reaches back from its tip, per thickness step.
    pub head_px: Vec<u16>,
    /// The text heights, smallest first.
    pub text_px: Vec<u16>,
    /// How far a click may miss a drawing, in window pixels.
    pub slop_px: u16,
    /// How much the Highlighter's ink covers.
    pub marker_share: f32,
    /// How far each side of an arrow's head opens from its shaft, in degrees.
    pub head_deg: f32,
    /// The weight of typed text and step numbers on the picture.
    pub mark_weight: u16,
    /// A step number's size as a share of its disc.
    pub step_share: f32,
    /// The editor's title bar, tools row and options row heights.
    pub editor_rows_px: [f32; 3],
    /// The editor's rows' side padding, and the margin round its picture.
    pub editor_pad_px: [f32; 2],
    /// An ink's swatch across, its edge, and the gap between swatches.
    pub ink_px: [f32; 3],
    /// The close question's width and padding.
    pub ask_px: [f32; 2],
    /// The Snip panel's width.
    pub panel_px: f32,
    /// The gap between typed text and its dashed outline, in window pixels.
    pub typing_gap_px: f32,
}

impl SnipShape {
    /// Refuses, naming the size: an odd weight or share, a drawn size of 0 or less, a negative gap or padding, uneven or
    /// empty steps, an edge and glow wider than the gap, a separator with no line left, a handle's click area smaller
    /// than its square, an editor whose least size passes its default, and an arrow's head that does not open.
    pub(super) fn check(&self) -> Result<(), String> {
        self.check_numbers()?;
        self.check_steps()?;
        let (edge, glow, gap) = (self.edge_px, self.glow_px, self.gap_px);
        if edge + glow > gap {
            return Err(format!(
                "shape: snip.edge_px {edge} and snip.glow_px {glow} must fit in snip.gap_px {gap}, so the bars clear the edge"
            ));
        }
        let (sep, bar) = (self.rec_sep_px, self.rec_px);
        if 2.0 * sep >= bar {
            return Err(format!(
                "shape: snip.rec_sep_px {sep} must leave a line on the {bar} px record bar"
            ));
        }
        let (square, hit) = (self.handle_px, self.handle_hit_px);
        if hit < square {
            return Err(format!(
                "shape: snip.handle_hit_px {hit} must hold the {square} px handle"
            ));
        }
        self.check_editor()
    }

    /// Weights from 100 to 900, shares from 0 to 1, every drawn size above 0, and every gap and padding 0 or more.
    fn check_numbers(&self) -> Result<(), String> {
        let weights = [
            ("snip.tag_weight", self.tag_weight),
            ("snip.count_weight", self.count_weight),
            ("snip.rec_time_weight", self.rec_time_weight),
            ("snip.mark_weight", self.mark_weight),
        ];
        if let Some((n, w)) = weights.iter().find(|(_, w)| !FONT_WEIGHTS.contains(w)) {
            return Err(format!("shape: {n} {w} is not 100 to 900"));
        }
        let shares = [
            ("snip.glow_share", self.glow_share),
            ("snip.marker_share", self.marker_share),
            ("snip.step_share", self.step_share),
        ];
        if let Some((n, v)) = outside_unit(&shares) {
            return Err(format!("shape: {n} {v} is not 0 to 1"));
        }
        if let Some((n, v)) = self
            .drawn()
            .into_iter()
            .find(|(_, v)| !(v.is_finite() && *v > 0.0))
        {
            return Err(format!("shape: {n} {v} must be above 0"));
        }
        match self
            .spaces()
            .into_iter()
            .find(|(_, v)| !(v.is_finite() && *v >= 0.0))
        {
            Some((n, v)) => Err(format!("shape: {n} {v} must be 0 or more")),
            None => Ok(()),
        }
    }

    /// The pen's, Highlighter's, disc's and head's lists hold one size per step, the text heights their own count, and
    /// every list and the slop sizes above 0.
    fn check_steps(&self) -> Result<(), String> {
        let steps = self.pen_px.len();
        let thick = [
            ("snip.marker_px", &self.marker_px),
            ("snip.disc_px", &self.disc_px),
            ("snip.head_px", &self.head_px),
        ];
        let own = [
            ("snip.pen_px", &self.pen_px),
            ("snip.text_px", &self.text_px),
        ];
        let mut all = own.iter().chain(thick.iter());
        if let Some((n, l)) = all.find(|(_, l)| l.is_empty() || l.contains(&0)) {
            return Err(format!("shape: {n} {l:?} must hold sizes above 0"));
        }
        if let Some((n, l)) = thick.iter().find(|(_, l)| l.len() != steps) {
            return Err(format!(
                "shape: {n} {l:?} must hold one size per step, as snip.pen_px holds {steps}"
            ));
        }
        if self.slop_px == 0 {
            return Err("shape: snip.slop_px must be 1 or more".into());
        }
        Ok(())
    }

    /// The editor's least size must be above 0 and no larger than its default, side by side, and an arrow's head must
    /// open, short of a right angle.
    fn check_editor(&self) -> Result<(), String> {
        let head = self.head_deg;
        if !(head > 0.0 && head < RIGHT_DEG) {
            return Err(format!(
                "shape: snip.head_deg {head} must be above 0 and under {RIGHT_DEG}"
            ));
        }
        let (least, size) = (self.editor_min_px, self.editor_px);
        let fits = least
            .iter()
            .zip(size)
            .all(|(l, s)| l.is_finite() && *l > 0.0 && *l <= s);
        if fits {
            return Ok(());
        }
        Err(format!(
            "shape: snip.editor_min_px {least:?} must be above 0 and at most snip.editor_px {size:?}"
        ))
    }
}
