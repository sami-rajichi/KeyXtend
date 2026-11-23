//! The board's property: whatever the job sends and whenever the clock moves, texts are told from index 0 in order,
//! within the run's cap, then at most one end and nothing after; a cancel comes only with an end the host decided.

use super::board::{Board, Step};
use super::*;
use crate::worker::{Lost, Told};
use kx_ipc::{MAX_NET_RUN_TEXT_BYTES, MAX_NET_TEXT_BYTES};
use kx_module_api::Mono;
use proptest::prelude::*;

/// One thing that happens to a run.
#[derive(Clone, Debug)]
enum Op {
    /// The job tells a text at this index, of this many bytes.
    Text(u16, usize),
    Done,
    Fault,
    Lost,
    /// The clock moves on by this many seconds, then the timer checks.
    Tick(u64),
    /// The run is dropped.
    Drop,
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        6 => (0u16..6, 0..=MAX_NET_TEXT_BYTES).prop_map(|(i, n)| Op::Text(i, n)),
        1 => Just(Op::Done),
        1 => Just(Op::Fault),
        1 => Just(Op::Lost),
        2 => (0u64..4_000).prop_map(Op::Tick),
        1 => Just(Op::Drop),
    ]
}

/// What `op` does to `board` at the time `now` moved to.
fn take(board: &mut Board, op: &Op, now: &mut Mono) -> Step {
    let note = |n| Told::Note(n);
    match op {
        Op::Text(index, n) => board.heard(note(text(*index, &"a".repeat(*n))), *now),
        Op::Done => board.heard(note(NetNote::Done { id: ID }), *now),
        Op::Fault => board.heard(note(fault(NetFault::Rate)), *now),
        Op::Lost => board.heard(Told::Lost(Lost::Gone), *now),
        Op::Tick(s) => {
            *now = now.saturating_add(Duration::from_secs(*s));
            board.tick(*now)
        }
        Op::Drop => {
            board.drop_run();
            Step::default()
        }
    }
}

proptest! {
    #[test]
    fn texts_go_in_order_within_the_cap_then_one_end_and_nothing_after(
        count in 0usize..5,
        ops in prop::collection::vec(op(), 0..40),
    ) {
        let mut board = Board::new(vec![PER_S; count], WAIT_S, TRIES, kx_ipc::NET_PACE_S);
        let mut now = Mono::default();
        board.start(now);
        let (mut told, mut bytes, mut ended) = (0usize, 0usize, false);
        for op in &ops {
            let step = take(&mut board, op, &mut now);
            for note in &step.notes {
                prop_assert!(!ended, "nothing after an end");
                match note {
                    CloudNote::Piece { index, text } => {
                        prop_assert_eq!(usize::from(*index), told);
                        prop_assert!(told < count);
                        told += 1;
                        bytes += text.expose().len();
                    }
                    CloudNote::Done => {
                        prop_assert_eq!(told, count);
                        ended = true;
                    }
                    CloudNote::Fault(_) => ended = true,
                }
            }
            let ours = matches!(
                step.notes.last(),
                Some(CloudNote::Fault(CloudFault::Gone | CloudFault::Strange | CloudFault::Timeout))
            );
            prop_assert!(!step.cancel || ours, "a cancel only with the host's own end");
            ended |= matches!(op, Op::Drop);
            prop_assert!(bytes <= MAX_NET_RUN_TEXT_BYTES);
        }
    }
}
