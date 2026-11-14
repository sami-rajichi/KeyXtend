//! The Models page: the language's models in its order, then the other files that may know it, each with its size,
//! marks and download; the best model's note, the folder line and the first download's folder question.

use kx_module_api::Redacted;
use kx_module_api::ports::{
    DeviceKind, ModelRow, ModelState, ModelsSheet, Pages, PanelButton, VoiceSheet,
};

use super::Facts;
use crate::folder::{File, Print, offer};
use crate::models::{Pick, Ranked, Row, code_in, table};
use crate::panel::{Choice, Key, Panel};

impl Panel {
    /// The Models page's shown rows.
    pub(super) fn models_sheet(&mut self, facts: &Facts<'_>) -> (VoiceSheet, Pages) {
        let ticks = self.ticks.clone();
        let list = self.model_list(facts).into_iter().map(|(choice, mut row)| {
            let key = Key::Model(choice);
            row.tick = ticks.contains(&key);
            (key, row)
        });
        let (rows, pages) = self.paged(list.collect());
        let order = table().ok().map(|t| t.order(facts.iso()));
        let best = order.as_ref().and_then(Ranked::first);
        let sheet = ModelsSheet {
            rows,
            best: best.map(|r| r.name.clone()),
            untested: order.is_some_and(|o| o.untested),
            folder: self.folder.line(facts.drives),
            offer: self
                .offer
                .as_ref()
                .and_then(|_| offer(facts.drives))
                .map(|(_, l)| l),
        };
        (VoiceSheet::Models(sheet), pages)
    }

    /// The Models page's buttons that act now.
    pub(crate) fn models_acts(&self, facts: &Facts<'_>) -> Vec<PanelButton> {
        let mut acts = vec![PanelButton::Change];
        acts.extend(self.folder.dir().map(|_| PanelButton::Open));
        let Some(Key::Model(choice)) = self.ticked() else {
            return acts;
        };
        let state = self.state_of(choice, facts);
        let fetching = matches!(choice, Choice::Row(id) if self.fetches.state(id).is_some());
        let used = self.used_choice(facts).as_ref() == Some(choice);
        let usable = state == Some(ModelState::Ready) || self.other_knows(choice, facts);
        let waits = self.fetches.busy() || self.folder.reading() || self.offer.is_some();
        let taken = self.taken(choice).is_some();
        // The model in use keeps its file, as in the mock-up; a file in the way of its name can still go.
        let own = matches!(state, Some(ModelState::Ready | ModelState::Other)) && !used;
        let missing = state == Some(ModelState::Missing) && !taken;
        acts.extend((usable && !used).then_some(PanelButton::Use));
        acts.extend((missing && !waits).then_some(PanelButton::Download));
        acts.extend(((taken || own) && !fetching).then_some(PanelButton::Delete));
        acts
    }

    /// Every model row for the language in front, in order, unticked.
    pub(crate) fn model_list(&self, facts: &Facts<'_>) -> Vec<(Choice, ModelRow)> {
        let Ok(table) = table() else {
            return Vec::new();
        };
        let order = table.order(facts.iso());
        let used = self.used_choice(facts);
        let default = order.first().map(|r| r.id.as_str());
        let row = |r: &Row| {
            let choice = Choice::Row(r.id.clone());
            let model = ModelRow {
                name: Redacted::new(r.name.clone()),
                bytes: r.bytes,
                default: default == Some(r.id.as_str()),
                state: self.state_of(&choice, facts).unwrap_or(ModelState::Missing),
                used: used.as_ref() == Some(&choice),
                tick: false,
                fetch: self.fetches.state(&r.id),
            };
            (choice, model)
        };
        let mut list: Vec<(Choice, ModelRow)> = order.rows.iter().map(|r| row(r)).collect();
        let others = self.folder.others().filter(|f| self.may_know(f, facts));
        list.extend(others.map(|f| other_row(f, used.as_ref())));
        list
    }

    /// Where `choice`'s file stands; `None` for an other file no longer there.
    pub(crate) fn state_of(&self, choice: &Choice, facts: &Facts<'_>) -> Option<ModelState> {
        let row = match choice {
            Choice::Row(id) => table().ok()?.get(id)?,
            Choice::File(name) => return self.folder.file(name).map(|_| ModelState::Other),
        };
        let cpu = facts.devices.iter().find(|d| d.kind == DeviceKind::Cpu);
        if let Some(not_here) = cpu.and_then(|d| row.not_here(&d.memory, facts.settings.job_mb)) {
            return Some(not_here);
        }
        Some(
            self.folder
                .of(row)
                .map_or(ModelState::Missing, |_| ModelState::Ready),
        )
    }

    /// The model used for the language in front, as the flow's `Supply` finds it.
    pub(crate) fn used_choice(&self, facts: &Facts<'_>) -> Option<Choice> {
        let table = table().ok()?;
        let default = || {
            table
                .default_for(facts.iso())
                .map(|r| Choice::Row(r.id.clone()))
        };
        match table.used(facts.iso(), &facts.settings.picks)? {
            Pick::Row(row) => Some(Choice::Row(row.id.clone())),
            Pick::File(name) => match self.folder.file(name).map(|f| f.print) {
                Some(Print::Known(p)) => match table.printed(&p) {
                    Some(row) => Some(Choice::Row(row.id.clone())),
                    None => Some(Choice::File(name.to_owned())),
                },
                Some(_) => Some(Choice::File(name.to_owned())),
                None => default(),
            },
        }
    }

    /// True for an other file whose facts name the language in front, so Use may pick it.
    fn other_knows(&self, choice: &Choice, facts: &Facts<'_>) -> bool {
        let Choice::File(name) = choice else {
            return false;
        };
        let print = self.folder.file(name).map(|f| f.print);
        let facts_of = |p| self.folder.facts(&p);
        let known = match print {
            Some(Print::Known(p)) => facts_of(p),
            _ => None,
        };
        known.is_some_and(|f| code_in(f, facts.iso()).is_some())
    }

    /// True for an other file whose facts are unread or name the language in front.
    fn may_know(&self, file: &File, facts: &Facts<'_>) -> bool {
        let Print::Known(print) = file.print else {
            return false;
        };
        let wanted = self.folder.wants_facts(&print);
        let named = self
            .folder
            .facts(&print)
            .is_some_and(|f| code_in(f, facts.iso()).is_some());
        wanted || named
    }
}

/// An other file's row: Other file, not checked.
fn other_row(file: &File, used: Option<&Choice>) -> (Choice, ModelRow) {
    let choice = Choice::File(file.name.clone());
    let model = ModelRow {
        name: Redacted::new(file.name.clone()),
        bytes: file.bytes,
        default: false,
        state: ModelState::Other,
        used: used == Some(&choice),
        tick: false,
        fetch: None,
    };
    (choice, model)
}
