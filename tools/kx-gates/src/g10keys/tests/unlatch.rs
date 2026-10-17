//! Tests of the undo that lets a failed Win step's latches go, over a model of the product's latches.

use std::cell::Cell;

use super::*;

/// Win's id and its spare's on the shipped board.
const KEYS: (&str, &str) = ("win", "shift");

/// The product's latches as Win's step meets them: Win on and used by a latch after it, the spare on, and how many
/// times Win went out alone, which opens Start.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Board {
    win: bool,
    used: bool,
    spare: bool,
    alone: usize,
}

impl Board {
    /// A click on `id`, as the product's latch takes it: latching uses the latches already on, and Win let go unused
    /// goes out alone.
    fn press(&mut self, id: &str) {
        match (id == KEYS.0, self.win, self.spare) {
            (true, true, _) => {
                self.alone += usize::from(!self.used);
                self.win = false;
            }
            (true, false, _) => (self.win, self.used) = (true, false),
            (false, _, true) => self.spare = false,
            (false, win, false) => (self.spare, self.used) = (true, self.used || win),
        }
    }

    /// The board after `presses`.
    fn after(mut self, presses: &[String]) -> Self {
        for id in presses {
            self.press(id);
        }
        self
    }

    /// True when both latches are let go and Win never went out alone.
    fn clear(self) -> bool {
        !self.win && !self.spare && self.alone == 0
    }
}

/// Every board: Win off, on and unused, or on and used, each with the spare off or on.
fn boards() -> Vec<Board> {
    let wins = [(false, false), (true, false), (true, true)];
    let each = |(win, used)| {
        [false, true].map(|spare| Board {
            win,
            used,
            spare,
            alone: 0,
        })
    };
    wins.into_iter().flat_map(each).collect()
}

/// Runs the undo on `start`, each spare press going out only when `spare_goes`: the board after, the presses that
/// went, and what the undo read last.
fn undone(start: Board, spare_goes: bool) -> (Board, usize, (Option<bool>, Option<bool>)) {
    let (b, went) = (Cell::new(start), Cell::new(0));
    let read = |id: &str| {
        Some(if id == KEYS.0 {
            b.get().win
        } else {
            b.get().spare
        })
    };
    let left = undo(KEYS, read, |id| {
        let goes = id == KEYS.0 || spare_goes;
        if goes {
            b.set(b.get().after(&[id.to_string()]));
            went.set(went.get() + 1);
        }
        goes
    });
    (b.get(), went.get(), left)
}

#[test]
fn wins_step_sends_nothing_from_a_clean_board_but_win_alone_after_a_spare_left_on() {
    let presses = step_of(false, "win").presses;
    assert!(Board::default().after(&presses).clear());
    let left_on = Board {
        spare: true,
        ..Board::default()
    };
    assert_eq!(left_on.after(&presses).alone, 1, "so `ready` refuses it");
}

#[test]
fn the_undo_lets_win_and_the_spare_go_from_any_board_and_never_sends_win_alone() {
    for start in boards() {
        let (end, went, left) = undone(start, true);
        assert!(end.clear(), "{start:?}: {end:?}");
        assert_eq!(left, (Some(false), Some(false)), "{start:?}");
        assert!(went <= UNDO_PRESSES, "{start:?}: {went} presses");
    }
}

#[test]
fn a_spare_that_may_have_latched_before_win_is_let_go_first_in_the_longest_undo() {
    let start = Board {
        win: true,
        spare: true,
        ..Board::default()
    };
    let (end, went, _) = undone(start, true);
    assert_eq!((end.alone, went), (0, UNDO_PRESSES));
}

#[test]
fn a_spare_press_that_does_not_go_stops_the_undo_before_win() {
    let start = Board {
        win: true,
        ..Board::default()
    };
    let (end, went, left) = undone(start, false);
    assert_eq!((end, went), (start, 0), "Win stays latched for the owner");
    assert_eq!(left, (Some(true), Some(false)));
}

#[test]
fn a_press_whose_key_reads_unread_after_stops_the_undo_before_win() {
    let b = Cell::new(Board {
        win: true,
        ..Board::default()
    });
    // The spare reads as unknown once it latched.
    let read = |id: &str| {
        let now = b.get();
        if id == KEYS.0 {
            Some(now.win)
        } else {
            (!now.spare).then_some(false)
        }
    };
    let left = undo(KEYS, read, |id| {
        b.set(b.get().after(&[id.to_string()]));
        true
    });
    assert_eq!(left, (Some(true), None));
    assert!(
        b.get().win && b.get().spare && b.get().alone == 0,
        "Win was never pressed"
    );
}

#[test]
fn an_unread_latch_stops_the_undo_with_nothing_pressed() {
    let pressed = Cell::new(false);
    let read = |id: &str| (id == KEYS.0).then_some(true);
    let left = undo(KEYS, read, |_| {
        pressed.set(true);
        true
    });
    assert!(!pressed.get());
    assert_eq!(left, (Some(true), None));
}

#[test]
fn the_next_undo_press_lets_the_spare_go_uses_win_or_lets_it_go_used() {
    let on = Some(true);
    let off = Some(false);
    assert_eq!(next(off, off, false), None, "both let go");
    assert_eq!(next(off, on, true), Some(Undo::Free));
    assert_eq!(
        next(None, on, false),
        Some(Undo::Free),
        "the spare sends nothing"
    );
    assert_eq!(next(on, off, false), Some(Undo::Use));
    assert_eq!(next(on, on, false), Some(Undo::Free), "it may predate Win");
    assert_eq!(next(on, on, true), Some(Undo::Key));
    assert_eq!(next(on, None, true), None, "unread");
    assert_eq!(next(None, off, false), None, "unread");
}
