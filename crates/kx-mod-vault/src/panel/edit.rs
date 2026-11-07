//! New and Edit (design "New and Edit"): the kind chips, then the name and the fields, each typed into a wiped line that
//! holds its room from the start, so a growing value leaves no unwiped copy; the labels table to add a field, Make
//! one, the hidden flag, and Delete entry's question.

use std::collections::BTreeSet;

use kx_module_api::Redacted;
use kx_module_api::ports::{EntryKind, Label};

use super::copy;
use crate::entry::{Draft, Entry, Field, Line, text};
use crate::kinds::tables;

/// Which line takes the letters.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Taking {
    /// The entry's name.
    Name,
    /// The highlighted field's own label.
    Label,
    /// The highlighted field's value.
    Value,
}

/// The part of the page that shows.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Sub {
    /// The name and the fields.
    Form,
    /// The labels table to add a field, with its highlighted label.
    Fields(usize),
    /// Delete entry's question.
    Asking,
}

/// A field's label as it is typed.
#[derive(Debug)]
pub(crate) enum Tag {
    /// A table label by its id.
    Table(String),
    /// The user's own label.
    Own(Line),
}

/// A field as it is typed.
#[derive(Debug)]
pub(crate) struct Slot {
    pub(crate) tag: Tag,
    pub(crate) value: Line,
    pub(crate) hidden: bool,
}

/// The New or Edit page.
#[derive(Debug)]
pub(crate) struct Editor {
    /// The entry edited, or `None` on New.
    pub(crate) id: Option<u64>,
    /// The kind, once picked.
    pub(crate) kind: Option<EntryKind>,
    pub(crate) name: Line,
    pub(crate) slots: Vec<Slot>,
    pub(crate) taking: Taking,
    /// The highlighted field, which the page follows.
    pub(crate) at: usize,
    /// The hidden fields shown, until the page closes.
    pub(crate) shown: BTreeSet<usize>,
    pub(crate) sub: Sub,
    /// The field Paste was asked on, until its answer comes.
    pub(crate) pasting: Option<usize>,
    /// The most characters a line holds.
    pub(crate) max: usize,
}

impl Editor {
    /// A New page, showing the kind chips.
    pub(crate) fn new(max: usize) -> Self {
        Self {
            id: None,
            kind: None,
            name: Line::new(max),
            slots: Vec::new(),
            taking: Taking::Name,
            at: 0,
            shown: BTreeSet::new(),
            sub: Sub::Form,
            pasting: None,
            max,
        }
    }

    /// The Edit page of `entry`; a saved text longer than `max`, from a lowered setting, gets room for itself.
    pub(crate) fn of(entry: &Entry, max: usize) -> Option<Self> {
        let line = |t: &str| Line::with(t, max.max(t.chars().count())).ok();
        let slot = |f: &Field| {
            let tag = match &f.label {
                Label::Table(id) => Tag::Table(id.clone()),
                Label::Own(own) => Tag::Own(line(own.expose())?),
            };
            let value = line(f.value.expose())?;
            Some(Slot {
                tag,
                value,
                hidden: f.hidden,
            })
        };
        let slots = entry.fields().iter().map(slot).collect::<Option<_>>()?;
        Some(Self {
            id: Some(entry.id()),
            kind: Some(entry.kind()),
            name: line(entry.name().expose())?,
            slots,
            ..Self::new(max)
        })
    }

    /// Picks `kind` on New, with its starter fields; false once a kind is picked.
    pub(crate) fn pick(&mut self, kind: EntryKind) -> bool {
        if self.kind.is_some() {
            return false;
        }
        self.kind = Some(kind);
        let draft = Draft::new(kind);
        let slot = |f: Field| match f.label {
            Label::Table(id) => Some(Slot {
                tag: Tag::Table(id),
                value: Line::new(self.max),
                hidden: f.hidden,
            }),
            Label::Own(_) => None,
        };
        self.slots = draft.fields.into_iter().filter_map(slot).collect();
        true
    }

    /// The highlight a page turn or a list key moves: the field, or the labels table's row.
    pub(crate) fn cursor(&mut self) -> &mut usize {
        match &mut self.sub {
            Sub::Fields(at) => at,
            Sub::Form | Sub::Asking => &mut self.at,
        }
    }

    /// Moves the typing to the line below or above: the name, then each field's own label and value.
    pub(crate) fn step(&mut self, down: bool) -> bool {
        if self.kind.is_none() {
            return false;
        }
        let lines = self.lines();
        let here = (
            self.taking,
            if self.taking == Taking::Name {
                0
            } else {
                self.at
            },
        );
        let at = lines.iter().position(|l| *l == here).unwrap_or_default();
        let to = if down {
            at.checked_add(1)
        } else {
            at.checked_sub(1)
        };
        let Some(&(taking, slot)) = to.and_then(|t| lines.get(t)) else {
            return false;
        };
        self.taking = taking;
        if taking != Taking::Name {
            self.at = slot;
        }
        true
    }

    /// Every line in order, by what takes the letters and field.
    fn lines(&self) -> Vec<(Taking, usize)> {
        let mut all = vec![(Taking::Name, 0)];
        for (i, slot) in self.slots.iter().enumerate() {
            if matches!(slot.tag, Tag::Own(_)) {
                all.push((Taking::Label, i));
            }
            all.push((Taking::Value, i));
        }
        all
    }

    /// Highlights field `i`, its own label when `label` and it has one, else its value; false when there is none.
    pub(crate) fn focus(&mut self, i: usize, label: bool) -> bool {
        let Some(slot) = self.slots.get(i) else {
            return false;
        };
        let own = matches!(slot.tag, Tag::Own(_));
        if label && !own {
            return false;
        }
        (self.at, self.taking) = (i, if label { Taking::Label } else { Taking::Value });
        true
    }

    /// Adds a field under `tag`, hidden as its label starts, typing its first line; false at `most` fields.
    pub(crate) fn add(&mut self, tag: Tag, most: usize) -> bool {
        if self.slots.len() >= most {
            return false;
        }
        let (hidden, taking) = match &tag {
            Tag::Table(id) => (tables().label(id).is_some_and(|l| l.hidden), Taking::Value),
            Tag::Own(_) => (false, Taking::Label),
        };
        let value = Line::new(self.max);
        self.slots.push(Slot { tag, value, hidden });
        (self.at, self.taking, self.sub) = (self.slots.len() - 1, taking, Sub::Form);
        true
    }

    /// A new own label's line.
    pub(crate) fn own(&self) -> Tag {
        Tag::Own(Line::new(self.max))
    }

    /// Switches field `i`'s hidden flag; a value switched to hidden is masked at once, and its caret keeps to an edge.
    pub(crate) fn switch(&mut self, i: usize) -> bool {
        let Some(slot) = self.slots.get_mut(i) else {
            return false;
        };
        slot.hidden = !slot.hidden;
        slot.value.to_edge();
        self.shown.remove(&i);
        true
    }

    /// Shows hidden field `i`, or hides it again, its caret keeping to an edge; false when it is not hidden.
    pub(crate) fn show(&mut self, i: usize) -> bool {
        let Some(slot) = self.slots.get_mut(i).filter(|s| s.hidden) else {
            return false;
        };
        slot.value.to_edge();
        if !self.shown.remove(&i) {
            self.shown.insert(i);
        }
        true
    }

    /// True when field `i`'s label offers Make one.
    pub(crate) fn makes(&self, i: usize) -> bool {
        let label = self.slots.get(i).and_then(|s| match &s.tag {
            Tag::Table(id) => tables().label(id),
            Tag::Own(_) => None,
        });
        label.is_some_and(|l| l.make)
    }

    /// True when nothing is typed: a blank name and every value empty.
    pub(crate) fn blank(&self) -> bool {
        text::blank(self.name.text()) && self.slots.iter().all(|s| s.value.chars() == 0)
    }

    /// What it hands over to save, once a kind is picked; each value is copied into a fresh wiped buffer.
    pub(crate) fn draft(&self) -> Option<Draft> {
        let kind = self.kind?;
        let field = |s: &Slot| Field {
            label: match &s.tag {
                Tag::Table(id) => Label::Table(id.clone()),
                Tag::Own(own) => Label::Own(Redacted::new(own.text().to_owned())),
            },
            value: copy(s.value.text()),
            hidden: s.hidden,
        };
        Some(Draft {
            name: Redacted::new(self.name.text().to_owned()),
            kind,
            fields: self.slots.iter().map(field).collect(),
        })
    }
}
