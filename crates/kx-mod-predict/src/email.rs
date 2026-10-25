//! Emails (spec §6 "Emails"): the emails typed with the keyboard, most recent first, and the chips after one `@`, a
//! typed `@@` or the start of a known email. Which characters an email holds comes from `email.toml`.

use std::fmt;
use std::sync::LazyLock;

use kx_module_api::Redacted;
use kx_module_api::ports::{Chip, ChipKind, ListPage};
use serde::Deserialize;

use crate::casing::Casing;
use crate::fold::fold;
use crate::script;
use crate::split::is_word_char;

mod bytes;

pub use bytes::{EmailFormatError, MAGIC, VERSION};

/// The email marks, compiled in so they are never read from disk.
const TABLE: &str = include_str!("email.toml");

/// The marks, parsed once; a broken table knows no email, and a test keeps it sound.
static MARKS: LazyLock<Option<Marks>> = LazyLock::new(|| toml::from_str(TABLE).ok());

/// The characters an email holds besides word characters.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Marks {
    /// The sign between the local part and the domain.
    at: char,
    /// How many at signs typed in a row list every known email.
    at_run: usize,
    /// The local part's marks.
    local: Vec<char>,
    /// The mark between a domain's labels.
    label: char,
    /// The marks inside a label.
    inner: Vec<char>,
}

/// What the email chips need from the settings; the module fills it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailConfig {
    /// The most emails kept (`[predict] max_emails`).
    pub max_emails: usize,
    /// The domains offered after an `@`, in order (`[predict] domains`).
    pub domains: Vec<String>,
    /// The fewest typed characters before a known email is offered.
    pub min_start: usize,
}

/// The sign between an email's local part and its domain; `None` only for a broken table, which a test rules out.
#[must_use]
pub fn at() -> Option<char> {
    MARKS.as_ref().map(|m| m.at)
}

/// How many at signs typed in a row list every known email; `None` only for a broken table.
#[must_use]
pub fn at_run() -> Option<usize> {
    MARKS.as_ref().map(|m| m.at_run)
}

/// True when `c` may be in an email: a word character or a mark of `email.toml`.
#[must_use]
pub fn is_email_char(c: char) -> bool {
    let mark =
        |m: &Marks| c == m.at || c == m.label || m.local.contains(&c) || m.inner.contains(&c);
    is_word_char(c) || MARKS.as_ref().is_some_and(mark)
}

/// The email-like end of `token`: its last characters that may be in an email.
#[must_use]
pub fn tail(token: &str) -> &str {
    let start = token
        .char_indices()
        .rev()
        .take_while(|&(_, c)| is_email_char(c))
        .last()
        .map_or(token.len(), |(at, _)| at);
    token.get(start..).unwrap_or_default()
}

/// The email in a finished `token`: the run of email characters around its at sign, less the trailing marks no label
/// ends with; `None` when that run does not look like an email.
#[must_use]
pub fn capture(token: &str) -> Option<&str> {
    let sign = token.find(at()?)?;
    let (head, rest) = token.split_at(sign);
    let start = head
        .char_indices()
        .rev()
        .take_while(|&(_, c)| is_email_char(c))
        .last()
        .map_or(sign, |(i, _)| i);
    let end = rest
        .char_indices()
        .find(|&(_, c)| !is_email_char(c))
        .map_or(token.len(), |(i, _)| sign + i);
    let run = token
        .get(start..end)?
        .trim_end_matches(|c| !is_word_char(c));
    looks_like(run).then_some(run)
}

/// True when `text` looks like `local@domain.tld`: one at sign, a local part of word characters and local marks,
/// and a domain as `is_domain` says.
#[must_use]
pub fn looks_like(text: &str) -> bool {
    let Some(m) = MARKS.as_ref() else {
        return false;
    };
    let mut parts = text.split(m.at);
    let (Some(local), Some(domain), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    let local_ok = local
        .chars()
        .all(|c| is_word_char(c) || m.local.contains(&c));
    !local.is_empty() && local_ok && is_domain(domain)
}

/// True when `text` is a domain: two or more labels that start and end with a word character, with only the inner
/// marks between; the `[predict] domains` setting is checked by it.
#[must_use]
pub fn is_domain(text: &str) -> bool {
    let Some(m) = MARKS.as_ref() else {
        return false;
    };
    let label_ok = |label: &str| {
        let ends = [label.chars().next(), label.chars().next_back()];
        ends.iter().all(|c| c.is_some_and(is_word_char))
            && label
                .chars()
                .all(|c| is_word_char(c) || m.inner.contains(&c))
    };
    let labels: Vec<&str> = text.split(m.label).collect();
    labels.len() > 1 && labels.iter().all(|l| label_ok(l))
}

/// The domain chips for `partial`, the text after the one `@`: the domains that start with it once folded, in the
/// settings' order, never the one it already spells, at most `lane`.
#[must_use]
pub fn domains(partial: &str, cfg: &EmailConfig, lane: usize) -> Vec<Chip> {
    completing(&cfg.domains, partial, lane, ChipKind::Domain)
}

/// The emails typed with the keyboard, most recent first.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Emails {
    /// The emails, most recent first.
    list: Vec<String>,
}

impl fmt::Debug for Emails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Emails")
            .field("count", &self.list.len())
            .finish()
    }
}

impl Emails {
    /// The known emails from `list`, most recent first: those that look like emails, once each, at most `max`.
    #[must_use]
    pub fn new(list: &[String], max: usize) -> Self {
        let mut emails = Self::default();
        for email in list.iter().rev() {
            emails.add(email, max);
        }
        emails
    }

    /// The known emails, most recent first.
    #[must_use]
    pub fn list(&self) -> &[String] {
        &self.list
    }

    /// Puts `token` first when it looks like an email, dropping its older copy (any case) and the oldest past `max`;
    /// true when it was taken.
    pub fn add(&mut self, token: &str, max: usize) -> bool {
        if !looks_like(token) {
            return false;
        }
        let lower = Casing::root().lower(token);
        self.list.retain(|e| Casing::root().lower(e) != lower);
        self.list.insert(0, token.to_owned());
        self.list.truncate(max);
        true
    }

    /// The known emails that start with `typed` once folded, at most `lane`: only from `min_start` typed
    /// characters and before any at sign.
    #[must_use]
    pub fn starting(&self, typed: &str, cfg: &EmailConfig, lane: usize) -> Vec<Chip> {
        let has_at = at().is_some_and(|a| typed.contains(a));
        if has_at || typed.chars().count() < cfg.min_start {
            return Vec::new();
        }
        completing(&self.list, typed, lane, ChipKind::Email)
    }

    /// Every known email, most recent first, as chips; past `lane`, the last chip is `More`, which opens the list.
    #[must_use]
    pub fn all(&self, lane: usize) -> Vec<Chip> {
        if self.list.len() <= lane {
            return self.list.iter().map(|e| chip(e, ChipKind::Email)).collect();
        }
        let shown = lane.saturating_sub(1);
        let mut chips: Vec<Chip> = self
            .list
            .iter()
            .take(shown)
            .map(|e| chip(e, ChipKind::Email))
            .collect();
        if lane > 0 {
            chips.push(chip("", ChipKind::More));
        }
        chips
    }

    /// Page `page` of the list, `rows` per page (`[ui] page_rows`); a page past the end shows the last one.
    #[must_use]
    pub fn page(&self, page: u16, rows: usize) -> ListPage {
        let rows = rows.max(1);
        let pages = self.list.len().div_ceil(rows).max(1);
        let at = usize::from(page).min(pages - 1);
        let items = self.list.iter().skip(at * rows).take(rows);
        ListPage {
            items: items.map(|e| Redacted::new(e.clone())).collect(),
            page: u16::try_from(at).unwrap_or(u16::MAX),
            pages: u16::try_from(pages).unwrap_or(u16::MAX),
        }
    }
}

/// The texts of `all` that start with `typed` once folded and are not `typed` itself, at most `lane`, as `kind`.
fn completing(all: &[String], typed: &str, lane: usize, kind: ChipKind) -> Vec<Chip> {
    let key = fold(typed, Casing::root());
    let fits = |text: &&String| {
        let folded = fold(text, Casing::root());
        folded.starts_with(&key) && folded != key
    };
    all.iter()
        .filter(fits)
        .take(lane)
        .map(|t| chip(t, kind))
        .collect()
}

/// A chip of `kind` showing `text`, right to left in a right-to-left script.
fn chip(text: &str, kind: ChipKind) -> Chip {
    Chip {
        text: Redacted::new(text.to_owned()),
        kind,
        rtl: script::of(text).is_some_and(script::Script::rtl),
    }
}

#[cfg(test)]
mod tests;
