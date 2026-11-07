//! The box: a padlock seals and the password opens; a wrong password, a cut, foreign, newer or forged box, a box with
//! no part, an unknown or repeated part, and a dropped, spliced or added chunk each fail cleanly; random bytes never
//! panic.

mod forge;
mod props;

use super::*;
use forge::{forged, open_with};
use kx_ipc::{BOX_MARK, BOX_VERSION, MIN_BOX_WORK, NOTEBOOK_PART};
use std::sync::OnceLock;

/// The tests' backup password, with an Arabic word in it.
const PASSWORD: &str = "zebra quartz كلمة";
/// The notebook the tests seal.
const NOTEBOOK: &[u8] = b"{\"entries\":[]} with \xd9\x85 bytes";

/// One padlock for every test, made once at the lowest work factor.
fn padlock_once() -> &'static Padlock {
    static ONE: OnceLock<Padlock> = OnceLock::new();
    ONE.get_or_init(|| padlock(PASSWORD, MIN_BOX_WORK).unwrap())
}

/// The notebook part holding `bytes`.
fn notebook(bytes: &[u8]) -> BoxPart<'_> {
    BoxPart {
        name: Cow::Borrowed(NOTEBOOK_PART),
        bytes: Wiped::Lent(bytes),
    }
}

/// A box of `bytes` as the notebook under the shared padlock.
fn sealed_with(bytes: &[u8]) -> Vec<u8> {
    let p = padlock_once();
    seal(&p.recipient, &p.locked, &[notebook(bytes)]).unwrap()
}

/// A box of the tests' notebook under the shared padlock.
fn sealed() -> Vec<u8> {
    sealed_with(NOTEBOOK)
}

#[test]
fn a_box_opens_with_its_password_to_its_parts_and_padlock() {
    let opened = open(&sealed(), PASSWORD).unwrap();
    let p = padlock_once();
    assert_eq!(opened.padlock.recipient, p.recipient);
    assert_eq!(opened.padlock.locked, p.locked);
    assert_eq!(opened.parts, [notebook(NOTEBOOK)]);
    assert!(p.recipient.starts_with("age1"), "an age X25519 recipient");
}

#[test]
fn a_box_starts_with_the_mark_and_version() {
    let bytes = sealed();
    assert_eq!(bytes[..BOX_MARK.len()], BOX_MARK);
    assert_eq!(bytes[BOX_MARK.len()], BOX_VERSION);
}

#[test]
fn sealing_twice_gives_other_bytes_that_open_the_same() {
    let (one, two) = (sealed(), sealed());
    assert_ne!(one, two);
    let parts = |b: &[u8]| open(b, PASSWORD).unwrap().parts;
    assert_eq!(parts(&one), parts(&two));
}

#[test]
fn an_empty_part_seals_and_opens_but_a_box_needs_a_part() {
    let empty = sealed_with(b"");
    assert_eq!(open_with(&empty).unwrap(), [notebook(b"")]);
    let p = padlock_once();
    assert_eq!(
        seal(&p.recipient, &p.locked, &[]).err(),
        Some(Fault::Internal)
    );
    assert_eq!(open_with(&forged(&[])).err(), Some(Fault::Damaged));
    assert_eq!(open(&forged(&[]), PASSWORD).err(), Some(Fault::Damaged));
}

#[test]
fn a_wrong_password_says_so_and_the_check_says_no() {
    assert_eq!(
        open(&sealed(), "zebra quartz").err(),
        Some(Fault::WrongPassword)
    );
    let locked = &padlock_once().locked;
    assert_eq!(check(locked, PASSWORD), Ok(true));
    assert_eq!(check(locked, ""), Ok(false));
}

#[test]
fn a_cut_or_longer_box_is_damaged_and_a_cut_head_before_any_key_is_derived() {
    let bytes = sealed();
    let head = layout::begin(&padlock_once().locked).unwrap().len();
    for cut in 0..=head {
        assert_eq!(
            layout::read(&bytes[..cut]).err(),
            Some(Fault::Damaged),
            "{cut}"
        );
    }
    for cut in head + 1..bytes.len() {
        assert_eq!(
            open_with(&bytes[..cut]).err(),
            Some(Fault::Damaged),
            "{cut}"
        );
    }
    let longer = [&bytes[..], b"x"].concat();
    assert_eq!(open_with(&longer).err(), Some(Fault::Damaged));
}

#[test]
fn another_mark_is_foreign_and_another_version_newer_or_damaged() {
    let bytes = sealed();
    let mut foreign = bytes.clone();
    foreign[0] ^= 0x20;
    assert_eq!(open(&foreign, PASSWORD).err(), Some(Fault::Foreign));
    assert_eq!(open(b"PK\x03\x04", PASSWORD).err(), Some(Fault::Foreign));
    let version = |v: u8| {
        let mut b = bytes.clone();
        b[BOX_MARK.len()] = v;
        open(&b, PASSWORD).err()
    };
    assert_eq!(version(BOX_VERSION + 1), Some(Fault::Unsupported));
    assert_eq!(version(u8::MAX), Some(Fault::Unsupported));
    assert_eq!(version(0), Some(Fault::Damaged));
}

#[test]
fn a_work_factor_past_the_top_is_damaged_before_scrypt_runs() {
    let p = padlock_once();
    let low = format!(" {MIN_BOX_WORK}\n");
    let high = format!(" {}\n", kx_ipc::MAX_BOX_WORK + 1);
    let at = p
        .locked
        .windows(low.len())
        .position(|w| w == low.as_bytes());
    let at = at.expect("the scrypt line names its work factor");
    let mut locked = p.locked.clone();
    locked[at..at + low.len()].copy_from_slice(high.as_bytes());
    assert_eq!(check(&locked, PASSWORD), Err(Fault::Damaged));
    let bytes = seal(&p.recipient, &locked, &[notebook(b"x")]).unwrap();
    assert_eq!(open(&bytes, PASSWORD).err(), Some(Fault::Damaged));
}

#[test]
fn an_unknown_part_is_unsupported_and_a_repeated_one_damaged() {
    let unknown = forged(&[(NOTEBOOK_PART, b"x"), ("settings", b"y")]);
    assert_eq!(open_with(&unknown).err(), Some(Fault::Unsupported));
    let repeated = forged(&[(NOTEBOOK_PART, b"x"), (NOTEBOOK_PART, b"y")]);
    assert_eq!(open_with(&repeated).err(), Some(Fault::Damaged));
    assert_eq!(open(&repeated, PASSWORD).err(), Some(Fault::Damaged));
}

#[test]
fn each_part_is_named_as_the_walk_reaches_it_so_a_long_list_is_never_walked_to_its_end() {
    let known = layout::head(NOTEBOOK_PART, 1).unwrap();
    let unknown = layout::head("settings", 0).unwrap();
    // The last part is cut short: a walk to the end would call the box damaged.
    let plain = [&known[..], b"x", &unknown, &[u8::MAX]].concat();
    assert_eq!(
        open_with(&forge::raw(&plain)).err(),
        Some(Fault::Unsupported)
    );
    let mut walk = layout::parts(&plain).map(|p| p.map(|(name, _)| name));
    assert_eq!(walk.next(), Some(Ok(NOTEBOOK_PART.as_bytes())));
}

#[test]
fn a_box_sealed_to_another_padlock_or_changed_is_damaged() {
    let other = padlock(PASSWORD, MIN_BOX_WORK).unwrap();
    let theirs = seal(&other.recipient, &padlock_once().locked, &[notebook(b"x")]).unwrap();
    assert_eq!(open_with(&theirs).err(), Some(Fault::Damaged));
    let mut changed = sealed();
    let last = changed.len() - 1;
    changed[last] ^= 1;
    assert_eq!(open_with(&changed).err(), Some(Fault::Damaged));
}

#[test]
fn a_dropped_or_spliced_chunk_is_damaged() {
    let (one, two) = (forge::big(1), forge::big(2));
    assert_eq!(open_with(&one).unwrap().len(), 1);
    let last = one.len() - forge::last_chunk();
    assert_eq!(open_with(&one[..last]).err(), Some(Fault::Damaged));
    let theirs = &two[two.len() - forge::last_chunk()..];
    let spliced = [&one[..last], theirs].concat();
    assert_eq!(open_with(&spliced).err(), Some(Fault::Damaged));
}

#[test]
fn a_bad_padlock_is_refused_before_sealing() {
    let p = padlock_once();
    let part = [notebook(b"x")];
    assert_eq!(
        seal("age1nope", &p.locked, &part).err(),
        Some(Fault::Damaged)
    );
    assert_eq!(
        seal(&p.recipient, b"not age", &part).err(),
        Some(Fault::Damaged)
    );
    assert_eq!(check(b"not age", PASSWORD), Err(Fault::Damaged));
}

#[test]
fn a_work_factor_outside_the_range_is_too_large() {
    for work in [0, MIN_BOX_WORK - 1, kx_ipc::MAX_BOX_WORK + 1] {
        assert_eq!(
            padlock(PASSWORD, work).err(),
            Some(Fault::TooLarge),
            "{work}"
        );
    }
}
