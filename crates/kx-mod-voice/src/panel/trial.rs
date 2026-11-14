//! Compare's acts on the panel: Start with the set text the ui shows, Cancel, Compare again, and Use for this language
//! on a ticked result; and the notes of Compare's run.

use kx_module_api::Redacted;
use kx_module_api::ports::{DevicePick, PanelButton, SpeechNote, VoiceTab};

use super::{Choice, Do, Key, Page, Panel, wrap};
use crate::compare::{Entrant, Plan};
use crate::folder::Print;
use crate::models::table;
use crate::view::Facts;

impl Panel {
    /// Start: the set text is read aloud once, then each ready model writes it; nothing while a dictation runs.
    pub(super) fn start(&mut self, set: &Redacted<String>, facts: &Facts<'_>) -> Vec<Do> {
        if self.page != Some(Page::Tab(VoiceTab::Compare)) || facts.dictating {
            return Vec::new();
        }
        let plan = Plan {
            lang: facts.iso().to_owned(),
            set: set.clone(),
            entrants: self.entrants(facts),
            day: facts.now.date,
            auto: facts.settings.device() == DevicePick::Auto,
        };
        self.ticks.clear();
        wrap(self.compare.start(plan, facts.settings))
    }

    /// The ready models that know the language in front, in its order, then the other files that do, each as the
    /// language would get it.
    pub(crate) fn entrants(&self, facts: &Facts<'_>) -> Vec<Entrant> {
        let (Ok(table), iso) = (table(), facts.iso()) else {
            return Vec::new();
        };
        let stock = self.stock(facts);
        let rows = (table.order(iso).rows.into_iter()).filter_map(|r| {
            Some(Entrant {
                name: Redacted::new(r.name.clone()),
                ready: stock.row(r, iso).ok()?,
            })
        });
        let others = (self.folder.others()).filter_map(|f| {
            Some(Entrant {
                name: Redacted::new(f.name.clone()),
                ready: stock.file(f, iso).ok()?,
            })
        });
        rows.chain(others).collect()
    }

    /// A Compare button.
    pub(super) fn compare_button(&mut self, button: PanelButton, facts: &Facts<'_>) -> Vec<Do> {
        match button {
            PanelButton::Cancel => wrap(self.compare.cancel()),
            PanelButton::Rerun => {
                self.compare.reset();
                self.ticks.clear();
                Vec::new()
            }
            PanelButton::Use => {
                let choice = match self.ticked() {
                    Some(Key::Score(print)) => self.choice_of(print),
                    _ => None,
                };
                choice.map_or_else(Vec::new, |c| self.pick(&c, facts))
            }
            _ => Vec::new(),
        }
    }

    /// A note from Compare's run `take`.
    pub fn compare_note(&mut self, take: u32, note: SpeechNote) -> Vec<Do> {
        wrap(self.compare.note(take, note))
    }

    /// The model a result's fingerprint names: its row, else the folder's file.
    fn choice_of(&self, print: &kx_module_api::ports::Fingerprint) -> Option<Choice> {
        if let Some(row) = table().ok().and_then(|t| t.printed(print)) {
            return Some(Choice::Row(row.id.clone()));
        }
        let file = self
            .folder
            .others()
            .find(|f| f.print == Print::Known(*print));
        file.map(|f| Choice::File(f.name.clone()))
    }
}
