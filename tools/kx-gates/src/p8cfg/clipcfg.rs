//! `[g8_clip]`: G8's clipboard part, the history filled with gate copies and then panel rounds.

use kx_mod_clipboard::ClipSettings;
use kx_module_api::ports::{MB, fits_iso};
use serde::Deserialize;

use super::{as_is, wait};
use crate::clipfill;
use crate::clipnames::PageNames;
use crate::config::GatesConfig;

/// `[g8_clip]`: the copies, the rounds and the waits; times in ms.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct G8Clip {
    /// Copies that fill the history.
    pub entries: usize,
    /// Every this many copies one is a picture.
    pub image_every: usize,
    /// The pictures' sides in turn, in px.
    pub image_px: Vec<u32>,
    /// The long texts' lengths in turn, in characters.
    pub text_chars: Vec<usize>,
    /// Start of every text copy.
    pub prefix: String,
    /// Short copies at the end, the only ones Windows' own history then holds.
    pub forget_tail: usize,
    /// Pause after each copy.
    pub copy_gap_ms: u64,
    /// Wait after the last copy, and before each memory reading.
    pub settle_ms: u64,
    /// Panel rounds before the first judged reading.
    pub warm_rounds: usize,
    /// Panel rounds measured after it.
    pub rounds: usize,
    /// Pages turned in each round.
    pub pages: usize,
    /// Typed into the search line in each round.
    pub search: String,
    /// The language of the layout the search is typed on.
    pub search_lang: String,
    /// Longest wait for the panel to open or close.
    pub panel_wait_ms: u64,
    /// Pause after each page turn and each search step.
    pub pause_ms: u64,
    /// Longest wait after the fill for the page count to show every entry.
    pub count_wait_ms: u64,
    /// kx-ui's text of the pager's page number.
    pub page_name: String,
    /// The variable in that text that holds the page count.
    pub page_var: String,
}

impl G8Clip {
    /// Refuses a count outside the product's history, a tail that leaves no long copy, and no copy every so often.
    fn check_counts(&self, d: &ClipSettings) -> Result<(), String> {
        if !(1..=d.max_entries).contains(&self.entries) {
            let most = d.max_entries;
            return Err(format!(
                "g8_clip.entries must be 1 to {most}, the product's default max_entries"
            ));
        }
        if !(1..self.entries).contains(&self.forget_tail) {
            return Err("g8_clip.forget_tail must be 1 to fewer than g8_clip.entries".to_string());
        }
        if !(1..=self.entries).contains(&self.image_every) {
            return Err("g8_clip.image_every must be 1 to g8_clip.entries".to_string());
        }
        if self.rounds == 0 || self.pages == 0 {
            let key = if self.rounds == 0 { "rounds" } else { "pages" };
            return Err(format!("g8_clip.{key} must be above 0"));
        }
        Ok(())
    }

    /// Refuses a copy the product would not keep whole: a picture past `max_pixels` or `max_entry_mb`, a text past
    /// `max_entry_mb` or too short to hold its prefix and number, and a short copy the clean-up never reads.
    fn check_sizes(&self, c: &GatesConfig, d: &ClipSettings) -> Result<(), String> {
        let bytes = u64::from(d.max_entry_mb).saturating_mul(MB);
        let least = clipfill::least_chars(&self.prefix, self.entries);
        as_is("g8_clip.prefix", &self.prefix, c.probes.forget_max_chars)?;
        if least > c.probes.forget_max_chars {
            let most = c.probes.forget_max_chars;
            return Err(format!(
                "g8_clip.prefix with a copy's number must fit probes.forget_max_chars ({most})"
            ));
        }
        let kept = |px: &u32| {
            let side = u64::from(*px);
            *px > 0 && side * side <= d.max_pixels && clipfill::dib_len(*px) <= bytes
        };
        if self.image_px.is_empty() || !self.image_px.iter().all(kept) {
            return Err(
                "g8_clip.image_px: each side above 0, within max_pixels and max_entry_mb"
                    .to_string(),
            );
        }
        let fits = |n: &usize| {
            *n >= least && clipfill::text_bytes(&self.prefix, self.entries, *n) <= bytes
        };
        if self.text_chars.is_empty() || !self.text_chars.iter().all(fits) {
            return Err(format!(
                "g8_clip.text_chars: each from {least} characters, within max_entry_mb"
            ));
        }
        Ok(())
    }

    /// Refuses bad counts or sizes, an empty search, a search layout that is no language, a wait past the guard, and a
    /// page text without its count in some language.
    pub(super) fn check(&self, c: &GatesConfig, d: &ClipSettings) -> Result<(), String> {
        self.check_counts(d)?;
        self.check_sizes(c, d)?;
        as_is("g8_clip.search", &self.search, d.search_chars)?;
        if !fits_iso(&self.search_lang) {
            return Err("g8_clip.search_lang must be a language code".to_string());
        }
        let limit = c.mouse_guard.limit_ms;
        let waits = [
            ("g8_clip.copy_gap_ms", self.copy_gap_ms),
            ("g8_clip.settle_ms", self.settle_ms),
            ("g8_clip.panel_wait_ms", self.panel_wait_ms),
            ("g8_clip.pause_ms", self.pause_ms),
            ("g8_clip.count_wait_ms", self.count_wait_ms),
        ];
        waits
            .iter()
            .try_for_each(|(key, ms)| wait(key, *ms, limit))?;
        PageNames::load(self)
            .map(drop)
            .map_err(|e| format!("g8_clip.page_name: {e}"))
    }
}
