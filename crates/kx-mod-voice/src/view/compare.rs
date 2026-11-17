//! The Compare page: the set text with the ready models and where they run, each model's turn while they run, and the
//! results with their day and device. A text shows only for a language with no set text.

use kx_module_api::ports::{CompareRow, CompareRun, CompareSheet, Pages, PanelButton, VoiceSheet};

use super::{Facts, device, short};
use crate::compare::Line;
use crate::flow::Lend;
use crate::panel::{Key, Panel};

impl Panel {
    /// The Compare page as it stands.
    pub(super) fn compare_sheet(&mut self, facts: &Facts<'_>) -> (VoiceSheet, Pages) {
        if let Some((at, models)) = self.compare.running() {
            let name = models
                .get(at.unwrap_or(0))
                .map(|(m, _)| m.name.clone())
                .unwrap_or_default();
            let runs = models.iter().map(|(m, step)| {
                let run = CompareRun {
                    name: m.name.clone(),
                    step: *step,
                };
                (Key::Plain, run)
            });
            let (at, of) = (short(at.unwrap_or(0) + 1), short(models.len()));
            let (rows, pages) = self.paged(runs.collect());
            let sheet = CompareSheet::Running { at, of, name, rows };
            return (VoiceSheet::Compare(sheet), pages);
        }
        if !self.compare.results().is_empty() {
            return self.results_sheet(facts);
        }
        let entrants = self.entrants(facts);
        let lend = self.compare.lend();
        let sheet = CompareSheet::Ready {
            models: short(entrants.len()),
            device: device::label(facts, entrants.first().map(|e| &e.ready)),
            recording: lend == Lend::Listen,
            preparing: lend == Lend::Prep,
        };
        (VoiceSheet::Compare(sheet), self.single())
    }

    /// The results, ticked to use one for the language.
    fn results_sheet(&mut self, facts: &Facts<'_>) -> (VoiceSheet, Pages) {
        let ticks = self.ticks.clone();
        let list: Vec<(Key, CompareRow)> = (self.compare.results().iter())
            .map(|line| {
                let key = Key::Score(line.print);
                let row = compare_row(line, ticks.contains(&key));
                (key, row)
            })
            .collect();
        let (day, auto) =
            (self.compare.plan()).map_or((facts.now.date, false), |p| (p.day, p.auto));
        let device = device::ran_on(facts, auto, self.compare.served());
        let (rows, pages) = self.paged(list);
        let sheet = CompareSheet::Results { day, device, rows };
        (VoiceSheet::Compare(sheet), pages)
    }

    /// The Compare page's buttons that act now.
    pub(crate) fn compare_acts(&self) -> Vec<PanelButton> {
        if self.compare.busy() {
            return vec![PanelButton::Cancel];
        }
        if self.compare.results().is_empty() {
            return Vec::new();
        }
        let ticked = matches!(self.ticked(), Some(Key::Score(_)));
        let mut acts = vec![PanelButton::Rerun];
        acts.extend(ticked.then_some(PanelButton::Use));
        acts
    }
}

/// A result's row; its text, in a copy wiped on drop, only with no set text.
fn compare_row(line: &Line, tick: bool) -> CompareRow {
    CompareRow {
        name: line.name.clone(),
        wrong: line.wrong,
        words: line.words,
        wait_ms: line.wait_ms,
        mb: line.mb,
        text: line.text.clone(),
        tick,
    }
}
