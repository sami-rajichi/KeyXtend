//! Property tests: random pieces, resets and reads never panic the tracker, its context never passes its cap, and no
//! plan erases more than the token holds.

use kx_module_api::Redacted;
use kx_module_api::ports::{FieldChange, FieldKind, Piece};
use proptest::prelude::*;

use super::*;

/// Random runs tried.
const CASES: u32 = 256;
/// The longest run of steps.
const MAX_STEPS: usize = 64;
/// The words kept before the current one in these tests.
const CONTEXT: usize = 2;
/// The longest word, short so long tokens are reached.
const MAX_WORD: usize = 6;
/// Characters the steps type: letters, an Arabic word with harakat, digits, both apostrophes, email marks, marks
/// and spaces.
const CHARS: &[char] = &[
    'a', 'b', 'L', 'é', 'e', '\u{0301}', 'ك', '\u{064E}', 'ا', 'ن', '1', '\'', '\u{2019}', '@',
    '.', '_', '-', '+', ',', ' ', '\u{00A0}', '\u{200C}',
];

/// One thing that can happen to the tracker.
#[derive(Debug, Clone)]
enum Step {
    /// Characters typed as one piece.
    Text(String),
    /// A Backspace.
    Erase,
    /// A word break.
    Break,
    /// A click outside our windows or a layout switch.
    Reset,
    /// Focus on box `n`.
    Focus(u64),
    /// A read of the box, or none.
    Read(Option<String>),
}

/// Some characters from `CHARS`.
fn text() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::sample::select(CHARS), 1..8).prop_map(|cs| cs.into_iter().collect())
}

/// Any step.
fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        4 => text().prop_map(Step::Text),
        2 => Just(Step::Erase),
        1 => Just(Step::Break),
        1 => Just(Step::Reset),
        1 => (0u64..3).prop_map(Step::Focus),
        1 => prop::option::of(text()).prop_map(Step::Read),
    ]
}

/// The language knows one elision and one whole word.
fn knows(word: &str) -> bool {
    matches!(word, "l'" | "a'b")
}

/// Applies `step` and gives what the tracker handed out.
fn apply(t: &mut Tracker, step: &Step) -> Vec<Told> {
    match step {
        Step::Text(s) => t.typed(&Piece::Text(Redacted::new(s.clone())), &knows),
        Step::Erase => t.typed(&Piece::Erase, &knows),
        Step::Break => t.typed(&Piece::Break, &knows),
        Step::Reset => t.reset(),
        Step::Focus(n) => t.focus(FieldChange {
            kind: FieldKind::Text,
            box_id: *n,
        }),
        Step::Read(text) => {
            t.read(text.as_deref(), &knows);
            Vec::new()
        }
    }
}

/// Every plan `show` offers, for a sample chip text.
fn plans(show: &Show) -> Vec<Plan> {
    let words = show.words.iter().filter_map(|q| q.plan("x"));
    let emails = show.emails.iter().filter_map(|e| e.plan("x@y.co"));
    words.chain(emails).collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn random_steps_keep_the_context_capped_and_plans_within_the_token(
        steps in prop::collection::vec(step(), 0..MAX_STEPS),
    ) {
        let mut t = Tracker::new(TrackConfig { context_words: CONTEXT, max_word: MAX_WORD });
        for step in &steps {
            for told in apply(&mut t, step) {
                if let Told::Learn { context, .. } = told {
                    prop_assert!(context.expose().len() <= CONTEXT);
                }
            }
            prop_assert!(t.context().is_none_or(|c| c.len() <= CONTEXT));
            let show = t.show(&knows);
            for reading in show.words.iter().flat_map(|q| &q.readings) {
                prop_assert!(reading.context.expose().len() <= CONTEXT);
            }
            for plan in plans(&show) {
                prop_assert!(usize::from(plan.erase) <= t.units());
            }
        }
    }
}
