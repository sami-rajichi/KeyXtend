//! Properties over every shipped shortcut and any latched keys: each step presses a key once and lets go in reverse.

use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;

use super::{ENTRIES, Fake, KEYNAMES, KeyEvent, Plan, SHORTCUTS, Scan, Table, code, plan, table};

/// The keyboard's latches: Shift, right Shift, Ctrl, Win, Alt and right Alt.
const LATCHES: [u16; 6] = [0x2A, 0x36, 0x1D, 0xE05B, 0x38, 0xE038];
/// Any main-block scan code, plain or extended, beside the latches.
const ANY: std::ops::RangeInclusive<u16> = 0x01..=0x7F;
/// The prefix of an extended scan code.
const EXTENDED: u16 = 0xE000;

/// A latched set: up to six keys, each a latch or any key, repeats allowed.
fn latched() -> impl Strategy<Value = Vec<Scan>> {
    let any = prop_oneof![
        select(LATCHES.to_vec()),
        ANY,
        ANY.prop_map(|c| EXTENDED | c)
    ];
    vec(any.prop_map(Scan::new), 0..=6)
}

/// Fails unless every key `events` presses is up before it goes down again and all go up in reverse.
fn balanced(events: &[KeyEvent]) -> Result<(), TestCaseError> {
    let mut down: Vec<u16> = Vec::new();
    for e in events {
        let at = code(e.key);
        if e.up {
            prop_assert_eq!(down.pop(), Some(at), "released out of order");
        } else {
            prop_assert!(!down.contains(&at), "pressed twice at once");
            down.push(at);
        }
    }
    prop_assert!(down.is_empty(), "left down");
    Ok(())
}

proptest! {
    #[test]
    fn every_plan_is_balanced_and_presses_each_key_once(
        held in latched(),
        at in 0..ENTRIES,
        away in any::<bool>(),
    ) {
        let fake = Fake { away, ..Fake::default() };
        let info = &table().known()[at];
        let planned = plan(&fake, table(), &info.id, &held);
        prop_assert!(planned.is_ok());
        if let Ok(Plan::Keys(events)) = planned {
            prop_assert!(!events.is_empty());
            balanced(&events)?;
            for latch in &held {
                let downs = events.iter().filter(|e| !e.up && code(e.key) == latch.code());
                prop_assert_eq!(downs.count(), info.steps.len(), "a latch joins every step once");
            }
        }
    }

    #[test]
    fn no_table_text_panics_the_parser(shortcuts in ".{0,200}", names in ".{0,80}") {
        let _ = Table::parse(&shortcuts, &names);
        let _ = Table::parse(SHORTCUTS, &names);
        let _ = Table::parse(&shortcuts, KEYNAMES);
    }
}
