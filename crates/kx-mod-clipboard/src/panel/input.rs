//! Keys and clicks: the question takes only its answer, the preview acts on its entry, and the list moves, ticks,
//! filters, searches and acts on the ticked rows (spec §7 "Keys", "Tick, then act").

use kx_module_api::Redacted;
use kx_module_api::ports::{ClipAct, Filter, PanelKey};

use super::{Do, Mode, Panel, unpinned};
use crate::entry::{Entry, flat};
use crate::order::History;

impl Panel {
    /// Takes `key` from our keyboard. The question takes only Enter (yes) and Esc (no), the preview Enter (insert)
    /// and Esc (back), and the list every key.
    pub fn key(&mut self, key: &PanelKey, history: &History) -> Do {
        self.sync(history);
        if self.asking {
            return match key {
                PanelKey::Esc => self.answer(false, history),
                PanelKey::Enter => self.answer(true, history),
                _ => Do::Stay,
            };
        }
        match &self.mode {
            Mode::Closed => Do::Stay,
            Mode::Preview(id, _) => match key {
                PanelKey::Esc => self.back(),
                PanelKey::Enter => Do::Insert(vec![*id]),
                _ => Do::Stay,
            },
            Mode::List => self.list_key(key, history),
        }
    }

    /// Takes `act`, a click on the panel. The question takes only its answer and Close, the preview page its footer,
    /// and a tick from an older view is ignored.
    pub fn act(&mut self, act: ClipAct, history: &History) -> Do {
        if !self.is_open() {
            return Do::Stay;
        }
        self.sync(history);
        let previewing = self.preview().is_some();
        match act {
            ClipAct::Close => self.shut(),
            ClipAct::Confirm(yes) => self.answer(yes, history),
            _ if self.asking => Do::Stay,
            ClipAct::DeleteAll => self.ask(history),
            ClipAct::Insert => acted(self.targets(), Do::Insert),
            ClipAct::Delete => acted(self.targets(), Do::Delete),
            ClipAct::Pin => self.pin(history),
            ClipAct::Back => self.back(),
            _ if previewing => Do::Stay,
            ClipAct::Tick { round, row } => self.tick_row(round, row, history),
            ClipAct::Filter(filter) => self.set_filter(filter),
            ClipAct::Page { back } => self.turn(back, history),
            ClipAct::Preview => self.peek(history),
        }
    }

    /// A key on the list: the arrows move and filter, Space ticks, Enter inserts, Esc backs out, the rest search.
    fn list_key(&mut self, key: &PanelKey, history: &History) -> Do {
        match key {
            PanelKey::Up => self.step(false, history),
            PanelKey::Down => self.step(true, history),
            PanelKey::Left => self.next_filter(false),
            PanelKey::Right => self.next_filter(true),
            PanelKey::Tick => self.tick_hi(history),
            PanelKey::Enter => self.enter(history),
            PanelKey::Esc if self.query.expose().is_empty() => self.shut(),
            PanelKey::Esc => self.search(String::new()),
            PanelKey::Erase => self.erase(),
            PanelKey::Text(text) => self.write(text.expose()),
        }
    }

    /// Closes the panel.
    fn shut(&mut self) -> Do {
        self.close();
        Do::Close
    }

    /// Leaves the preview page.
    fn back(&mut self) -> Do {
        if self.preview().is_none() {
            return Do::Stay;
        }
        self.mode = Mode::List;
        Do::Show
    }

    /// Shows the Delete-all question while an unpinned entry is there to delete.
    fn ask(&mut self, history: &History) -> Do {
        if history.entries().iter().all(Entry::pinned) {
            return Do::Stay;
        }
        self.asking = true;
        Do::Show
    }

    /// Answers the question shown: yes deletes every unpinned entry, whatever the view.
    fn answer(&mut self, yes: bool, history: &History) -> Do {
        if !std::mem::take(&mut self.asking) {
            return Do::Stay;
        }
        let gone = if yes { unpinned(history) } else { Vec::new() };
        if gone.is_empty() {
            Do::Show
        } else {
            Do::Delete(gone)
        }
    }

    /// Pins the targets, or unpins them when every one is pinned.
    fn pin(&self, history: &History) -> Do {
        let targets = self.targets();
        if self.unpins(history) {
            Do::Unpin(targets)
        } else {
            acted(targets, Do::Pin)
        }
    }

    /// Asks for the preview of the last ticked entry, else of the highlighted one.
    fn peek(&mut self, history: &History) -> Do {
        let id = self.ticks.last().copied();
        id.or_else(|| self.highlighted(history))
            .map_or(Do::Stay, Do::Preview)
    }

    /// Enter on the list: inserts the ticked rows in tick order, else the highlighted one.
    fn enter(&mut self, history: &History) -> Do {
        if !self.ticks.is_empty() {
            return Do::Insert(self.ticks.clone());
        }
        let id = self.highlighted(history);
        id.map_or(Do::Stay, |id| Do::Insert(vec![id]))
    }

    /// Ticks or unticks the highlighted row.
    fn tick_hi(&mut self, history: &History) -> Do {
        let Some(id) = self.highlighted(history) else {
            return Do::Stay;
        };
        self.toggle(id);
        Do::Show
    }

    /// Ticks or unticks row `row` of the view of `round` and highlights it; a row of an older view is ignored.
    fn tick_row(&mut self, round: u32, row: u8, history: &History) -> Do {
        let seen = self.shown.get(usize::from(row)).copied();
        let Some(id) = seen.filter(|_| round == self.round) else {
            return Do::Stay;
        };
        let list = self.list(history);
        let Some(at) = list.iter().position(|e| e.id == id) else {
            return Do::Stay;
        };
        self.go(at, &list);
        self.toggle(id);
        Do::Show
    }

    /// Unticks `id`, closing the gap in the tick order, or ticks it last.
    fn toggle(&mut self, id: u64) {
        if let Some(at) = self.ticks.iter().position(|&t| t == id) {
            self.ticks.remove(at);
        } else {
            self.ticks.push(id);
        }
    }

    /// The highlighted entry, if any is listed.
    fn highlighted(&mut self, history: &History) -> Option<u64> {
        let list = self.list(history);
        self.highlight(&list);
        self.hi_id
    }

    /// Moves the highlight a row down or up, stopping at the ends; the page follows it.
    fn step(&mut self, down: bool, history: &History) -> Do {
        let list = self.list(history);
        let Some(hi) = self.highlight(&list) else {
            return Do::Stay;
        };
        let end = list.len().saturating_sub(1);
        let to = if down {
            (hi + 1).min(end)
        } else {
            hi.saturating_sub(1)
        };
        self.jump(hi, to, &list)
    }

    /// Turns to the first row of the page before or after, stopping at the ends.
    fn turn(&mut self, back: bool, history: &History) -> Do {
        let list = self.list(history);
        let Some(hi) = self.highlight(&list) else {
            return Do::Stay;
        };
        let rows = self.rows();
        let (page, end) = (hi / rows, list.len().saturating_sub(1) / rows);
        let to = if back {
            page.saturating_sub(1)
        } else {
            (page + 1).min(end)
        };
        if to == page {
            return Do::Stay;
        }
        self.jump(hi, to * rows, &list)
    }

    /// Highlights place `to` of `list` when it is not `from`.
    fn jump(&mut self, from: usize, to: usize, list: &[&Entry]) -> Do {
        if from == to {
            return Do::Stay;
        }
        self.go(to, list);
        Do::Show
    }

    /// Steps to the filter after or before, stopping at the ends.
    fn next_filter(&mut self, on: bool) -> Do {
        let at = Filter::ORDER.iter().position(|&f| f == self.filter);
        let to = at.and_then(|i| {
            if on {
                i.checked_add(1)
            } else {
                i.checked_sub(1)
            }
        });
        let next = to.and_then(|i| Filter::ORDER.get(i)).copied();
        next.map_or(Do::Stay, |f| self.set_filter(f))
    }

    /// Shows `filter`, highlighting the top row.
    fn set_filter(&mut self, filter: Filter) -> Do {
        if filter == self.filter {
            return Do::Stay;
        }
        self.filter = filter;
        self.top();
        Do::Show
    }

    /// Adds `text` to the search on one line, up to `query_chars` characters.
    fn write(&mut self, text: &str) -> Do {
        let mut query = self.query.expose().clone();
        let room = self.cfg.query_chars.saturating_sub(query.chars().count());
        query.extend(flat(text).take(room));
        self.search(query)
    }

    /// Erases the search's last character.
    fn erase(&mut self) -> Do {
        let mut query = self.query.expose().clone();
        query.pop();
        self.search(query)
    }

    /// Sets the search to `query`, highlighting the top row; nothing when it is the same.
    fn search(&mut self, query: String) -> Do {
        if query == *self.query.expose() {
            return Do::Stay;
        }
        self.query = Redacted::new(query);
        self.top();
        Do::Show
    }
}

/// `make` of `ids`, or nothing when there is none.
fn acted(ids: Vec<u64>, make: fn(Vec<u64>) -> Do) -> Do {
    if ids.is_empty() { Do::Stay } else { make(ids) }
}
