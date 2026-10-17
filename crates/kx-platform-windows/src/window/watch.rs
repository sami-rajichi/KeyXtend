//! The entry watch's rules (ADR-0029 decision 4): where the pointer is for our watched window after each thing Windows
//! says, whether to ask for the next leave, and which move is an entry. Pure, so they are tested everywhere.

/// Where the pointer is for the watched window.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Inside {
    /// Not on it: the next move is an entry.
    Out,
    /// On it since a move: a leave under capture may ask again.
    Moved,
    /// On it, and a leave under capture asked again: the next one waits for a move.
    Asked,
}

impl Inside {
    /// The number a window property holds for it; `Out` holds none.
    pub(super) const fn mark(self) -> usize {
        match self {
            Self::Out => 0,
            Self::Moved => 1,
            Self::Asked => 2,
        }
    }

    /// The state a window property's number `n` holds; any other number is `Out`.
    pub(super) const fn from_mark(n: usize) -> Self {
        match n {
            1 => Self::Moved,
            2 => Self::Asked,
            _ => Self::Out,
        }
    }
}

/// What Windows said about the watched window.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Seen {
    /// The guard began to watch it, with the pointer `on` it or not.
    Mark { on: bool },
    /// The pointer moved on it.
    Move,
    /// The pointer left it, `captured` by it or not.
    Leave { captured: bool },
    /// It lost the capture with the pointer `on` it or not.
    Lost { on: bool },
}

/// One step of the watch: the new state, whether to ask Windows for the next leave, and whether the move was an entry.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) struct Step {
    pub(super) next: Inside,
    pub(super) track: bool,
    pub(super) entry: bool,
}

impl Step {
    /// To `next`, asking for the next leave.
    const fn ask(next: Inside) -> Self {
        Self {
            next,
            track: true,
            entry: false,
        }
    }

    /// To `next`, asking nothing.
    const fn rest(next: Inside) -> Self {
        Self {
            next,
            track: false,
            entry: false,
        }
    }
}

/// The step from `now` after `seen`: a move from out is the entry and every move asks for the leave; a leave under
/// capture, as a pressed key gives, asks once until the next move; a leave without capture, or a capture lost off the
/// window, ends the entry.
pub(super) const fn step(now: Inside, seen: Seen) -> Step {
    use Inside::{Asked, Moved, Out};
    match (now, seen) {
        (Out, Seen::Move) => Step {
            next: Moved,
            track: true,
            entry: true,
        },
        (_, Seen::Mark { on: true }) | (Moved | Asked, Seen::Move | Seen::Lost { on: true }) => {
            Step::ask(Moved)
        }
        (Moved, Seen::Leave { captured: true }) => Step::ask(Asked),
        (
            _,
            Seen::Mark { on: false } | Seen::Leave { captured: false } | Seen::Lost { on: false },
        ) => Step::rest(Out),
        (Out | Asked, Seen::Leave { captured: true }) | (Out, Seen::Lost { on: true }) => {
            Step::rest(now)
        }
    }
}

#[cfg(test)]
mod rules {
    use super::*;
    use proptest::prelude::*;

    /// Every state.
    const ALL: [Inside; 3] = [Inside::Out, Inside::Moved, Inside::Asked];

    /// Anything Windows may say.
    fn seen() -> impl Strategy<Value = Seen> {
        prop_oneof![
            any::<bool>().prop_map(|on| Seen::Mark { on }),
            Just(Seen::Move),
            any::<bool>().prop_map(|captured| Seen::Leave { captured }),
            any::<bool>().prop_map(|on| Seen::Lost { on }),
        ]
    }

    /// Any state.
    fn inside() -> impl Strategy<Value = Inside> {
        prop::sample::select(ALL.to_vec())
    }

    /// The states `run` goes through from `from`, with each step.
    fn walk(from: Inside, run: &[Seen]) -> Vec<(Inside, Seen, Step)> {
        let mut now = from;
        let mut steps = Vec::new();
        for &seen in run {
            let s = step(now, seen);
            steps.push((now, seen, s));
            now = s.next;
        }
        steps
    }

    #[test]
    fn each_state_keeps_its_own_mark_and_others_read_as_out() {
        for i in ALL {
            assert_eq!(Inside::from_mark(i.mark()), i);
        }
        assert_eq!(Inside::Out.mark(), 0, "no property");
        assert_eq!(Inside::from_mark(usize::MAX), Inside::Out);
    }

    #[test]
    fn a_mark_with_the_pointer_on_the_window_asks_for_the_leave_and_one_off_it_waits_for_an_entry()
    {
        for now in ALL {
            assert_eq!(step(now, Seen::Mark { on: true }), Step::ask(Inside::Moved));
            assert_eq!(step(now, Seen::Mark { on: false }), Step::rest(Inside::Out));
        }
    }

    #[test]
    fn a_capture_lost_on_the_window_while_out_keeps_it_out_so_the_next_move_enters() {
        let lost = step(Inside::Out, Seen::Lost { on: true });
        assert_eq!(lost, Step::rest(Inside::Out));
        assert!(step(lost.next, Seen::Move).entry);
    }

    #[test]
    fn a_leave_without_capture_after_a_leave_under_it_ends_the_entry() {
        let asked = step(Inside::Moved, Seen::Leave { captured: true });
        assert_eq!(asked, Step::ask(Inside::Asked));
        let left = step(asked.next, Seen::Leave { captured: false });
        assert_eq!(left, Step::rest(Inside::Out));
        assert!(step(left.next, Seen::Move).entry, "a new entry");
    }

    proptest! {
        #[test]
        fn only_the_first_move_after_out_is_an_entry(from in inside(), run in prop::collection::vec(seen(), 0..64)) {
            for (now, seen, s) in walk(from, &run) {
                prop_assert_eq!(s.entry, seen == Seen::Move && now == Inside::Out);
                prop_assert!(!s.entry || s.next == Inside::Moved, "an entry is on the window");
            }
        }

        #[test]
        fn a_run_of_leaves_asks_again_once_at_most(from in inside(), run in prop::collection::vec(seen(), 0..64)) {
            let mut asks = 0;
            for (_, seen, s) in walk(from, &run) {
                match seen {
                    Seen::Leave { .. } => asks += usize::from(s.track),
                    _ => asks = 0,
                }
                prop_assert!(asks <= 1, "a leave Windows answers at once cannot loop");
            }
        }

        #[test]
        fn a_leave_without_capture_or_a_capture_lost_off_the_window_ends_the_entry(
            from in inside(),
            run in prop::collection::vec(seen(), 0..64),
        ) {
            for (_, seen, s) in walk(from, &run) {
                if matches!(seen, Seen::Leave { captured: false } | Seen::Lost { on: false }) {
                    prop_assert_eq!(s, Step::rest(Inside::Out));
                }
                prop_assert!(!s.track || s.next != Inside::Out, "a leave is asked only while on the window");
            }
        }
    }
}
