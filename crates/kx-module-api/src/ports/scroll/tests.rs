//! The scroll port and events: the key's gate, each event's name and gate, `Dir`, and `Debug` that hides every point
//! and box.

use super::*;
use crate::capability::is_kebab;
use crate::ports::{
    AssistAsk, AssistChanged, AssistNote, AssistToggle, LatchesAsk, LatchesChanged, LatchesUsed,
    LayoutChanged, PointerMoved, PointerPressed, PowerAsk, PowerAsked, PowerChanged,
};
use crate::{ModuleStateChanged, Notice, REDACTED_MARKER, SettingsChanged};
use std::collections::HashSet;

/// A point and a box whose numbers appear nowhere else in the output.
const AT: Point = Point { x: 4321, y: 8765 };
const AREA: Rect = Rect {
    x: 1357,
    y: 2468,
    w: 9753,
    h: 8642,
};
/// Their numbers, which no output may show.
const HIDDEN: [&str; 6] = ["4321", "8765", "1357", "2468", "9753", "8642"];

/// An event's name and gate.
fn row<E: Event>() -> (&'static str, Option<Capability>) {
    (E::NAME, E::CAPABILITY)
}

/// The scroll events, in the order `events_have_their_names_and_no_gate` checks.
fn scroll() -> [(&'static str, Option<Capability>); 7] {
    [
        row::<ScrollTap>(),
        row::<ScrollStop>(),
        row::<ScrollPeek>(),
        row::<ScrollChanged>(),
        row::<ScrollAsk>(),
        row::<ScrollAimed>(),
        row::<ScrollPage>(),
    ]
}

/// Every other event name in the crate.
fn older() -> [&'static str; 16] {
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
        PowerAsked::NAME,
        PowerChanged::NAME,
        PowerAsk::NAME,
    ]
}

/// An aim at `AT` with the box `AREA`.
fn aim() -> Aim {
    Aim {
        at: AT,
        on_pointer: true,
        area: Some(AREA),
        ours: false,
        id: TargetId::new(7),
    }
}

#[test]
fn the_target_is_named_and_gated_by_scroll() {
    assert_eq!(ScrollTargetKey::ID.as_str(), "scroll-target");
    assert_eq!(ScrollTargetKey::CAPABILITY, Some(Capability::Scroll));
    assert_eq!(Capability::Scroll.name(), "scroll");
}

#[test]
fn events_have_their_names_and_no_gate() {
    let want = [
        "scroll-tap",
        "scroll-stop",
        "scroll-peek",
        "scroll-changed",
        "scroll-ask",
        "scroll-aimed",
        "scroll-page",
    ];
    assert_eq!(scroll().map(|r| r.0), want);
    let gates = scroll().map(|r| r.1);
    assert_eq!(gates, [None; 7], "the keyboard and the UI hold no Scroll");
}

#[test]
fn names_are_kebab_case_and_unique_in_the_crate() {
    let ours = scroll().map(|r| r.0);
    assert!(ours.iter().all(|n| is_kebab(n)), "{ours:?}");
    let others: HashSet<_> = older().into_iter().collect();
    let mine: HashSet<_> = ours.iter().copied().collect();
    assert_eq!(mine.len(), ours.len(), "{ours:?}");
    assert!(mine.is_disjoint(&others), "{ours:?}");
}

#[test]
fn each_dir_has_an_opposite_on_its_own_axis() {
    assert_eq!(Dir::Up.opposite(), Dir::Down);
    assert_eq!(Dir::Left.opposite(), Dir::Right);
    for dir in Dir::ALL {
        assert_ne!(dir.opposite(), dir);
        assert_eq!(dir.opposite().opposite(), dir);
        assert_eq!(dir.opposite().sideways(), dir.sideways());
    }
    let sideways: Vec<_> = Dir::ALL.into_iter().filter(|d| d.sideways()).collect();
    assert_eq!(sideways, [Dir::Left, Dir::Right]);
}

#[test]
fn a_dir_is_written_in_lower_case() {
    let want = [r#""up""#, r#""down""#, r#""left""#, r#""right""#];
    for (dir, json) in Dir::ALL.into_iter().zip(want) {
        assert_eq!(serde_json::to_string(&dir).unwrap(), json);
        assert_eq!(serde_json::from_str::<Dir>(json).unwrap(), dir);
    }
    assert!(serde_json::from_str::<Dir>(r#""Up""#).is_err());
    assert!(serde_json::from_str::<Dir>(r#""top""#).is_err());
}

#[test]
fn aims_and_outlines_hide_every_point_and_box() {
    let texts = [
        format!("{:?}", aim()),
        format!("{:?}", ScrollAimed::from(aim())),
    ];
    for text in texts {
        assert!(text.contains(REDACTED_MARKER), "{text}");
        for hidden in HIDDEN {
            assert!(!text.contains(hidden), "{text} shows {hidden}");
        }
    }
}

#[test]
fn an_aim_still_shows_its_flags_and_id() {
    let r = REDACTED_MARKER;
    let want = format!(
        "Aim {{ at: {r}, on_pointer: true, area: Some({r}), ours: false, id: TargetId(7) }}"
    );
    assert_eq!(format!("{:?}", aim()), want);
    let none = ScrollAimed::NONE;
    assert_eq!(
        format!("{none:?}"),
        "ScrollAimed { area: None, ours: false }"
    );
    assert_eq!(TargetId::new(7).get(), 7);
}

/// The outline of an aim takes its box and whether it is ours; none outlines nothing.
#[test]
fn an_outline_takes_its_aims_box_and_whether_it_is_ours() {
    let theirs = ScrollAimed::from(aim());
    assert_eq!((theirs.area, theirs.ours), (Some(AREA), false));
    let list = ScrollAimed::from(Aim {
        ours: true,
        ..aim()
    });
    assert_eq!((list.area, list.ours), (Some(AREA), true));
    assert_eq!(
        (ScrollAimed::NONE.area, ScrollAimed::NONE.ours),
        (None, false)
    );
}

#[test]
fn each_error_says_something_else() {
    let errors = [
        ScrollError::NoTarget,
        ScrollError::Gone,
        ScrollError::Refused,
    ];
    let texts: HashSet<_> = errors.iter().map(ToString::to_string).collect();
    assert_eq!(texts.len(), errors.len());
}

#[test]
fn off_is_a_module_that_does_not_run_or_scroll() {
    let off = ScrollChanged::OFF;
    assert!(!off.live && !off.fast);
    assert_eq!(off.dir, None);
}
