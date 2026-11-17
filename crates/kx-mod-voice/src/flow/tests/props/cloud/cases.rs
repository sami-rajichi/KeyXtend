//! The cases the cloud property once shrank to, run again by name: a saved seed replays another case once the steps'
//! odds change, so each failure the property found stays pinned here.

use super::steps::Told;
use super::*;

/// Each shrunk case, in order: a pick with no key at Start; an upload's end, then a stray Silent; the pick moved, the
/// local model wrote it, then Try again; an empty reply, the local model, then Try again; a password box took the
/// text, then a text box and Try again; and the pick left while listening, or after Stop, before a late hand-over.
fn cases() -> Vec<Vec<Wire>> {
    use Wire::{Key, Local, Over, Pick, Retry, Talk, Write};
    let base = || Wire::Base(Step::Key);
    let said = || Wire::Base(Step::Said(Said::Done, false));
    let sent = |told| Wire::Sent(told, false);
    vec![
        vec![Pick(3), Key, Talk, Key, Over],
        vec![
            base(),
            base(),
            Talk,
            Over,
            sent(Told::Done),
            sent(Told::Silent),
        ],
        vec![base(), Talk, Write, Pick(2), Local, Write, Retry],
        vec![base(), Talk, Over, sent(Told::Done), Local, Write, Retry],
        vec![
            Key,
            Talk,
            said(),
            Wire::Box(true),
            Write,
            Wire::Box(false),
            Retry,
        ],
        vec![Key, Talk, Pick(0), Local, Over, Write],
        vec![Key, Talk, Key, Pick(0), Local, Over, Write],
    ]
}

#[test]
fn the_cases_the_cloud_property_once_shrank_to_hold() {
    for (at, steps) in cases().iter().enumerate() {
        let held = run(steps, false, false);
        assert!(held.is_ok(), "case {at}: {held:?}");
    }
}
