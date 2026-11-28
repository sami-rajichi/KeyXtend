//! Snip's sizes by kind, for its checks: those that draw something, and the gaps, paddings and radii that may be 0.

use super::SnipShape;

impl SnipShape {
    /// Every size that draws something, by name: a pair's parts each under the pair's name.
    pub(super) fn drawn(&self) -> Vec<(&'static str, f32)> {
        let [dash, gap] = self.dash_px;
        let [ink, edge, _] = self.ink_px;
        let rows = self.editor_rows_px.map(|v| ("snip.editor_rows_px", v));
        let mut all = vec![
            ("snip.ink_px", ink),
            ("snip.ink_px", edge),
            ("snip.ask_px", self.ask_px[0]),
            ("snip.panel_px", self.panel_px),
        ];
        all.extend(rows);
        all.extend([
            ("snip.tag_px", self.tag_px),
            ("snip.hole_px", self.hole_px),
            ("snip.hover_px", self.hover_px),
            ("snip.handle_px", self.handle_px),
            ("snip.handle_hit_px", self.handle_hit_px),
            ("snip.key_px", self.key_px),
            ("snip.word_px", self.word_px),
            ("snip.icon_px", self.icon_px),
            ("snip.ring_px", self.ring_px),
            ("snip.hint_px", self.hint_px),
            ("snip.count_px", self.count_px),
            ("snip.count_num_px", self.count_num_px),
            ("snip.count_word_px", self.count_word_px),
            ("snip.count_word_w_px", self.count_word_w_px),
            ("snip.edge_px", self.edge_px),
            ("snip.soon_px", self.soon_px),
            ("snip.dash_px", dash),
            ("snip.dash_px", gap),
            ("snip.rec_px", self.rec_px),
            ("snip.rec_key_px", self.rec_key_px),
            ("snip.rec_dot_px", self.rec_dot_px),
            ("snip.rec_time_px", self.rec_time_px),
        ]);
        all
    }

    /// Every gap, padding and radius, which may be 0, by name: a pair's parts each under the pair's name.
    pub(super) fn spaces(&self) -> Vec<(&'static str, f32)> {
        let pairs = [
            ("snip.tag_pad_px", self.tag_pad_px),
            ("snip.key_pad_px", self.key_pad_px),
            ("snip.hint_pad_px", self.hint_pad_px),
            ("snip.rec_pad_px", self.rec_pad_px),
        ];
        let ones = [
            ("snip.tag_radius_px", self.tag_radius_px),
            ("snip.tag_gap_px", self.tag_gap_px),
            ("snip.hover_radius_px", self.hover_radius_px),
            ("snip.bar_top_px", self.bar_top_px),
            ("snip.plate_pad_px", self.plate_pad_px),
            ("snip.icon_gap_px", self.icon_gap_px),
            ("snip.sep_px", self.sep_px),
            ("snip.hint_gap_px", self.hint_gap_px),
            ("snip.gap_px", self.gap_px),
            ("snip.count_gap_px", self.count_gap_px),
            ("snip.glow_px", self.glow_px),
            ("snip.rec_gap_px", self.rec_gap_px),
            ("snip.rec_key_pad_px", self.rec_key_pad_px),
            ("snip.rec_time_gap_px", self.rec_time_gap_px),
            ("snip.rec_sep_px", self.rec_sep_px),
            ("snip.ink_px", self.ink_px[2]),
            ("snip.ask_px", self.ask_px[1]),
            ("snip.typing_gap_px", self.typing_gap_px),
        ];
        let pads = [("snip.editor_pad_px", self.editor_pad_px)];
        let split = pairs
            .into_iter()
            .chain(pads)
            .flat_map(|(n, [a, b])| [(n, a), (n, b)]);
        ones.into_iter().chain(split).collect()
    }
}
