//! The rows of the shown page: turning pages, ticking (one row, or History's in order), opening a folder, an option's
//! switch and stepper, and the answer to the question shown.

use kx_module_api::ports::BarHint;

use super::{Do, Key, Panel, options};
use crate::VoiceSettings;
use crate::folder::offer;
use crate::view::Facts;

impl Panel {
    /// Turns the page; the view keeps it within the list.
    pub(super) fn turn(&mut self, back: bool) {
        self.at = if back {
            self.at.saturating_sub(1)
        } else {
            self.at.saturating_add(1)
        };
    }

    /// A row of the shown page: open a folder in the folder list, else tick or untick it.
    pub(super) fn row(&mut self, at: usize) -> Vec<Do> {
        let Some(key) = self.shown.get(at).cloned() else {
            return Vec::new();
        };
        match key {
            Key::Dir(dir) => return self.open_dir(dir),
            Key::Opt(_) | Key::Plain => {}
            Key::Text(_) => self.tick_many(key),
            Key::Model(_) | Key::Device(_) | Key::Score(_) => self.tick_one(key),
        }
        Vec::new()
    }

    /// Ticks `key` alone, or unticks it.
    fn tick_one(&mut self, key: Key) {
        self.ticks = if self.ticks == [key.clone()] {
            Vec::new()
        } else {
            vec![key]
        };
    }

    /// Ticks `key` after the others, or unticks it.
    fn tick_many(&mut self, key: Key) {
        match self.ticks.iter().position(|k| *k == key) {
            Some(at) => drop(self.ticks.remove(at)),
            None => self.ticks.push(key),
        }
    }

    /// The ticked row, on pages that tick one.
    pub(crate) fn ticked(&self) -> Option<&Key> {
        self.ticks.first()
    }

    /// An option's switch on the shown page.
    pub(super) fn flip(&self, at: usize, settings: &VoiceSettings) -> Vec<Do> {
        let opt = self.opt(at);
        let value = opt.and_then(|o| o.flipped(&options::values(settings)));
        opt.zip(value)
            .map(|(o, v)| Do::Set(o.key.as_str(), v))
            .into_iter()
            .collect()
    }

    /// An option's minus or plus on the shown page, where its row is live.
    pub(super) fn step(&self, at: usize, up: bool, settings: &VoiceSettings) -> Vec<Do> {
        let table = options::values(settings);
        let opt = self
            .opt(at)
            .filter(|o| o.row(&table, None).is_some_and(|r| r.live));
        let value = opt.and_then(|o| o.moved(&table, up));
        opt.zip(value)
            .map(|(o, v)| Do::Set(o.key.as_str(), v))
            .into_iter()
            .collect()
    }

    /// The option of the shown page's row `at`.
    fn opt(&self, at: usize) -> Option<&'static options::Opt> {
        match self.shown.get(at) {
            Some(Key::Opt(i)) => options::rows().get(*i),
            _ => None,
        }
    }

    /// An answer to the question shown: Delete all, or the folder offered at the first download.
    pub(super) fn confirm(&mut self, yes: bool, facts: &Facts<'_>) -> Vec<Do> {
        if let Some(asked) = self.asked.take() {
            return if yes {
                vec![Do::DeleteAll(asked)]
            } else {
                Vec::new()
            };
        }
        if self.offer.is_none() || !yes {
            self.offer = None;
            return Vec::new();
        }
        if let Some((dir, _)) = offer(facts.drives) {
            self.choose(dir)
        } else {
            self.note = Some(BarHint::Folder);
            Vec::new()
        }
    }
}
