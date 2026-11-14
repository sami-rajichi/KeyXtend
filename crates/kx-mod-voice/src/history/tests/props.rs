//! Property: any mix of keeps, recordings, pins, deletes and trims keeps the limits, never drops a pin but by an
//! explicit Delete, and gives edits that name only what changed.

use proptest::prelude::*;

use super::*;

/// One change to the history.
#[derive(Clone, Debug)]
enum Op {
    /// A text of this take.
    Keep(u32),
    /// A recording of this many bytes for this take.
    Sound(u32, u64),
    /// Pin the entry at this place, wrapped.
    Pin(usize),
    /// Unpin the entry at this place, wrapped.
    Unpin(usize),
    /// Delete the entry at this place, wrapped.
    Delete(usize),
    /// Ask, then delete all.
    All,
    /// Trim to these limits.
    Trim(usize, u64),
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        (0..6_u32).prop_map(Op::Keep),
        (0..6_u32, 0..40_u64).prop_map(|(t, b)| Op::Sound(t, b)),
        any::<usize>().prop_map(Op::Pin),
        any::<usize>().prop_map(Op::Unpin),
        any::<usize>().prop_map(Op::Delete),
        Just(Op::All),
        (1..5_usize, 0..60_u64).prop_map(|(m, v)| Op::Trim(m, v)),
    ]
}

/// The id at `at`, wrapped, if any.
fn pick(history: &History, at: usize) -> Option<u64> {
    let n = history.len();
    (n > 0).then(|| history.entries()[at % n].id)
}

/// Applies `op` and gives its edit, the ids it may delete although pinned, and whether it applied the limits.
fn apply(history: &mut History, op: &Op, r: &mut Rules) -> (Edit, Vec<u64>, bool) {
    match *op {
        Op::Keep(take) => (history.keep(kept(take, "t"), at(take), r), vec![], true),
        Op::Sound(take, bytes) => {
            let edit = history.admit(take, bytes, r).map(|id| {
                let blobs = 1;
                history.attach(id, Sound { bytes, blobs }, r)
            });
            let ran = edit.is_some();
            (edit.unwrap_or_default(), vec![], ran)
        }
        Op::Pin(at) => (
            history.pin(&pick(history, at).into_iter().collect::<Vec<_>>()),
            vec![],
            false,
        ),
        Op::Unpin(place) => (
            history.unpin(&pick(history, place).into_iter().collect::<Vec<_>>(), at(0)),
            vec![],
            false,
        ),
        Op::Delete(at) => {
            let ids: Vec<u64> = pick(history, at).into_iter().collect();
            (history.delete(&ids), ids, false)
        }
        Op::All => {
            let edit = history.ask_all().map(|asked| history.delete_all(asked));
            (edit.unwrap_or_default(), vec![], false)
        }
        Op::Trim(max, voice) => {
            (r.max, r.voice_bytes) = (max, voice);
            (history.trim(r), vec![], true)
        }
    }
}

proptest! {
    #[test]
    fn the_limits_hold_and_pins_stay(ops in prop::collection::vec(op(), 0..40)) {
        let mut r = rules(3, 30);
        let mut history = History::default();
        for op in &ops {
            let pins: Vec<u64> = history.entries().iter().filter(|e| e.pinned).map(|e| e.id).collect();
            let (edit, asked, limited) = apply(&mut history, op, &mut r);
            for id in &edit.put {
                prop_assert!(history.get(*id).is_some(), "a put names a text that is gone");
            }
            for id in edit.gone.keys() {
                prop_assert!(history.get(*id).is_none(), "a gone text stays");
                prop_assert!(!pins.contains(id) || asked.contains(id), "a pin left without Delete");
            }
            for id in edit.mute.keys() {
                prop_assert!(history.get(*id).is_none_or(|e| e.sound.is_none()), "a muted recording stays");
            }
            if limited {
                prop_assert!(history.unpinned() <= r.max, "past history_max");
                let pinned_only = history.entries().iter().all(|e| e.pinned || e.sound.is_none());
                prop_assert!(history.voice_bytes() <= r.voice_bytes || pinned_only, "past voice_max_mb");
            }
            let mut ids: Vec<u64> = history.entries().iter().map(|e| e.id).collect();
            ids.sort_unstable();
            ids.dedup();
            prop_assert_eq!(ids.len(), history.len(), "an id is used twice");
        }
    }
}
