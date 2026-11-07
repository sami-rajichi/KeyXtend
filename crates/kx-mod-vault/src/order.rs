//! The list's order and filters (design "List" and "By label"): the last used entries first, then A to Z by folded
//! name; each filter shows its kinds; By label counts the values under each label in use.

use std::cmp::Reverse;
use std::collections::BTreeMap;

use kx_module_api::Redacted;
use kx_module_api::ports::{EntryKind, FillFilter, Label, LabelCount};
use kx_module_api::text::fold_any;

use crate::entry::{Entry, Field, Stamp};
use crate::find::Find;
use crate::kinds::tables;

/// Last-used times by entry id that the list sorts by in place of the entries' own, so a fill moves no row while the
/// panel is open.
pub type Uses = BTreeMap<u64, Option<Stamp>>;

/// `entries` in the list's order: the last used first, then the never used, each A to Z by folded name, ties by id; an
/// entry `held` names sorts by its held time.
#[must_use]
pub fn sorted<'a>(entries: impl IntoIterator<Item = &'a Entry>, held: &Uses) -> Vec<&'a Entry> {
    let used = |e: &Entry| held.get(&e.id()).copied().unwrap_or_else(|| e.used());
    let mut all: Vec<&Entry> = entries.into_iter().collect();
    all.sort_by_cached_key(|e| (Reverse(used(e)), fold_any(e.name().expose()), e.id()));
    all
}

/// True when `filter` shows entries of `kind`: All shows every one, and By label groups every one.
#[must_use]
pub fn shows(filter: FillFilter, kind: EntryKind) -> bool {
    match filter {
        FillFilter::All | FillFilter::Labels => true,
        _ => tables().kind(kind).is_some_and(|r| r.filter == filter),
    }
}

/// The entries of `all` that `filter` shows and `find` keeps, in the list's order.
#[must_use]
pub fn listed<'a>(
    all: &'a [Entry],
    filter: FillFilter,
    find: &Find,
    held: &Uses,
) -> Vec<&'a Entry> {
    let shown = all.iter().filter(|e| shows(filter, e.kind()));
    sorted(find.list(shown), held)
}

/// The labels in use and how many values each has: table labels in the table's order, then own labels A to Z by
/// folded text, where an own label joins the first one of the same folded text met in the list's order.
#[must_use]
pub fn labels<'a>(all: impl IntoIterator<Item = &'a Entry>, held: &Uses) -> Vec<LabelCount> {
    let table = tables().labels();
    let mut counts = vec![0u32; table.len()];
    let mut own: BTreeMap<String, LabelCount> = BTreeMap::new();
    for field in sorted(all, held).into_iter().flat_map(Entry::fields) {
        match &field.label {
            Label::Table(id) => {
                if let Some(at) = table.iter().position(|l| &l.id == id) {
                    counts[at] = counts[at].saturating_add(1);
                }
            }
            Label::Own(text) => {
                let met = LabelCount {
                    label: field.label.clone(),
                    count: 0,
                };
                let row = own.entry(fold_any(text.expose())).or_insert(met);
                row.count = row.count.saturating_add(1);
            }
        }
    }
    let used = table.iter().zip(counts).filter(|(_, n)| *n > 0);
    let rows = used.map(|(l, count)| LabelCount {
        label: Label::Table(l.id.clone()),
        count,
    });
    rows.chain(own.into_values()).collect()
}

/// Every field under `label` with its entry, in the list's order; an own label matches by folded text.
#[must_use]
pub fn values<'a>(
    all: impl IntoIterator<Item = &'a Entry>,
    label: &Label,
    held: &Uses,
) -> Vec<(&'a Entry, &'a Field)> {
    let wanted = folded(label);
    let under = |e: &'a Entry| {
        e.fields()
            .iter()
            .filter(|f| folded(&f.label) == wanted)
            .map(move |f| (e, f))
    };
    sorted(all, held).into_iter().flat_map(under).collect()
}

/// A label as By label tells them apart: a table id, or an own text folded.
fn folded(label: &Label) -> Label {
    match label {
        Label::Table(id) => Label::Table(id.clone()),
        Label::Own(text) => Label::Own(Redacted::new(fold_any(text.expose()))),
    }
}

#[cfg(test)]
mod tests;
