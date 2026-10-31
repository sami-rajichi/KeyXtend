//! `kx-gates-p8.toml`: G20 on the product (`[g20_product]`) and G8's clipboard part (`[g8_clip]`).
#![cfg(windows)]

use kx_mod_clipboard::ClipSettings;
use kx_module_api::Settings;
use kx_module_api::is_hidden;
use kx_module_api::ports::fits_iso;
use kx_module_api::text::fold_any;
use serde::Deserialize;

use crate::config::GatesConfig;
use crate::fit::held_fits;
use crate::g22cfg::plain_name;
use crate::p5cfg::SEP;
use crate::p7cfg::{backup_free, gate_files};
use crate::{clipdata, ftl, g20plan};

/// The least ticks G20 makes, so its insert pastes a join between two texts.
pub const JOINED: usize = 2;
/// The app name the product gives the gate's own copies: kx-gates' program, which carries no description.
const OWN_APP: &str = env!("CARGO_PKG_NAME");

/// `[g20_product]`: the copies, the searches, the panel's gate ids and texts, and the waits; times in ms.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct G20Product {
    /// The out folder's file that keeps the test build's settings bytes during a run.
    pub backup: String,
    /// The file that marks a data folder the gate made.
    pub marker: String,
    /// One plain copy per language, oldest first.
    pub copies: Vec<Line>,
    /// Copied with each exclusion marker; it never shows.
    pub marked: String,
    /// Copied, then cleared at once; it goes.
    pub cleared: String,
    /// Copied in the skipped app; it never shows.
    pub skipped: String,
    /// The run's skipped app, by the product's app id.
    pub skip_app: String,
    /// Searches typed with the product's keys.
    pub searches: Vec<Search>,
    /// The panel's gate-id group.
    pub group: String,
    /// The list's id.
    pub list: String,
    /// A row's id before its place.
    pub row: String,
    /// The search line's id.
    pub search_line: String,
    /// Insert's id.
    pub insert: String,
    /// The page-on button's id.
    pub page_on: String,
    /// kx-ui's text of Insert's label.
    pub insert_name: String,
    /// kx-ui's text of a row's tick note.
    pub ticked_name: String,
    /// kx-ui's text before a row's tick note.
    pub join_name: String,
    /// The count's variable in those texts.
    pub count_var: String,
    /// Wait after the copy's keyboard shows before its first copy.
    pub ready_ms: u64,
    /// Pause after each copy.
    pub copy_gap_ms: u64,
    /// Longest wait for the panel to open or close.
    pub panel_wait_ms: u64,
    /// Pause after each panel click or search.
    pub settle_ms: u64,
}

/// One plain copy: its language and its text.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Line {
    /// The text's language, by ISO 639 code.
    pub lang: String,
    /// What the gate copies.
    pub text: String,
}

/// One search: the layout's language, what is typed, and whether its first row is ticked.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Search {
    /// The layout's language, by ISO 639 code.
    pub lang: String,
    /// What the product's keys type into the search line.
    pub typed: String,
    /// The first row found is ticked.
    pub tick: bool,
}

pub use clipcfg::G8Clip;

impl G20Product {
    /// The gate id of panel part `part`.
    pub fn id(&self, part: &str) -> String {
        format!("{}{SEP}{part}", self.group)
    }

    /// The gate id of the row at `place` on the page, 0 the top.
    pub fn row_id(&self, place: usize) -> String {
        self.id(&format!("{}{place}", self.row))
    }

    /// The plain copies' texts, oldest first.
    pub fn texts(&self) -> Vec<&str> {
        self.copies.iter().map(|k| k.text.as_str()).collect()
    }

    /// The texts that must never show: the marked, the cleared and the skipped copy.
    pub fn hidden(&self) -> [&str; 3] {
        [&self.marked, &self.cleared, &self.skipped]
    }

    /// Refuses no copy, a bad language, a text the panel would not show as it is, and two equal texts or languages.
    fn check_texts(&self, most: usize) -> Result<(), String> {
        if self.copies.is_empty() {
            return Err("g20_product.copies must hold at least one copy".to_string());
        }
        for k in &self.copies {
            if !fits_iso(&k.lang) {
                return Err(format!(
                    "g20_product.copies: {:?} is no language code",
                    k.lang
                ));
            }
            as_is("g20_product.copies", &k.text, most)?;
        }
        let hidden = [
            ("marked", &self.marked),
            ("cleared", &self.cleared),
            ("skipped", &self.skipped),
        ];
        for (key, text) in hidden {
            as_is(&format!("g20_product.{key}"), text, most)?;
        }
        let langs: Vec<&str> = self.copies.iter().map(|k| k.lang.as_str()).collect();
        let texts: Vec<&str> = self.texts().into_iter().chain(self.hidden()).collect();
        if !distinct(&langs) || !distinct(&texts) {
            return Err(
                "g20_product.copies: each language once, and every text differs".to_string(),
            );
        }
        Ok(())
    }

    /// Refuses a search that types no letters, finds no row or matches the gate's own app name, and ticks that are
    /// fewer than `JOINED` or pick one row twice.
    fn check_searches(&self) -> Result<(), String> {
        let rows = g20plan::shown(&self.texts());
        let own = fold_any(OWN_APP);
        let bad = |s: &Search| {
            let letters = !s.typed.is_empty() && s.typed.chars().all(char::is_alphabetic);
            let mine = own.contains(fold_any(&s.typed).trim());
            !fits_iso(&s.lang) || !letters || mine || g20plan::found(&rows, &s.typed).is_empty()
        };
        if self.searches.is_empty() || self.searches.iter().any(bad) {
            return Err(format!(
                "g20_product.searches: each needs a language code and letters that find a copy, never {OWN_APP}"
            ));
        }
        let ticked = g20plan::ticked(self);
        let picked: Vec<&str> = ticked.iter().map(String::as_str).collect();
        let ticks = self.searches.iter().filter(|s| s.tick).count();
        if ticks < JOINED || !distinct(&picked) {
            return Err(format!(
                "g20_product.searches: at least {JOINED} ticks, each on another row"
            ));
        }
        Ok(())
    }

    /// Refuses an empty or dotted group, an empty id, a marker that is no plain name or that the product uses, and a
    /// text kx-ui lacks.
    fn check_ids(&self) -> Result<(), String> {
        if self.group.is_empty() || self.group.contains(SEP) {
            return Err(format!("g20_product.group must be a name without {SEP}"));
        }
        let ids = [
            ("list", &self.list),
            ("row", &self.row),
            ("search_line", &self.search_line),
            ("insert", &self.insert),
            ("page_on", &self.page_on),
        ];
        if let Some((key, _)) = ids.iter().find(|(_, id)| id.is_empty()) {
            return Err(format!("g20_product.{key} must not be empty"));
        }
        if !plain_name(&self.marker) || clipdata::taken(&self.marker) {
            return Err(
                "g20_product.marker must be a plain file name the product never writes in its data folder"
                    .to_string(),
            );
        }
        let named = |key: &str, e: String| format!("g20_product.{key}: {e}");
        ftl::around(&self.insert_name, &self.count_var).map_err(|e| named("insert_name", e))?;
        ftl::around(&self.ticked_name, &self.count_var).map_err(|e| named("ticked_name", e))?;
        ftl::every(&self.join_name)
            .map(drop)
            .map_err(|e| named("join_name", e))
    }

    /// Refuses bad texts, searches, ids, a skipped app the product refuses, a wait past `most_ms`, and a backup a gate
    /// deletes or the owner's.
    fn check(&self, c: &GatesConfig, d: &ClipSettings) -> Result<(), String> {
        self.check_texts(d.title_chars.min(c.probes.forget_max_chars))?;
        self.check_searches()?;
        self.check_ids()?;
        let skip = ClipSettings {
            skip_apps: vec![self.skip_app.clone()],
            ..d.clone()
        };
        skip.check()
            .map_err(|e| format!("g20_product.skip_app: {e}"))?;
        let waits = [
            ("g20_product.ready_ms", self.ready_ms),
            ("g20_product.copy_gap_ms", self.copy_gap_ms),
            ("g20_product.panel_wait_ms", self.panel_wait_ms),
            ("g20_product.settle_ms", self.settle_ms),
        ];
        waits
            .iter()
            .try_for_each(|(key, ms)| wait(key, *ms, c.mouse_guard.limit_ms))?;
        if !plain_name(&self.backup) {
            return Err(
                "g20_product.backup must be a plain file name, in the out folder".to_string(),
            );
        }
        let mut taken = gate_files(c).to_vec();
        taken.push(&c.owner_settings.backup);
        backup_free(&self.backup, &taken).map_err(|_| {
            format!(
                "g20_product.backup ({}) must be no file a gate deletes or writes, nor the owner's backup",
                self.backup
            )
        })
    }
}

/// Refuses `text` under `key` unless the panel shows it as it is: one line of 1 to `most` characters, no hidden mark.
pub(crate) fn as_is(key: &str, text: &str, most: usize) -> Result<(), String> {
    let n = text.chars().count();
    if (1..=most).contains(&n) && !text.chars().any(is_hidden) {
        return Ok(());
    }
    Err(format!(
        "{key} must be one line of 1 to {most} characters, no hidden mark"
    ))
}

/// True when no two of `items` are equal.
fn distinct(items: &[&str]) -> bool {
    let mut sorted = items.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    sorted.len() == items.len()
}

/// Refuses a wait of `ms` outside 1 ms to `most`, the mouse guard's limit, naming `key`.
pub(crate) fn wait(key: &str, ms: u64, most: u64) -> Result<(), String> {
    if (1..=most).contains(&ms) {
        return Ok(());
    }
    Err(format!(
        "{key} must be 1 to {most} ms, the mouse guard's limit"
    ))
}

/// Refuses P8 settings that could copy nothing the panel shows, tick the wrong rows, fill past the product's limits,
/// wait past the mouse guard, or keep the settings in a file a gate deletes.
pub fn check(c: &GatesConfig) -> Result<(), String> {
    let d = ClipSettings::from_defaults().map_err(|e| format!("the clipboard defaults: {e}"))?;
    c.g20_product.check(c, &d)?;
    c.g8_clip.check(c, &d)?;
    held_fits(c, "g20_product: G20 on the product", runs::g20_ms(c))?;
    held_fits(c, "g8_clip: g8 clip", runs::clip_ms(c))
}

mod clipcfg;
pub(crate) mod runs;

#[cfg(test)]
mod tests;
