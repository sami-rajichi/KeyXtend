//! `kx-gates-p10.toml`: G19 on the product (`[g19_product]`), the selection strip checked on the test build. Its
//! struct lives in `p10cfg/fields.rs` and its key and wait checks in `p10cfg/limits.rs`; this file checks the rest
//! and counts the least time the run holds the mouse.
#![cfg(windows)]

mod fields;
mod limits;

pub use fields::{Bar, G19Product};

use std::ops::RangeInclusive;
use std::time::Duration;

use kx_mod_selection::{SelSettings, sites};
use kx_module_api::ports::SelChip;

use crate::apps::AppKind;
use crate::config::GatesConfig;
use crate::fit::{held_fits, times, total};
use crate::g19chips::text_of;
use crate::p5cfg::SEP;
use crate::p7cfg::runs::end_ms;
use crate::{ftl, g19chips};

/// The cases that must show no strip for `quiet_ms`.
pub const QUIET: usize = g19chips::QUIET;
/// The gate's clicks, keys and drags, each followed by `settle_ms`.
pub const ACTS: usize = g19chips::ACTS;
/// The target-windows the run starts: the hung one, and the password box the field watch hears.
const TARGETS: usize = 2;
/// What a page text, name or mark may not hold: quotes and markup, which would end the page's tags or script.
const NOT_IN_PAGE: &str = "'\"\\<>&";
/// What a test line may not hold besides that: a line break, which would split it.
const BREAKS: [char; 2] = ['\n', '\r'];
/// Unicode's Arabic block, which every letter of the Arabic line comes from.
pub const ARABIC: RangeInclusive<char> = '\u{0600}'..='\u{06FF}';

/// The selection module's defaults, as a fresh data folder holds them.
pub fn selection_defaults() -> Result<SelSettings, String> {
    SelSettings::from_defaults().map_err(|e| format!("the selection defaults: {e}"))
}

/// The module's `caret_s`, in ms.
pub fn caret_ms(s: &SelSettings) -> u64 {
    u64::try_from(Duration::from_secs(u64::from(s.caret_s)).as_millis()).unwrap_or(u64::MAX)
}

/// The ms one window takes to be brought forward when it is not yet.
pub fn front_ms(c: &GatesConfig) -> u64 {
    c.timing
        .alt_settle_ms
        .saturating_add(c.timing.front_wait_ms)
}

/// The ms one click takes: the pointer's step onto its point, then the button held.
pub fn press_ms(c: &GatesConfig) -> u64 {
    c.sim.step_ms.saturating_add(c.sim.click_ms)
}

/// The least time `g19 keyxtend` holds the mouse: its copy, its Notepad, Chrome and two target-windows, every quiet
/// wait, the hang, the ping, the caret's Paste timing out, and the settle after each act.
pub fn g19_ms(c: &GatesConfig) -> u64 {
    let g = &c.g19_product;
    let ready = |app: AppKind| c.apps.get(app.name()).map_or(0, |a| a.ready_ms);
    let caret = selection_defaults().map_or(0, |s| caret_ms(&s));
    total(&[
        c.g8.start_wait_ms,
        g.ready_ms,
        end_ms(c),
        ready(AppKind::Notepad),
        ready(AppKind::Chrome),
        times(TARGETS, ready(AppKind::Target)),
        times(QUIET, g.quiet_ms),
        g.hang_ms,
        g.ping_ms,
        caret,
        times(ACTS, g.settle_ms),
    ])
}

/// Refuses P10 settings the gate could not tell apart, kx-ui lacks, or that would wait past the mouse guard.
pub fn check(c: &GatesConfig) -> Result<(), String> {
    let g = &c.g19_product;
    g.check_lines()?;
    g.check_page()?;
    g.check_strip()?;
    g.check_keys()?;
    g.check_ping()?;
    g.check_bars()?;
    g.check_waits(c.mouse_guard.limit_ms, (front_ms(c), press_ms(c)))?;
    held_fits(c, "g19_product: G19 on the product", g19_ms(c))
}

/// True when `s` holds a quote or markup.
fn marked(s: &str) -> bool {
    s.chars().any(|c| NOT_IN_PAGE.contains(c))
}

impl G19Product {
    /// The long Translate line: `arabic` `arabic_repeat` times, joined by a space.
    pub fn long_line(&self) -> String {
        vec![self.arabic.as_str(); self.arabic_repeat].join(" ")
    }

    /// The long line's length in characters, counted without building it, so a huge repeat is refused first.
    pub fn long_chars(&self) -> usize {
        let (one, n) = (self.arabic.chars().count(), self.arabic_repeat);
        one.saturating_mul(n).saturating_add(n.saturating_sub(1))
    }

    /// `chip`'s own id, as the lists of each box kind name it.
    pub fn chip_name(&self, chip: SelChip) -> Option<&str> {
        let at = SelChip::ALL.iter().position(|c| *c == chip)?;
        self.chips.get(at).map(String::as_str)
    }

    /// The UI Automation id of `chip`'s keycap, which the tests pin to kx-ui's.
    #[cfg(test)]
    pub fn chip_id(&self, chip: SelChip) -> Option<String> {
        Some(format!("{}{SEP}{}", self.group, self.chip_name(chip)?))
    }

    /// The chip whose keycap has UI Automation id `id`.
    pub fn chip_of(&self, id: &str) -> Option<SelChip> {
        let own = id.strip_prefix(&self.group)?.strip_prefix(SEP)?;
        let at = self.chips.iter().position(|c| c == own)?;
        SelChip::ALL.get(at).copied()
    }

    /// Refuses an empty, shared or broken test line, and an Arabic line that is not Arabic or not long enough.
    fn check_lines(&self) -> Result<(), String> {
        let lines = [
            ("copy_line", &self.copy_line),
            ("cut_line", &self.cut_line),
            ("paste_line", &self.paste_line),
            ("term_line", &self.term_line),
            ("box_text", &self.box_text),
            ("page_text", &self.page_text),
        ];
        for (i, (key, line)) in lines.iter().enumerate() {
            let shared = lines[..i].iter().any(|(_, l)| l == line);
            if line.is_empty() || line.contains(BREAKS) || marked(line) || shared {
                return Err(format!(
                    "g19_product.{key} must be one line of its own, with no quote or markup"
                ));
            }
        }
        let arabic = |c: char| ARABIC.contains(&c);
        let plain = |c: char| arabic(c) || c == ' ' || c.is_ascii_punctuation();
        let a = &self.arabic;
        if !a.chars().any(arabic) || !a.chars().all(plain) || marked(a) {
            return Err("g19_product.arabic must be Arabic text, with no quote or markup".into());
        }
        self.check_long()
    }

    /// Refuses a long line shorter than `long_min_chars`, past the module's `translate_chars`, or cut by the site.
    fn check_long(&self) -> Result<(), String> {
        let s = selection_defaults()?;
        let (chars, most) = (self.long_chars(), usize::from(s.translate_chars));
        if self.long_min_chars == 0 || self.long_min_chars > most {
            return Err(format!("g19_product.long_min_chars must be 1 to {most}"));
        }
        let wrong = || {
            let min = self.long_min_chars;
            format!(
                "g19_product.arabic_repeat makes a line of {chars} characters: it must be {min} to {most}, and the \
                 site's link must take it whole"
            )
        };
        if !(self.long_min_chars..=most).contains(&chars) {
            return Err(wrong());
        }
        let long = self.long_line();
        let site = sites().ok().and_then(|t| t.get(&s.translate_site));
        let site = site.ok_or("the selection module's translate_site names no site")?;
        let key = g19chips::site_key(&s).ok_or("the site's link names no key for its text")?;
        let sent = site.link(&long, &site.target(&s.translate_to, ""), s.translate_chars);
        let whole = sent.and_then(|l| text_of(l.expose(), &key));
        if whole.as_deref() != Some(&long) {
            return Err(wrong());
        }
        Ok(())
    }

    /// Refuses page names and marks the gate could not tell apart, and a host or code with more than a name.
    fn check_page(&self) -> Result<(), String> {
        let names = [
            ("box_name", &self.box_name),
            ("pass_name", &self.pass_name),
            ("unknown_name", &self.unknown_name),
            ("unknown_role", &self.unknown_role),
            ("pass_value", &self.pass_value),
            ("css", &self.css),
        ];
        for (i, (key, name)) in names.iter().enumerate() {
            let shared = names[..i].iter().any(|(_, n)| n == name);
            if name.is_empty() || marked(name) || shared {
                return Err(format!(
                    "g19_product.{key} must be a name of its own, with no quote or markup"
                ));
            }
        }
        let (copy, shift) = (&self.copy_mark, &self.shift_mark);
        for (key, m) in [("copy_mark", copy), ("shift_mark", shift)] {
            if m.is_empty() || marked(m) || m.chars().any(|c| c.is_ascii_digit()) {
                return Err(format!(
                    "g19_product.{key} must be a mark with no digit, quote or markup"
                ));
            }
        }
        if copy.contains(shift.as_str()) || shift.contains(copy.as_str()) {
            return Err(
                "g19_product.shift_mark must hold no part of copy_mark, nor copy_mark of it".into(),
            );
        }
        if self.copy_code.is_empty() || !self.copy_code.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err("g19_product.copy_code must be a DOM code: letters and digits only".into());
        }
        let host = |c: char| c.is_ascii_alphanumeric() || ".-:".contains(c);
        if self.ping_host.is_empty() || !self.ping_host.chars().all(host) {
            return Err("g19_product.ping_host must be a plain host name or address".into());
        }
        Ok(())
    }

    /// Refuses a title, group or chip list kx-ui does not have, and a box's chips out of the strip's order.
    fn check_strip(&self) -> Result<(), String> {
        let named = |e: String| format!("g19_product.strip_title: {e}");
        ftl::every(&self.strip_title).map_err(named)?;
        if self.group.is_empty() || self.group.contains(SEP) {
            return Err(format!("g19_product.group must be a name without {SEP}"));
        }
        let (msg, _) = self.strip_title.split_once(SEP).unwrap_or_default();
        let texted = |id: &String| ftl::every(&format!("{msg}{SEP}{id}")).is_ok();
        let twice = |i: usize| self.chips[..i].contains(&self.chips[i]);
        let bad = (0..self.chips.len()).any(twice) || !self.chips.iter().all(texted);
        if self.chips.len() != SelChip::ALL.len() || bad {
            return Err(format!(
                "g19_product.chips must name kx-ui's {} keycaps once each, in the strip's order",
                SelChip::ALL.len()
            ));
        }
        let lists = [
            ("editable_chips", &self.editable_chips),
            ("page_chips", &self.page_chips),
            ("terminal_chips", &self.terminal_chips),
            ("caret_chips", &self.caret_chips),
        ];
        for (key, list) in lists {
            let at: Vec<Option<usize>> = list
                .iter()
                .map(|id| self.chips.iter().position(|c| c == id))
                .collect();
            let ordered = at.windows(2).all(|w| w[0] < w[1]);
            if list.is_empty() || at.contains(&None) || !ordered {
                return Err(format!(
                    "g19_product.{key} must name some of g19_product.chips, in their order"
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
