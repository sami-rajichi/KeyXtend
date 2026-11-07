//! The known emails as the lane offers them (spec §6 "Emails"): the typed ones, most recent first, then the
//! notebook's others in its order. One both typed and saved shows once, as saved, in the notebook's spelling.

use std::collections::{HashMap, HashSet};

use kx_module_api::ports::{Chip, ChipKind, ListPage};
use kx_module_api::text::Casing;

use super::{EmailConfig, Emails, Saved, at, chip, completing};
use crate::menu::pages;

/// The emails one lane offers, borrowed from the typed and saved ones; each is an `Email` or `Saved` chip.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Offer<'a> {
    /// Each email and its kind, in the order offered.
    list: Vec<(&'a str, ChipKind)>,
}

impl<'a> Offer<'a> {
    /// The `typed` emails, each saved when `saved` holds it in any case, then the other `saved` ones.
    #[must_use]
    pub fn new(typed: &'a Emails, saved: Option<&'a Saved>) -> Self {
        let keyed: HashMap<&str, &str> = saved.into_iter().flat_map(Saved::keyed).collect();
        let mut seen = HashSet::new();
        let mut list = Vec::new();
        for email in typed.list() {
            let key = Casing::root().lower(email);
            list.push(match keyed.get(key.as_str()) {
                Some(text) => (*text, ChipKind::Saved),
                None => (email.as_str(), ChipKind::Email),
            });
            seen.insert(key);
        }
        let rest = saved.into_iter().flat_map(Saved::keyed);
        let rest = rest.filter(|(key, _)| !seen.contains(*key));
        list.extend(rest.map(|(_, text)| (text, ChipKind::Saved)));
        Self { list }
    }

    /// What a lane offers: the `typed` emails, and the `saved` ones unless `forgetting`, as those are never forgotten.
    #[must_use]
    pub fn shown(typed: &'a Emails, saved: &'a Saved, forgetting: bool) -> Self {
        Self::new(typed, (!forgetting).then_some(saved))
    }

    /// How many emails it offers.
    #[must_use]
    pub fn len(&self) -> usize {
        self.list.len()
    }

    /// True when it offers none.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// The email at `at` and its kind.
    #[must_use]
    pub fn get(&self, at: usize) -> Option<(&'a str, ChipKind)> {
        self.list.get(at).copied()
    }

    /// The emails `keep` takes, in the same order.
    #[must_use]
    pub fn kept(&self, keep: impl Fn(&str) -> bool) -> Self {
        let list = self.list.iter().filter(|(e, _)| keep(e)).copied().collect();
        Self { list }
    }

    /// The emails that start with `typed` once folded, at most `lane`: only from `min_start` typed characters and
    /// before any at sign.
    #[must_use]
    pub fn starting(&self, typed: &str, cfg: &EmailConfig, lane: usize) -> Vec<Chip> {
        let has_at = at().is_some_and(|a| typed.contains(a));
        if has_at || typed.chars().count() < cfg.min_start {
            return Vec::new();
        }
        completing(self.list.iter().copied(), typed, lane)
    }

    /// Every email in order, as chips; past `lane`, the last chip is `More`, which opens the list.
    #[must_use]
    pub fn all(&self, lane: usize) -> Vec<Chip> {
        if self.list.len() <= lane {
            return self.first(lane);
        }
        let mut chips = self.first(lane.saturating_sub(1));
        if lane > 0 {
            chips.push(chip("", ChipKind::More));
        }
        chips
    }

    /// The first `lane` emails, as chips.
    #[must_use]
    pub fn first(&self, lane: usize) -> Vec<Chip> {
        let shown = self.list.iter().take(lane);
        shown.map(|&(e, kind)| chip(e, kind)).collect()
    }

    /// Page `page` of the list, `rows` per page (`[ui] page_rows`); a page past the end shows the last one.
    #[must_use]
    pub fn page(&self, page: u16, rows: usize) -> ListPage {
        let rows = rows.max(1);
        let pages = pages(self.list.len(), rows);
        let at = usize::from(page).min(pages - 1);
        let items = self.list.iter().skip(at * rows).take(rows);
        ListPage {
            items: items.map(|&(e, kind)| chip(e, kind)).collect(),
            page: u16::try_from(at).unwrap_or(u16::MAX),
            pages: u16::try_from(pages).unwrap_or(u16::MAX),
        }
    }
}

#[cfg(test)]
mod tests;
