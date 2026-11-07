//! Property test: random keys, clicks, copies, pins and deletes never panic, keep the highlight inside the list, the
//! ticks dense and in their order, the question's count right, the page inside its bounds and the caret inside its
//! line, and a caret moves no list.

mod steps;

use kx_module_api::ports::{ClipAct, ClipPage, Filter, PanelLine, Preview};
use kx_module_api::{Redacted, caret};
use proptest::prelude::*;

use self::steps::{Step, step};

use super::rig::{PANEL, now};
use super::*;
use crate::entry::Place;
use crate::find::Find;
use crate::fixture::{LIMITS, copied};
use crate::view::{Facts, build};

/// Random runs tried.
const CASES: u32 = 256;
/// The longest run of steps.
const MAX_STEPS: usize = 80;
/// The most rows a page shows in a run; 0 counts as 1.
const MAX_ROWS: usize = 6;
/// The most entries a run starts with.
const MAX_START: usize = 14;
/// The most entries a run keeps, so copies stop there.
const MAX_KEPT: usize = 24;
/// What the copies hold, so searches find some: accents, alef forms, a link and plain words.
const WORDS: [&str; 8] = [
    "école",
    "ECOLE du soir",
    "أرسلت",
    "ارسلت الملف",
    "note",
    "idée",
    "https://a.example",
    "rapport",
];

/// A run: the history, the panel, the last view's round.
struct Run {
    history: History,
    panel: Panel,
    round: u32,
}

impl Run {
    /// An open panel of `rows` rows over copies of `words`.
    fn new(rows: usize, words: &[usize]) -> Self {
        let mut history = History::default();
        for &n in words {
            history.add(copied(WORDS[n % WORDS.len()], &now()));
        }
        let mut panel = Panel::new(PanelConfig { rows, ..*PANEL });
        panel.open();
        Self {
            history,
            panel,
            round: 0,
        }
    }

    /// The id at place `n`, wrapped, if any entry is kept.
    fn pick(&self, n: usize) -> Option<u64> {
        let all = self.history.entries();
        all.get(n.checked_rem(all.len())?).map(|e| e.id)
    }

    /// Applies `step` and gives what the panel said.
    fn apply(&mut self, step: &Step) -> Do {
        let h = &mut self.history;
        match step {
            Step::Key(k) => self.panel.key(k, h),
            Step::Act(a) => self.panel.act(*a, h),
            Step::Tick { stale, row } => {
                let round = self.round.wrapping_sub(u32::from(*stale));
                self.panel.act(ClipAct::Tick { round, row: *row }, h)
            }
            Step::Copy(n) if h.len() < MAX_KEPT => {
                h.add(copied(WORDS[n % WORDS.len()], &now()));
                Do::Show
            }
            Step::Remove(n) => {
                if let Some(id) = self.pick(*n) {
                    self.history.remove(id);
                }
                Do::Show
            }
            Step::Pin(n) => {
                if let Some(id) = self.pick(*n)
                    && self.history.pin(id) == Some(Place::Pinned)
                {
                    self.history.unpin(id, &now());
                }
                Do::Show
            }
            Step::Copy(_) => Do::Stay,
        }
    }

    /// Does what the module would for `done`: the acts name kept entries once, and Yes deletes what the question
    /// counted.
    fn follow(&mut self, done: Do, asked: Option<&[u64]>) -> Result<(), TestCaseError> {
        if let Do::Insert(ids) | Do::Pin(ids) | Do::Unpin(ids) | Do::Delete(ids) = &done {
            let mut seen = ids.clone();
            seen.sort_unstable();
            seen.dedup();
            prop_assert!(!ids.is_empty() && seen.len() == ids.len());
            prop_assert!(ids.iter().all(|&id| self.history.get(id).is_some()));
        }
        let h = &mut self.history;
        match done {
            Do::Insert(ids) => ids.into_iter().for_each(|id| _ = h.raise(id, &now())),
            Do::Pin(ids) => ids.into_iter().for_each(|id| _ = h.pin(id)),
            Do::Unpin(ids) => ids.into_iter().for_each(|id| _ = h.unpin(id, &now())),
            Do::Delete(mut ids) => {
                if let Some(all) = asked {
                    ids.sort_unstable();
                    prop_assert_eq!(&ids, all, "Yes deletes what the question counted");
                }
                h.take_where(|e| ids.contains(&e.id));
            }
            Do::Preview(id) => {
                prop_assert!(self.history.get(id).is_some());
                let text = Redacted::new(String::new());
                self.panel.show(id, Preview::Text { text, chars: 0 });
            }
            Do::Close => {
                prop_assert!(!self.panel.is_open());
                self.panel.open();
            }
            Do::Stay | Do::Show => {}
        }
        Ok(())
    }

    /// Builds the view and checks it against the history, the panel and the ticks `before` the step.
    fn check(&mut self, before: &[u64]) -> Result<(), TestCaseError> {
        let facts = Facts {
            now: now(),
            room: None,
            note: None,
        };
        let view = build(&mut self.panel, &self.history, &facts, &LIMITS, |_| None);
        self.round = view.0.round;
        self.pages(&view.0)?;
        self.line(&view.0)?;
        self.ticks(&view.0, before)?;
        let free = unpinned(&self.history);
        prop_assert_eq!(usize::from(view.0.unpinned), free.len());
        let asked = self.panel.asked();
        prop_assert_eq!(view.0.confirm.map(usize::from), asked.map(<[u64]>::len));
        prop_assert!(asked.is_none_or(|ids| ids.iter().all(|id| free.contains(id))));
        let previewed = self.panel.preview().map(|(id, _)| id);
        prop_assert!(previewed.is_none_or(|id| self.history.get(id).is_some()));
        prop_assert_eq!(view.0.preview.is_some(), previewed.is_some());
        Ok(())
    }

    /// The line keeps its cap and its caret inside it, on a char boundary.
    fn line(&self, page: &ClipPage) -> Result<(), TestCaseError> {
        let text = page.search.expose();
        prop_assert!(text.chars().count() <= PANEL.query_chars);
        prop_assert!(text.is_char_boundary(*self.panel.caret.expose()));
        prop_assert!(*page.caret.expose() <= caret::units(text, text.len()));
        Ok(())
    }

    /// True when `step` only moves the caret: the list shows with no question and its line draws text.
    fn moves_caret_only(&self, step: &Step) -> bool {
        let aimed = matches!(self.panel.line(), PanelLine::Search(_));
        aimed && step.is_caret() && !caret::empty(self.panel.query().expose())
    }

    /// What a caret must not move: the filter, the search's text, the highlight and the list.
    fn list_of(&mut self) -> (Filter, String, Option<usize>, Vec<u64>) {
        let list = self.panel.list(&self.history);
        let hi = self.panel.highlight(&list);
        let ids = list.iter().map(|e| e.id).collect();
        (
            self.panel.filter(),
            self.panel.query().expose().clone(),
            hi,
            ids,
        )
    }

    /// The page is the right slice of the list, and the highlight is on it.
    fn pages(&self, page: &ClipPage) -> Result<(), TestCaseError> {
        let find = Find::new(self.panel.filter(), self.panel.query().expose());
        let list = find.list(self.history.entries());
        let rows = self.panel.rows();
        prop_assert_eq!(
            usize::from(page.pages.count),
            list.len().div_ceil(rows).max(1)
        );
        prop_assert!(page.pages.at < page.pages.count);
        let start = usize::from(page.pages.at) * rows;
        let want: Vec<u64> = list.iter().skip(start).take(rows).map(|e| e.id).collect();
        let shown: Vec<u64> = page.rows.iter().map(|r| r.id).collect();
        prop_assert_eq!(shown, want);
        prop_assert_eq!(page.highlight.is_none(), page.rows.is_empty());
        let on_page = |h: u8| usize::from(h) < page.rows.len();
        prop_assert!(page.highlight.is_none_or(on_page));
        Ok(())
    }

    /// The ticks are kept entries, once each, numbered 1 to n in tick order, and the step kept their order.
    fn ticks(&self, page: &ClipPage, before: &[u64]) -> Result<(), TestCaseError> {
        let ticks = self.panel.ticks();
        let mut seen = ticks.to_vec();
        seen.sort_unstable();
        seen.dedup();
        prop_assert_eq!(seen.len(), ticks.len());
        prop_assert!(ticks.iter().all(|&id| self.history.get(id).is_some()));
        prop_assert_eq!(usize::from(page.ticked), ticks.len());
        for r in &page.rows {
            let place = ticks.iter().position(|&t| t == r.id).map(|i| i + 1);
            prop_assert_eq!(r.tick.map(usize::from), place);
        }
        let kept: Vec<u64> = before
            .iter()
            .copied()
            .filter(|id| ticks.contains(id))
            .collect();
        prop_assert!(ticks.starts_with(&kept), "tick order kept");
        prop_assert!(ticks.len() <= kept.len() + 1, "one tick a step");
        Ok(())
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn random_keys_and_clicks_keep_the_highlight_the_ticks_and_the_page(
        rows in 0..=MAX_ROWS,
        words in prop::collection::vec(any::<usize>(), 0..MAX_START),
        steps in prop::collection::vec(step(), 0..MAX_STEPS),
    ) {
        let mut run = Run::new(rows, &words);
        run.check(&[])?;
        for step in &steps {
            let before = run.panel.ticks().to_vec();
            let asked = run.panel.asked().map(|ids| {
                let mut all = ids.to_vec();
                all.sort_unstable();
                all
            });
            let held = run.moves_caret_only(step).then(|| run.list_of());
            let done = run.apply(step);
            run.follow(done, asked.as_deref())?;
            run.check(&before)?;
            if let Some(held) = held {
                prop_assert_eq!(run.list_of(), held, "a caret moves no list");
            }
        }
    }
}
