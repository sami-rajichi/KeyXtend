//! The topmost band rules the guard follows after a move: which window is out, which one it lifts back, and the order
//! it brings the windows that one owns back in. Pure, so they are tested everywhere.

use super::WS_EX_TOPMOST;

/// True when extended style `ex` puts the window in the topmost band.
pub(super) fn is_top(ex: u32) -> bool {
    ex & WS_EX_TOPMOST != 0
}

/// True when a window of `chain`, as (topmost, lifting), is out of the topmost band: the guard checks it again later,
/// even while a running lift skips it now.
pub(super) fn dropped(chain: &[(bool, bool)]) -> bool {
    chain.iter().any(|&(top, _)| !top)
}

/// Which window the guard lifts back, from `chain`: the window that moved, then each guarded owner, as (topmost,
/// lifting). When one left the topmost band the furthest owner goes up, so the windows it owns follow; none while one
/// of them is lifting, so a refused lift cannot loop.
pub(super) fn to_lift(chain: &[(bool, bool)]) -> Option<usize> {
    let lifting = chain.iter().any(|&(_, lifting)| lifting);
    if dropped(chain) && !lifting {
        chain.len().checked_sub(1)
    } else {
        None
    }
}

/// The windows `root` owns, directly or not, in the order the guard brings them to the front after lifting it: each
/// right after its owner, so it lands above it, and siblings bottom first, so they keep their order. `owned` holds
/// (window, owner) pairs, front first.
pub(super) fn family<T: Copy + PartialEq>(root: T, owned: &[(T, T)]) -> Vec<T> {
    let mut found = Vec::new();
    let mut next = vec![root];
    while let Some(owner) = next.pop() {
        if owner != root {
            found.push(owner);
        }
        let owns = owned.iter().filter(|&&(w, o)| o == owner && w != root);
        next.extend(owns.map(|&(w, _)| w));
    }
    found
}

#[cfg(test)]
mod rules {
    use super::super::KEPT;
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn a_window_out_of_the_topmost_band_is_lifted_back_once() {
        assert_eq!(to_lift(&[(is_top(KEPT & !WS_EX_TOPMOST), false)]), Some(0));
        assert_eq!(to_lift(&[(false, true)]), None, "a lift is running");
        assert_eq!(to_lift(&[(is_top(KEPT), false)]), None, "still topmost");
        assert_eq!(to_lift(&[]), None);
    }

    #[test]
    fn a_window_out_of_the_band_is_checked_again_even_while_a_lift_skips_it() {
        assert!(dropped(&[(true, false), (false, false)]), "an owner");
        assert!(dropped(&[(false, true)]), "skipped by a running lift");
        assert!(!dropped(&[(true, false), (true, true)]), "all topmost");
        assert!(!dropped(&[]));
    }

    #[test]
    fn a_dropped_keyboard_or_shadow_lifts_the_shadow_so_the_keyboard_follows() {
        assert_eq!(
            to_lift(&[(false, false), (true, false)]),
            Some(1),
            "keyboard"
        );
        assert_eq!(to_lift(&[(true, false), (false, false)]), Some(1), "shadow");
        assert_eq!(to_lift(&[(true, false), (true, false)]), None);
        assert_eq!(
            to_lift(&[(false, false), (false, true)]),
            None,
            "the shadow's lift brings it"
        );
    }

    #[test]
    fn a_lifted_shadow_brings_its_family_back_each_above_its_owner_and_siblings_as_they_were() {
        // Front first: the list over its shadow, then the tip, all owned by the keyboard, which the shadow (1) owns.
        let (kb, tip, glow, list, other) = (2, 3, 4, 5, 9);
        let owned = [(list, glow), (glow, kb), (tip, kb), (kb, 1), (other, 8)];
        assert_eq!(family(1, &owned), [kb, tip, glow, list]);
        assert_eq!(family(glow, &owned), [list], "a branch alone");
        assert!(family(list, &owned).is_empty(), "nothing owned");
        let (front, back) = ([(10, 11), (11, 1)], [(20, 21), (21, 1)]);
        let both: Vec<_> = front.into_iter().chain(back).collect();
        assert_eq!(
            family(1, &both),
            [21, 20, 11, 10],
            "the window behind, with its own, goes first"
        );
    }

    proptest! {
        #[test]
        fn a_family_lists_each_window_once_after_its_owner(owners in prop::collection::vec(0u8..12, 0..12)) {
            // Window i + 1 is owned by a window numbered below it, so owners never loop.
            let owned: Vec<(u8, u8)> = owners.iter().zip(1u8..).map(|(&o, w)| (w, o % w)).collect();
            let got = family(0, &owned);
            for (i, w) in got.iter().enumerate() {
                let owner = owned.iter().find(|(x, _)| x == w).map(|&(_, o)| o);
                prop_assert!(owner == Some(0) || got[..i].iter().any(|&b| Some(b) == owner));
                prop_assert!(!got[..i].contains(w), "once each");
            }
            prop_assert_eq!(got.len(), owned.len(), "every window descends from 0");
        }

        #[test]
        fn only_the_furthest_owner_is_lifted_and_never_during_a_lift(chain: Vec<(bool, bool)>) {
            let picked = to_lift(&chain);
            prop_assert!(picked.is_none() || picked == Some(chain.len() - 1));
            let needed = chain.iter().any(|l| !l.0) && chain.iter().all(|l| !l.1);
            prop_assert_eq!(picked.is_some(), needed);
            prop_assert!(picked.is_none() || dropped(&chain), "every lift is checked again");
        }
    }
}
