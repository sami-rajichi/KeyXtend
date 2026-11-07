//! Property test: any run of keys, clicks (some from an older view) and notebook changes never panics, keeps the
//! highlight on the shown page and the page inside its count, keeps typed lines within `field_chars` and each caret
//! inside its line, asks only fills of kept fields, moves no list by a caret, and shows no hidden value on a page just
//! entered or whose entry was just replaced.

use kx_module_api::caret;
use kx_module_api::ports::{
    EditSheet, EntryKind, FieldValue, FillClick, FillFilter, FillPage, Hint, Label, PanelKey,
    Sheet, Typing,
};
use proptest::prelude::*;

use super::rig::{CONFIG, NOW, Rig, sample};
use super::{Do, Page, Panel, PanelConfig, Part, Sub};
use crate::book::{Limits, Notebook};
use crate::entry::text::STRIPPED;
use crate::fixture::{LIMITS, draft, field};
use steps::{Step, step};

/// Random runs tried.
const CASES: u32 = 1024;
/// The longest run of steps.
const MAX_STEPS: usize = 120;
/// The most extra notes a run starts with, so lists span pages.
const MAX_NOTES: usize = 9;
/// A short `field_chars`, so lines fill up.
const FIELD_CHARS: usize = 12;
/// A Make one length within the short `field_chars`.
const MAKE_CHARS: usize = 10;

/// Which page shows, so a page just entered is told apart.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Shows {
    Other,
    Entry(u64),
    Form,
}

/// The sample and `notes` more notes, under a short `field_chars`.
fn rig(notes: usize) -> Rig {
    let mut drafts = sample();
    let note = |n| {
        draft(
            &format!("n{n}"),
            EntryKind::Note,
            vec![field("text", "x", false)],
        )
    };
    drafts.extend((0..notes).map(note));
    let limits = Limits {
        field_chars: FIELD_CHARS,
        ..LIMITS
    };
    Rig::with(
        PanelConfig {
            limits,
            make_chars: MAKE_CHARS,
            ..CONFIG
        },
        drafts,
    )
}

/// The page `panel` shows.
fn shows(panel: &Panel) -> Shows {
    match &panel.page {
        Page::Entry(open) => Shows::Entry(open.id),
        Page::Edit(e) if !matches!(e.sub, Sub::Fields(_)) => Shows::Form,
        _ => Shows::Other,
    }
}

/// Applies `step`, giving what the panel said.
fn apply(r: &mut Rig, step: &Step) -> Do {
    match step {
        Step::Key(k) => r.key(k),
        Step::Click { stale: true, click } => r.stale(*click),
        Step::Click {
            stale: false,
            click,
        } => r.click(*click),
        Step::Remove(n) => {
            if let Some(id) = pick(r, *n) {
                r.book.remove(id);
            }
            r.draw();
            Do::Show
        }
        Step::Use(n) => {
            if let Some(id) = pick(r, *n) {
                r.book.use_at(id, NOW);
            }
            r.draw();
            Do::Show
        }
        Step::Replace(n) => {
            if let Some(id) = pick(r, *n) {
                let fields = vec![field("user", "u", false), field("password", "p", true)];
                let back = draft("r", EntryKind::Login, fields);
                r.book.edit(id, back, NOW, &LIMITS).ok();
            }
            r.draw();
            Do::Show
        }
    }
}

/// The id of the entry at place `n` of `r`'s notebook, wrapped.
fn pick(r: &Rig, n: usize) -> Option<u64> {
    let all = r.book.entries();
    n.checked_rem(all.len()).map(|at| all[at].id())
}

/// A fill names a kept entry's kept fields, with Tabs only between values.
fn fills(r: &Rig, done: &Do) -> Result<(), TestCaseError> {
    let Do::Fill(fill) = done else {
        return Ok(());
    };
    let entry = r.book.get(fill.id);
    prop_assert!(entry.is_some());
    let len = entry.map_or(0, |e| e.fields().len());
    prop_assert!(!fill.parts.is_empty());
    for (n, part) in fill.parts.iter().enumerate() {
        match part {
            Part::Value(i) => prop_assert!(*i < len),
            Part::Tab => prop_assert!(n > 0 && n + 1 < fill.parts.len()),
        }
    }
    Ok(())
}

/// The page is inside its count, the highlight and the typing on the shown rows, and no row past the page.
fn bounds(page: &FillPage, rows: usize) -> Result<(), TestCaseError> {
    prop_assert!(page.pages.at < page.pages.count);
    let len = match &page.sheet {
        Sheet::List(r) => r.len(),
        Sheet::Labels(r) => r.len(),
        Sheet::Values { rows, .. } => rows.len(),
        Sheet::Entry(e) => e.fields.len(),
        Sheet::Edit(e) => e.fields.len(),
        Sheet::AddField(l) => l.len(),
        other => return Err(TestCaseError::fail(format!("{other:?}"))),
    };
    prop_assert!(len <= rows);
    prop_assert!(page.highlight.is_none_or(|h| usize::from(h) < len));
    if let Sheet::Edit(e) = &page.sheet {
        let typed = match e.typing {
            Some(Typing::Label(p) | Typing::Value(p)) => Some(usize::from(p)),
            _ => None,
        };
        prop_assert!(typed.is_none_or(|p| p < len));
        prop_assert_eq!(page.highlight.map(usize::from), typed);
    } else {
        prop_assert_eq!(page.highlight.is_none(), len == 0);
    }
    Ok(())
}

/// How much `text` draws, as a caret counts it.
fn drawn(text: &str) -> u32 {
    caret::units(text, text.len())
}

/// How far the caret of the form's line that takes the letters may go, in what the line draws.
fn line_end(e: &EditSheet) -> u32 {
    let at = |p: u8| e.fields.get(usize::from(p));
    match e.typing {
        Some(Typing::Name) => drawn(e.name.expose()),
        Some(Typing::Label(p)) => at(p).map_or(0, |f| match &f.label {
            Label::Own(own) => drawn(own.expose()),
            Label::Table(_) => 0,
        }),
        Some(Typing::Value(p)) => at(p).map_or(0, |f| {
            let dots = f
                .value
                .count()
                .map(|n| u32::try_from(n).unwrap_or(u32::MAX));
            dots.unwrap_or_else(|| f.value.text().map_or(0, |t| drawn(t.expose())))
        }),
        None => 0,
    }
}

/// Each caret stays inside what its line draws, and the panel's on a char boundary of its text.
fn carets(page: &FillPage, panel: &Panel) -> Result<(), TestCaseError> {
    prop_assert!(*page.caret.expose() <= drawn(page.search.expose()));
    prop_assert!(panel.search.text().is_char_boundary(panel.search.caret()));
    if let Sheet::Edit(e) = &page.sheet {
        prop_assert!(*e.caret.expose() <= line_end(e));
    }
    Ok(())
}

/// The list a caret must not move: the filter, the search's text and the shown rows.
fn list_of(r: &Rig) -> (FillFilter, String, Vec<u64>) {
    let ids = r.rows().iter().map(|e| e.id).collect();
    (r.page.filter, r.page.search.expose().clone(), ids)
}

/// True when `step`, on the list page with `search` holding text, only moves the caret.
fn moves_caret_only(step: &Step, search: &str) -> bool {
    let moves = matches!(
        step,
        Step::Key(PanelKey::Left | PanelKey::Right)
            | Step::Click {
                click: FillClick::Caret { .. },
                ..
            }
    );
    moves && !caret::empty(search)
}

/// Every value the page carries, with whether its page may show a hidden one.
fn values(page: &FillPage) -> (Vec<&FieldValue>, bool) {
    match &page.sheet {
        Sheet::List(rows) => {
            let hints = rows.iter().filter_map(|r| match &r.hint {
                Hint::Values(v) => Some(v.iter()),
                Hint::Fields(_) => None,
            });
            (hints.flatten().collect(), false)
        }
        Sheet::Values { rows, .. } => (rows.iter().map(|v| &v.value).collect(), false),
        Sheet::Entry(e) => (e.fields.iter().map(|f| &f.value).collect(), true),
        Sheet::Edit(e) => (e.fields.iter().map(|f| &f.value).collect(), true),
        _ => (Vec::new(), false),
    }
}

/// The characters of `text` a field counts: the stripped marks left out.
fn counted(text: &str) -> usize {
    text.chars().filter(|c| !STRIPPED.contains(c)).count()
}

/// The longest text `book` holds, which an Edit line may keep past a lowered `field_chars`.
fn longest(book: &Notebook) -> usize {
    let texts = book.entries().iter().flat_map(|e| {
        let values = e.fields().iter().map(|f| counted(f.value.expose()));
        values.chain([counted(e.name().expose())])
    });
    texts.max().unwrap_or_default()
}

/// No hidden value with text shows but on its own page, and none on a page just entered; only the form tells a count,
/// and only of a value it masks; a New line fits `field_chars`, and an Edit line the longer of it and the longest saved
/// text.
fn hidden(page: &FillPage, entered: bool, saved: usize) -> Result<(), TestCaseError> {
    let (all, own) = values(page);
    let form = matches!(page.sheet, Sheet::Edit(_));
    for v in all {
        if v.hidden() && (!own || entered) {
            let blank = v.text().is_none_or(|t| t.expose().is_empty());
            prop_assert!(blank, "a hidden value showed");
        }
        prop_assert!(v.count().is_none() || form, "a read page told a length");
        prop_assert!(v.count().is_none() || v.text().is_none());
    }
    if let Sheet::Edit(e) = &page.sheet {
        let most = if e.new {
            FIELD_CHARS
        } else {
            FIELD_CHARS.max(saved)
        };
        prop_assert!(counted(e.name.expose()) <= most);
        let texts = e.fields.iter().filter_map(|f| f.value.text());
        prop_assert!(texts.into_iter().all(|t| counted(t.expose()) <= most));
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn any_run_keeps_the_panel_valid_and_shows_no_hidden_value_once_its_page_closed(
        notes in 0..=MAX_NOTES,
        steps in prop::collection::vec(step(), 0..MAX_STEPS),
    ) {
        let mut r = rig(notes);
        let saved = longest(&r.book);
        let mut was = shows(&r.panel);
        for step in &steps {
            let replaced = match step {
                Step::Replace(n) => pick(&r, *n).map(Shows::Entry),
                _ => None,
            };
            let list = matches!(r.panel.page, Page::List);
            let held = (list && moves_caret_only(step, r.page.search.expose())).then(|| list_of(&r));
            let done = apply(&mut r, step);
            fills(&r, &done)?;
            if let Some(held) = held {
                prop_assert_eq!(list_of(&r), held, "a caret moves no list");
            }
            if done == Do::Close {
                prop_assert!(!r.panel.is_open());
                r.panel.open();
                r.draw();
            }
            let now = shows(&r.panel);
            let rows = r.panel.rows();
            bounds(&r.page, rows)?;
            carets(&r.page, &r.panel)?;
            let entered = now != was && now != Shows::Other;
            hidden(&r.page, entered || replaced == Some(now.clone()), saved)?;
            was = now;
        }
    }
}

mod steps;
