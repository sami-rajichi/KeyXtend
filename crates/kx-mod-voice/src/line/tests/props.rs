//! Any run of letters, marks, Backspaces, Deletes, arrows and clicks on either kind of line: it never grows past its
//! room, holds no hidden mark, and keeps its caret on a character's edge inside its text.

use kx_module_api::is_hidden;
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;

use super::super::Line;
use super::{E_ACUTE, MOST, WIDE};

/// The most steps in one case.
const MAX_STEPS: usize = 60;

/// One thing done to a line.
#[derive(Clone, Debug)]
enum Op {
    Write(String),
    Erase,
    Delete,
    Step(bool),
    Place(u32),
}

/// Texts a step may type: plain, wide, joined and Arabic letters, a space, and marks no line takes.
fn texts() -> Vec<String> {
    let wide = WIDE.to_string();
    let more = ["a", "ab", E_ACUTE, "س", " ", "\n", "\u{200e}", "\t"];
    more.iter().map(|t| (*t).to_owned()).chain([wide]).collect()
}

/// Any step.
fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        select(texts()).prop_map(Op::Write),
        Just(Op::Erase),
        Just(Op::Delete),
        any::<bool>().prop_map(Op::Step),
        (0..u32::try_from(MOST * 2).unwrap_or(u32::MAX)).prop_map(Op::Place),
    ]
}

proptest! {
    #[test]
    fn any_edits_keep_the_line_in_its_room_with_no_hidden_mark(
        hidden in any::<bool>(),
        ops in vec(op(), 0..MAX_STEPS),
    ) {
        let mut line = Line::new(MOST, hidden);
        let room = line.room();
        for op in ops {
            let _ = match op {
                Op::Write(text) => line.write(&text),
                Op::Erase => line.erase(),
                Op::Delete => line.delete(),
                Op::Step(on) => line.step(on),
                Op::Place(at) => line.place(at),
            };
            prop_assert_eq!(line.room(), room, "never reallocated");
            prop_assert!(line.text().len() <= MOST + char::MAX_LEN_UTF8);
            prop_assert!(!line.text().chars().any(is_hidden));
            prop_assert!(line.text().is_char_boundary(line.at));
            prop_assert!(line.at <= line.text().len());
        }
    }
}
