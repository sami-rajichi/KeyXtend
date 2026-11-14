//! Every short address and `Location` over an alphabet of tricky characters: what reads, reads back the same, and a
//! relative `Location` never leaves its host. A sweep in place of a fuzz target, with no new dependency.

use crate::Target;
use kx_ipc::{HTTPS, rules};

/// Characters that mark a part of an address, or that a reader might take for one.
const TRICKY: &[char] = &[
    'a', 'A', '1', '.', '-', ':', '@', '/', '\\', '?', '#', '[', ']', '%', ' ',
];
/// The longest text the sweep builds, in characters.
const LONGEST: usize = 4;
/// The address every `Location` is read against.
const BASE: &str = "https://models.example/a/b?q";

/// Every text of up to `LONGEST` characters from `TRICKY`.
fn texts() -> Vec<String> {
    let (mut all, mut last) = (vec![String::new()], vec![String::new()]);
    for _ in 0..LONGEST {
        last = last
            .iter()
            .flat_map(|s| TRICKY.iter().map(move |c| format!("{s}{c}")))
            .collect();
        all.extend_from_slice(&last);
    }
    all
}

/// What every target holds: a plain host, a path from `/` without a fragment, and the same target read back.
fn sound(t: &Target) {
    assert!(!t.host().is_empty() && rules::host(t.host()).is_ok(), "{t}");
    assert!(t.path().starts_with('/') && !t.path().contains('#'), "{t}");
    assert_eq!(Target::parse(&t.to_string()).as_ref(), Ok(t), "{t}");
}

#[test]
fn every_short_address_reads_back_the_same_or_is_refused() {
    for tail in texts() {
        let read = Target::parse(&format!("{HTTPS}{tail}"));
        let auth = &tail[..tail.find(['/', '?', '#']).unwrap_or(tail.len())];
        if auth.contains(['@', '[', ']']) {
            assert!(read.is_err(), "{tail:?}");
        }
        read.iter().for_each(sound);
        Target::parse(&format!("{BASE}{tail}"))
            .iter()
            .for_each(sound);
    }
}

#[test]
fn a_short_location_keeps_its_host_unless_it_names_one() {
    let base = Target::parse(BASE).unwrap();
    for to in texts() {
        let Ok(t) = base.join(&to) else { continue };
        sound(&t);
        if !to.starts_with("//") {
            assert_eq!((t.host(), t.port()), (base.host(), base.port()), "{to:?}");
        }
    }
}
