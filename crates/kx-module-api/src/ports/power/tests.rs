//! The power port and events: the dialog's gate, each event's name and gate, and the errors.

use super::*;
use crate::capability::is_kebab;
use crate::ports::{
    AssistAsk, AssistChanged, AssistNote, AssistToggle, LatchesAsk, LatchesChanged, LatchesUsed,
    LayoutChanged, PointerMoved, PointerPressed, ScrollAimed, ScrollAsk, ScrollChanged, ScrollPage,
    ScrollPeek, ScrollStop, ScrollTap,
};
use crate::{ModuleStateChanged, Notice, SettingsChanged};
use std::collections::HashSet;

/// An event's name and gate.
fn row<E: Event>() -> (&'static str, Option<Capability>) {
    (E::NAME, E::CAPABILITY)
}

/// The power events, in the order `events_have_their_names_and_no_gate` checks.
fn power() -> [(&'static str, Option<Capability>); 3] {
    [
        row::<PowerAsked>(),
        row::<PowerChanged>(),
        row::<PowerAsk>(),
    ]
}

/// Every other event name in the crate.
fn older() -> [&'static str; 20] {
    [
        Notice::NAME,
        ModuleStateChanged::NAME,
        SettingsChanged::NAME,
        LayoutChanged::NAME,
        AssistToggle::NAME,
        AssistChanged::NAME,
        AssistAsk::NAME,
        AssistNote::NAME,
        PointerMoved::NAME,
        PointerPressed::NAME,
        LatchesChanged::NAME,
        LatchesUsed::NAME,
        LatchesAsk::NAME,
        ScrollTap::NAME,
        ScrollStop::NAME,
        ScrollPeek::NAME,
        ScrollChanged::NAME,
        ScrollAsk::NAME,
        ScrollAimed::NAME,
        ScrollPage::NAME,
    ]
}

#[test]
fn the_dialog_is_named_and_gated_by_power() {
    assert_eq!(PowerDialogKey::ID.as_str(), "power-dialog");
    assert_eq!(PowerDialogKey::CAPABILITY, Some(Capability::Power));
    assert_eq!(Capability::Power.name(), "power");
}

#[test]
fn events_have_their_names_and_no_gate() {
    let want = ["power-asked", "power-changed", "power-ask"];
    assert_eq!(power().map(|r| r.0), want);
    let gates = power().map(|r| r.1);
    assert_eq!(gates, [None; 3], "the keyboard and the UI hold no Power");
}

#[test]
fn names_are_kebab_case_and_unique_in_the_crate() {
    let ours = power().map(|r| r.0);
    assert!(ours.iter().all(|n| is_kebab(n)), "{ours:?}");
    let others: HashSet<_> = older().into_iter().collect();
    let mine: HashSet<_> = ours.iter().copied().collect();
    assert_eq!(mine.len(), ours.len(), "{ours:?}");
    assert!(mine.is_disjoint(&others), "{ours:?}");
}

#[test]
fn off_is_a_module_that_does_not_run() {
    assert_eq!(PowerChanged::OFF, PowerChanged { live: false });
}

#[test]
fn each_error_says_something_else() {
    let errors = [PowerError::Unavailable, PowerError::Failed];
    let texts: HashSet<_> = errors.iter().map(ToString::to_string).collect();
    assert_eq!(texts.len(), errors.len());
}
