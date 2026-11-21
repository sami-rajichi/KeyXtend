//! Typing on the key page's lines (ADR-0049, edge 22): our keyboard's keys and a click on a line work its text at the
//! caret, and Enter or Save sets it through the page's own acts and checks. A line's text lives only while it shows,
//! and the key typed only here, wiped on Save, Paste, Remove, leaving the page and the panel closing.

use kx_module_api::Redacted;
use kx_module_api::caret;
use kx_module_api::ports::{CloudLine, HttpsUrl, MAX_KEY_BYTES, PanelKey, PanelLine, TAG_LEN};
use kx_module_api::text::SPACE;

use super::{Do, Page, Panel};
use crate::line::Line;
use crate::services::{Address, Service};
use crate::view::Facts;

/// The key page's lines as typed: the line taking the letters and each one's text once edited.
#[derive(Clone, Debug, Default)]
pub(crate) struct Typed {
    /// The line taking the letters; `None` for the page's first.
    at: Option<CloudLine>,
    /// The key typed so far.
    pub(crate) key: Option<Line>,
    /// The address as edited; `None` while the line shows the one kept or refused.
    address: Option<Line>,
    /// The Other model id as edited.
    model: Option<Line>,
}

impl Typed {
    /// `line`'s text as edited, if it was.
    const fn draft(&self, line: CloudLine) -> Option<&Line> {
        match line {
            CloudLine::Key => self.key.as_ref(),
            CloudLine::Address => self.address.as_ref(),
            CloudLine::Model => self.model.as_ref(),
        }
    }

    /// `line`'s text as edited, to edit.
    const fn slot(&mut self, line: CloudLine) -> &mut Option<Line> {
        match line {
            CloudLine::Key => &mut self.key,
            CloudLine::Address => &mut self.address,
            CloudLine::Model => &mut self.model,
        }
    }

    /// The shown line taking the letters, among `lines`.
    pub(crate) fn at(&self, lines: &[CloudLine]) -> Option<CloudLine> {
        let first = lines.first().copied();
        self.at.filter(|l| lines.contains(l)).or(first)
    }

    /// The key typed so far, as its count of dots.
    pub(crate) fn key_dots(&self) -> u32 {
        self.key.as_ref().map_or(0, Line::count)
    }

    /// Puts `text` on `line` as typed, such as a pasted address, and aims the letters at it.
    pub(super) fn put(&mut self, line: CloudLine, text: &str) {
        *self.slot(line) = Some(Line::with(text, line_max(line)));
        self.at = Some(line);
    }
}

impl Panel {
    /// The line the page shows, which the keyboard aims by (ADR-0049): `Private` on a key page, `None` while closed.
    #[must_use]
    pub fn line(&self, facts: &Facts<'_>) -> Option<PanelLine> {
        let page = self.page?;
        let lines = page == Page::Lines && self.shown_row(facts).is_some();
        Some(if lines {
            PanelLine::Private
        } else {
            PanelLine::None
        })
    }

    /// A key from our keyboard while a key page's line is aimed at: Up and Down pick the line, Enter saves it, and the
    /// rest work it at its caret. Off a key page it does nothing.
    pub fn line_key(&mut self, key: &PanelKey, facts: &Facts<'_>) -> Vec<Do> {
        let Some(lines) = self.typed_lines(facts) else {
            return Vec::new();
        };
        let Some(at) = self.cloud.typed.at(lines) else {
            return Vec::new();
        };
        match key {
            PanelKey::Up => self.pick_next(lines, at, false),
            PanelKey::Down => self.pick_next(lines, at, true),
            PanelKey::Enter => return self.save(at, facts),
            // Esc goes to the app, and prediction offers no word to a private line.
            PanelKey::Esc | PanelKey::Word { .. } => {}
            PanelKey::Left
            | PanelKey::Right
            | PanelKey::Tick
            | PanelKey::Erase
            | PanelKey::Delete
            | PanelKey::Text(_) => self.edit(at, key, facts),
        }
        Vec::new()
    }

    /// Works `key` on `line` at its caret: an arrow moves it, and Space, Backspace, Delete and letters change the text.
    fn edit(&mut self, line: CloudLine, key: &PanelKey, facts: &Facts<'_>) {
        let draft = self.draft(line, facts);
        let _ = match key {
            PanelKey::Left => draft.step(false),
            PanelKey::Right => draft.step(true),
            PanelKey::Tick => draft.write(&SPACE.to_string()),
            PanelKey::Erase => draft.erase(),
            PanelKey::Delete => draft.delete(),
            PanelKey::Text(text) => draft.write(text.expose()),
            PanelKey::Up
            | PanelKey::Down
            | PanelKey::Enter
            | PanelKey::Esc
            | PanelKey::Word { .. } => false,
        };
    }

    /// A click on the shown `line`: it takes the letters, with its caret `at` into what it draws.
    pub(super) fn place_caret(&mut self, line: CloudLine, at: u32, facts: &Facts<'_>) {
        if !self.shows_line(line, facts) {
            return;
        }
        self.cloud.typed.at = Some(line);
        if line != CloudLine::Key || self.cloud.typed.key.is_some() {
            self.draft(line, facts).place(at);
        }
    }

    /// Saves `line` through the key page's own acts, as Enter does: an empty key saves nothing, an emptied address or
    /// model clears it. The key typed is wiped; a refused address or model stays on its line to be put right.
    pub(super) fn save(&mut self, line: CloudLine, facts: &Facts<'_>) -> Vec<Do> {
        if !self.shows_line(line, facts) {
            return Vec::new();
        }
        if line == CloudLine::Key {
            let key = self.cloud.typed.key.take().filter(|k| !k.is_empty());
            return key.map_or_else(Vec::new, |k| self.set_key(&k.into_secret()));
        }
        let text = Redacted::new(self.line_text(line, facts));
        let (out, refused) = if line == CloudLine::Address {
            let out = self.type_address(&text, facts);
            (out, self.cloud.refused.is_some())
        } else {
            let out = self.type_model(&text, facts);
            (out, self.cloud.refused_model.is_some())
        };
        if !refused {
            *self.cloud.typed.slot(line) = None;
        }
        out
    }

    /// The key page's lines in page order, while it shows.
    pub(crate) fn typed_lines(&self, facts: &Facts<'_>) -> Option<&'static [CloudLine]> {
        if self.page != Some(Page::Lines) {
            return None;
        }
        self.shown_row(facts).map(page_lines)
    }

    /// What `line` shows: its text as edited, else the one just refused, else the one kept; never the key.
    pub(crate) fn line_text(&self, line: CloudLine, facts: &Facts<'_>) -> String {
        if line == CloudLine::Key {
            return String::new();
        }
        if let Some(draft) = self.cloud.typed.draft(line) {
            return draft.text().to_owned();
        }
        let refused = match line {
            CloudLine::Address => self.cloud.refused.as_ref().map(|(text, _)| text),
            _ => self.cloud.refused_model.as_ref(),
        };
        refused.map_or_else(|| self.kept(line, facts), |t| t.expose().clone())
    }

    /// The address or model `line` keeps in `services`; none for the key, which is never read back.
    fn kept(&self, line: CloudLine, facts: &Facts<'_>) -> String {
        let id = self.cloud.shown.as_ref();
        let entry = id.and_then(|id| facts.settings.services.get(id));
        let kept = entry.map(|e| match line {
            CloudLine::Key => "",
            CloudLine::Address => e.address.as_str(),
            CloudLine::Model => e.model.as_str(),
        });
        kept.unwrap_or_default().to_owned()
    }

    /// `line`'s caret in what it draws: a line not yet edited waits at its text's end, and the key line with nothing
    /// typed after the sealed key's dots, which it draws then.
    pub(crate) fn line_caret(&self, line: CloudLine, facts: &Facts<'_>) -> u32 {
        if line == CloudLine::Key && self.cloud.typed.key_dots() == 0 {
            return self.cloud.dots;
        }
        if let Some(draft) = self.cloud.typed.draft(line) {
            return draft.caret();
        }
        let text = self.line_text(line, facts);
        caret::units(&text, text.len())
    }

    /// True while Save on `line` acts: a key typed, or an address or model holding text or emptied over a kept one.
    pub(crate) fn line_saves(&self, line: CloudLine, facts: &Facts<'_>) -> bool {
        match line {
            CloudLine::Key => self.cloud.typed.key_dots() > 0,
            _ => !self.line_text(line, facts).is_empty() || !self.kept(line, facts).is_empty(),
        }
    }

    /// True while the key page shows `line`.
    pub(super) fn shows_line(&self, line: CloudLine, facts: &Facts<'_>) -> bool {
        self.typed_lines(facts).is_some_and(|l| l.contains(&line))
    }

    /// Picks the line after `at` among `lines` when `on`, else the one before, stopping at the ends.
    fn pick_next(&mut self, lines: &[CloudLine], at: CloudLine, on: bool) {
        let i = lines.iter().position(|&l| l == at).unwrap_or_default();
        let to = if on {
            i.saturating_add(1)
        } else {
            i.saturating_sub(1)
        };
        if let Some(&next) = lines.get(to) {
            self.cloud.typed.at = Some(next);
        }
    }

    /// `line`'s text to edit: the key starts empty and hidden, an address or model from what its line shows.
    fn draft(&mut self, line: CloudLine, facts: &Facts<'_>) -> &mut Line {
        let seed = (line != CloudLine::Key && self.cloud.typed.draft(line).is_none())
            .then(|| self.line_text(line, facts));
        let max = line_max(line);
        self.cloud
            .typed
            .slot(line)
            .get_or_insert_with(|| match seed {
                Some(text) => Line::with(&text, max),
                None => Line::new(max, true),
            })
    }
}

/// The lines `row`'s key page shows, in page order: a typed address's address, model and key (owner, 2026-11-21), else
/// the key and model.
const fn page_lines(row: &Service) -> &'static [CloudLine] {
    match row.address {
        Address::Typed { .. } => &[CloudLine::Address, CloudLine::Model, CloudLine::Key],
        Address::Fixed(_) => &[CloudLine::Key, CloudLine::Model],
    }
}

/// The most `line` holds before Save refuses it, from the frames' own limits: a header's key, an address, a model id.
pub(super) fn line_max(line: CloudLine) -> usize {
    match line {
        CloudLine::Key => MAX_KEY_BYTES,
        CloudLine::Address => HttpsUrl::MAX_BYTES,
        CloudLine::Model => *TAG_LEN.end(),
    }
}
