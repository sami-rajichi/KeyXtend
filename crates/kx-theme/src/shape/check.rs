//! The rules shared sizes must keep: weights, shares and sizes that make sense.

use super::Shape;
use crate::theme::{FONT_WEIGHTS, outside_unit};

impl Shape {
    /// Refuses a weight outside 100 to 900, a share outside 0 to 1, a shimmer span, outline, smallest label, row or mark
    /// of 0 or less, a panel gap, label padding or mark gap below 0, and a lit dot no larger than the others, naming the
    /// size.
    pub(crate) fn check(&self) -> Result<(), String> {
        self.check_weights()?;
        self.check_sizes()?;
        self.check_dots()?;
        self.check_shares()
    }

    /// Cycle's lit dot must be larger than an unlit one, which is how it stands out.
    fn check_dots(&self) -> Result<(), String> {
        let c = &self.lang.cycle;
        if c.dot_on_px > c.dot_px {
            return Ok(());
        }
        let (on, off) = (c.dot_on_px, c.dot_px);
        Err(format!(
            "shape: lang.cycle.dot_on_px {on} must be above dot_px {off}"
        ))
    }

    /// Font weights must be 100 to 900.
    fn check_weights(&self) -> Result<(), String> {
        let weights = [
            ("bar.weight", self.bar.weight),
            ("bar.lit_weight", self.bar.lit_weight),
            ("chip.weight", self.chip.weight),
            ("legend.corner_weight", self.legend.corner_weight),
            ("lang.weight", self.lang.weight),
            ("lang.side_weight", self.lang.side_weight),
            ("tip.weight", self.tip.weight),
            ("pill.weight", self.pill.weight),
            ("caption.status_weight", self.caption.status_weight),
            ("snip.tag_weight", self.snip.tag_weight),
            ("panel.title_weight", self.panel.title_weight),
            ("panel.count_weight", self.panel.count_weight),
            ("notice.button_weight", self.notice.button_weight),
            ("list.badge_weight", self.list.badge_weight),
        ];
        match weights.iter().find(|(_, w)| !FONT_WEIGHTS.contains(w)) {
            Some((n, w)) => {
                let (lo, hi) = (FONT_WEIGHTS.start(), FONT_WEIGHTS.end());
                Err(format!("shape: {n} {w} is not {lo} to {hi}"))
            }
            None => Ok(()),
        }
    }

    /// The shimmer span, the outline, the smallest label, the row and the mark's size must be above 0, and the panel
    /// gaps, the label padding, the language icon's gap and the mark's 0 or more, all finite.
    fn check_sizes(&self) -> Result<(), String> {
        let above = [
            ("caption.shimmer_span", self.caption.shimmer_span),
            ("hc_outline.px", self.hc_outline.px),
            ("legend.label_min_px", self.legend.label_min_px),
            ("drawn.row_px", self.drawn.row_px),
            ("bar.mark_px", self.bar.mark_px[0]),
            ("bar.mark_px", self.bar.mark_px[1]),
        ];
        if let Some((n, v)) = above.iter().find(|(_, v)| !(v.is_finite() && *v > 0.0)) {
            return Err(format!("shape: {n} {v} must be above 0"));
        }
        let gaps = self.panel.gap_px.iter().map(|g| ("panel.gap_px", *g));
        let mut zero_up = gaps.chain([
            ("legend.label_pad_px", self.legend.label_pad_px),
            ("lang.icon_gap_px", self.lang.icon_gap_px),
            ("bar.mark_px", self.bar.mark_px[2]),
        ]);
        match zero_up.find(|(_, v)| !(v.is_finite() && *v >= 0.0)) {
            Some((n, v)) => Err(format!("shape: {n} {v} must be 0 or more")),
            None => Ok(()),
        }
    }

    /// Shares must be 0 to 1.
    fn check_shares(&self) -> Result<(), String> {
        let shares = [
            ("bar.radius_share", self.bar.radius_share),
            ("legend.corner_scale", self.legend.corner_scale[0]),
            ("legend.corner_scale", self.legend.corner_scale[1]),
            ("lang.side", self.lang.side[1]),
            ("lang.back_share", self.lang.back_share),
            ("dpad.icon_pad", self.dpad.icon_pad),
            ("dpad.hub", self.dpad.hub),
            ("dpad.stop", self.dpad.stop),
            ("press.scale", self.press.scale),
            ("panel.radius_share", self.panel.radius_share),
            ("panel.off_share", self.panel.off_share),
            ("ring.core_share", self.ring.core_share),
            ("lang.cycle.icon_share", self.lang.cycle.icon_share),
            ("lang.cycle.dot_off", self.lang.cycle.dot_off),
            ("lang.list.icon_share", self.lang.list.icon_share),
            ("dim.share", self.dim.share),
            ("notice.radius_share", self.notice.radius_share),
            ("menu.icon_share", self.menu.icon_share),
            ("tray.radius_share", self.tray.radius_share),
            ("tray.glyph_share", self.tray.glyph_share),
            ("backdrop.plate", self.backdrop.plate),
        ];
        match outside_unit(&shares) {
            Some((n, v)) => Err(format!("shape: {n} {v} is not 0 to 1")),
            None => Ok(()),
        }
    }
}
