//! `kx-gates-p9.toml`: G23 on the product (`[g23]`), a test Login saved through Quick-fill and typed into Chrome.
#![cfg(windows)]

use kx_mod_keyboard::KeyboardSettings;
use kx_mod_vault::settings::TAB_GAP_MS;
use kx_module_api::keys;
use kx_module_api::ports::fits_iso;
use serde::Deserialize;

use crate::apps::AppKind;
use crate::config::GatesConfig;
use crate::fit::{held_fits, times, total};
use crate::g23::OFFER_ASK;
use crate::p5cfg::SEP;
use crate::p7cfg::runs::end_ms;
use crate::p8cfg::wait;
use crate::{ftl, product};

/// The fills G23 judges: Both, then Password and User, each on its own page.
pub const FILLS: usize = 3;
/// The panel clicks that save the entry: the fill key, New, the kind, three lines, Save and the fill key again.
const SEED_CLICKS: usize = 8;
/// The layout asks: the test language's before the seed, and the first layout's after it.
const SWITCHES: usize = 2;
/// The lines the seed types: the name, the user and the password.
const LINES: usize = 3;
/// The seed's panel wait beyond its clicks: for Save's page to go.
const SEED_WAITS: usize = 1;
/// The clicks around the fills, as the panel stays open: the fill key before the first, its close after the last.
const OPEN_CLOSE: usize = 2;
/// The panel wait beyond the clicks after the last fill: for the panel to go.
const SHUT_WAITS: usize = 1;
/// The waits for the backup offer: for it to show, and for it to go after Later.
const NOTICE_WAITS: usize = 2;
/// What a title mark may not hold: quotes and markup, which would end the page's script or its tags.
const NOT_IN_MARK: &str = "'\"\\<>&";

/// `[g23]`: the test entry, the panel's gate ids and texts, the page's title marks and the waits; times in ms.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct G23 {
    /// The test entry's name.
    pub name: String,
    /// Its test user.
    pub user: String,
    /// Its test password.
    pub password: String,
    /// The layout the letters are typed in, by ISO 639 code.
    pub lang: String,
    /// The Login kind's id.
    pub kind: String,
    /// The user field's place on the New page.
    pub user_field: usize,
    /// The password field's place on the New page.
    pub password_field: usize,
    /// The panel's gate-id group.
    pub group: String,
    /// The header's close, which every page shows.
    pub close: String,
    /// A row's id before its place.
    pub row: String,
    /// A field's id before its place.
    pub field: String,
    /// A kind chip's id before its kind.
    pub kind_chip: String,
    /// The part between a place and a button's id.
    pub part: String,
    /// The header's New.
    pub new: String,
    /// The name line.
    pub name_line: String,
    /// A field's value line.
    pub value_line: String,
    /// Save.
    pub save: String,
    /// The row's Both.
    pub both: String,
    /// The row's User.
    pub user_button: String,
    /// The row's Password.
    pub password_button: String,
    /// kx-ui's title of the notice banner's window.
    pub notice_title: String,
    /// The title mark before the password's length.
    pub length_mark: String,
    /// The title mark before the Enter key-downs.
    pub enter_mark: String,
    /// The title mark before the submits.
    pub submit_mark: String,
    /// The DOM key name the page counts as Enter.
    pub enter_key: String,
    /// Wait after the copy's keyboard shows before the first click.
    pub ready_ms: u64,
    /// Longest wait for the panel or a part of it to show or go.
    pub panel_wait_ms: u64,
    /// Pause after each panel click and each typed line.
    pub settle_ms: u64,
    /// Pause after a fill's text arrived and before it is judged; longer than the latest a part after a Tab can go.
    pub late_ms: u64,
    /// Longest wait for a fill's text to arrive.
    pub fill_wait_ms: u64,
    /// Longest wait for a notice to show, and to go once answered.
    pub notice_wait_ms: u64,
}

impl G23 {
    /// The gate id of panel part `part`.
    pub fn id(&self, part: &str) -> String {
        format!("{}{SEP}{part}", self.group)
    }

    /// The gate id of `button` on the row at `place`, 0 the top.
    pub fn row_button(&self, place: usize, button: &str) -> String {
        self.id(&format!("{}{place}{}{button}", self.row, self.part))
    }

    /// The gate id of the value line of the field at `place`.
    pub fn value_id(&self, place: usize) -> String {
        let (f, p, v) = (&self.field, &self.part, &self.value_line);
        self.id(&format!("{f}{place}{p}{v}"))
    }

    /// The gate id of the Login kind's chip.
    pub fn kind_id(&self) -> String {
        self.id(&format!("{}{}", self.kind_chip, self.kind))
    }

    /// The title marks, in the page's order: the length, the Enter key-downs and the submits.
    pub fn marks(&self) -> [&str; 3] {
        [&self.length_mark, &self.enter_mark, &self.submit_mark]
    }

    /// Refuses a test value no key types at once or longer than `most`, values that share a length, as the password
    /// shows only its length, and no language.
    fn check_values(&self, most: usize) -> Result<(), String> {
        let typed = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit();
        let values = [
            ("name", &self.name),
            ("user", &self.user),
            ("password", &self.password),
        ];
        for (key, v) in values {
            if !(1..=most).contains(&v.chars().count()) || !v.chars().all(typed) {
                return Err(format!(
                    "g23.{key} must be 1 to {most} lowercase letters or digits"
                ));
            }
        }
        let [name, user, password] = values.map(|(_, v)| v.chars().count());
        if name == user || user == password || name == password {
            return Err(
                "g23.name, g23.user and g23.password must each have a length of its own: the password shows only its \
                 length"
                    .to_string(),
            );
        }
        if !fits_iso(&self.lang) {
            return Err(format!("g23.lang: {:?} is no language code", self.lang));
        }
        Ok(())
    }

    /// Refuses an empty or dotted group, an empty id, and one field place for both values.
    fn check_ids(&self) -> Result<(), String> {
        if self.group.is_empty() || self.group.contains(SEP) {
            return Err(format!("g23.group must be a name without {SEP}"));
        }
        let ids = [
            ("kind", &self.kind),
            ("close", &self.close),
            ("row", &self.row),
            ("field", &self.field),
            ("kind_chip", &self.kind_chip),
            ("part", &self.part),
            ("new", &self.new),
            ("name_line", &self.name_line),
            ("value_line", &self.value_line),
            ("save", &self.save),
            ("both", &self.both),
            ("user_button", &self.user_button),
            ("password_button", &self.password_button),
        ];
        if let Some((key, _)) = ids.iter().find(|(_, id)| id.is_empty()) {
            return Err(format!("g23.{key} must not be empty"));
        }
        if self.user_field == self.password_field {
            return Err("g23.password_field must differ from g23.user_field".to_string());
        }
        Ok(())
    }

    /// Refuses a mark that is empty, holds a digit, a quote or markup, or holds an earlier mark or sits in one.
    fn check_marks(&self) -> Result<(), String> {
        let keys = ["length_mark", "enter_mark", "submit_mark"];
        let marks = self.marks();
        let bad = |m: &str| {
            m.chars()
                .any(|c| c.is_ascii_digit() || NOT_IN_MARK.contains(c))
        };
        for (i, (key, m)) in keys.iter().zip(marks).enumerate() {
            let clash = marks[..i].iter().any(|e| e.contains(m) || m.contains(e));
            if m.is_empty() || bad(m) || clash {
                return Err(format!(
                    "g23.{key} must be a mark of its own, with no digit, quote or markup"
                ));
            }
        }
        if self.enter_key.is_empty() || !self.enter_key.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err("g23.enter_key must be a DOM key name: letters only".to_string());
        }
        Ok(())
    }

    /// Refuses a text kx-ui lacks in a shipped language: the banner's title, and the notices G23 meets.
    fn check_texts(&self) -> Result<(), String> {
        ftl::every(&self.notice_title).map_err(|e| format!("g23.notice_title: {e}"))?;
        let ours = |e: String| format!("g23: {e}");
        ftl::shapes(keys::HIDDEN_RUN).map_err(ours)?;
        [keys::VAULT_OFFER, OFFER_ASK.no]
            .iter()
            .try_for_each(|k| ftl::every(k).map(drop).map_err(ours))
    }

    /// Refuses a wait outside 1 ms to `most_ms`, the mouse guard's limit.
    fn check_waits(&self, most_ms: u64) -> Result<(), String> {
        let waits = [
            ("g23.ready_ms", self.ready_ms),
            ("g23.panel_wait_ms", self.panel_wait_ms),
            ("g23.settle_ms", self.settle_ms),
            ("g23.late_ms", self.late_ms),
            ("g23.fill_wait_ms", self.fill_wait_ms),
            ("g23.notice_wait_ms", self.notice_wait_ms),
        ];
        waits
            .iter()
            .try_for_each(|(key, ms)| wait(key, *ms, most_ms))?;
        let late = KeyboardSettings::from_defaults()
            .map_err(|e| format!("the keyboard's defaults: {e}"))?
            .fill_late_ms;
        let latest = u64::from(*TAB_GAP_MS.end()).saturating_add(late);
        if self.late_ms <= latest {
            return Err(format!(
                "g23.late_ms must be longer than Quick-fill's longest tab_gap_ms plus the keyboard's fill_late_ms, \
                 {latest} ms, or a part sent after a Tab could come after the judging"
            ));
        }
        Ok(())
    }
}

/// The least time G23 holds the mouse: its copy, a Chrome page per fill, the layout switches, each line's follow, keys
/// and pause, every panel click with its pause, every panel wait, each fill's field click, wait and late pause, and
/// the backup offer's waits. The follow takes the keyboard's defaults, as the fresh data folder does. Notices at the
/// start, which a fresh folder does not show, are not counted.
pub fn g23_ms(c: &GatesConfig) -> u64 {
    let (g, k) = (&c.g23, &c.g27);
    let letters = [&g.name, &g.user, &g.password]
        .iter()
        .map(|t| t.chars().count())
        .sum();
    let page_ms = c.apps.get(AppKind::Chrome.name()).map_or(0, |a| a.ready_ms);
    let follow = KeyboardSettings::from_defaults()
        .map_or(0, |s| product::follow_after(&s, c.timing.poll_ms));
    // Each fill takes one click on its row, and one wait for that row to show.
    let clicks = SEED_CLICKS + OPEN_CLOSE + FILLS;
    let panel_waits = clicks + SEED_WAITS + SHUT_WAITS + FILLS;
    total(&[
        c.g8.start_wait_ms,
        g.ready_ms,
        end_ms(c),
        times(FILLS, page_ms),
        times(
            FILLS,
            c.timing.read_wait_ms.saturating_add(c.probes.settle_ms),
        ),
        times(SWITCHES, k.switch_wait_ms),
        times(LINES, total(&[follow, k.switch_wait_ms, g.settle_ms])),
        times(letters, k.key_gap_ms),
        times(clicks, g.settle_ms),
        times(panel_waits, g.panel_wait_ms),
        times(FILLS, g.fill_wait_ms.saturating_add(g.late_ms)),
        times(NOTICE_WAITS, g.notice_wait_ms),
    ])
}

/// Refuses P9 settings that no key could type, that could name no panel part, whose title marks could not be told
/// apart, whose texts kx-ui lacks, or whose run would wait past the mouse guard.
pub fn check(c: &GatesConfig) -> Result<(), String> {
    let g = &c.g23;
    g.check_values(c.g1.max_read_chars)?;
    g.check_ids()?;
    g.check_marks()?;
    g.check_texts()?;
    g.check_waits(c.mouse_guard.limit_ms)?;
    held_fits(c, "g23: G23 on the product", g23_ms(c))
}

#[cfg(test)]
mod tests;
