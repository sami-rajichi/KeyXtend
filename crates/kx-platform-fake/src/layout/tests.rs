//! The fake layout source over the example and captured tables: keys, facts, switch requests and held keys.

use super::*;

const EXAMPLE: &str = "example";
const ID: LayoutId = LayoutId::new(0x040C_040C);
const OTHER: LayoutId = LayoutId::new(0x0409_0409);
const Q: Scan = Scan::new(0x10);
const ENTER: Scan = Scan::new(0x1C);
const MISSING: Scan = Scan::new(0x59);

fn example() -> FakeLayoutSource {
    FakeLayoutSource::golden(&[EXAMPLE]).unwrap()
}

fn other() -> LayoutFile {
    LayoutFile {
        id: OTHER,
        ..example_file()
    }
}

fn example_file() -> LayoutFile {
    LayoutFile::golden(EXAMPLE).unwrap()
}

#[test]
fn it_serves_the_example_table() {
    let fake = example();
    assert_eq!(fake.installed(), [ID]);
    assert_eq!(fake.foreground(), Some(ID));
    assert_eq!(fake.facts(ID).iso, "fr");
    let keys = fake.keys(ID, &[Q, MISSING]);
    assert_eq!((keys[0].base.as_str(), keys[0].shift.as_str()), ("a", "A"));
    assert_eq!(keys[1], KeyChars::default());
}

#[test]
fn an_unknown_layout_types_nothing_and_has_no_facts() {
    let fake = example();
    assert_eq!(fake.keys(OTHER, &[Q]), [KeyChars::default()]);
    assert_eq!(fake.facts(OTHER), LangFacts::default());
    assert!(!fake.types_any(OTHER, Q, &[]));
}

#[test]
fn controls_and_characters_type_something() {
    let fake = example();
    assert!(fake.types_any(ID, ENTER, &[]));
    assert!(fake.types_any(ID, Q, &[]));
    assert!(!fake.types_any(ID, MISSING, &[]));
}

#[test]
fn a_request_switches_the_app_in_front_and_is_recorded() {
    let fake = example();
    fake.add(other());
    assert_eq!(fake.installed(), [ID, OTHER]);
    assert_eq!(fake.request(OTHER), Ok(()));
    assert_eq!(fake.foreground(), Some(OTHER));
    assert_eq!(fake.requests(), [OTHER]);
}

#[test]
fn with_no_app_in_front_a_request_finds_no_target() {
    let fake = example();
    fake.set_foreground(None);
    assert_eq!(fake.request(ID), Err(SwitchError::NoTarget));
    assert_eq!(fake.foreground(), None);
    assert!(fake.requests().is_empty());
}

#[test]
fn an_app_that_does_not_follow_keeps_its_layout_until_the_test_switches_it() {
    let fake = example();
    fake.add(other());
    fake.set_follow(false);
    assert_eq!(fake.request(OTHER), Ok(()));
    assert_eq!(fake.foreground(), Some(ID), "asked, not switched");
    assert_eq!(fake.requests(), [OTHER]);
    fake.set_foreground(Some(OTHER));
    assert_eq!(fake.foreground(), Some(OTHER));
}

#[test]
fn a_request_can_fail() {
    let fake = example();
    assert_eq!(fake.request(OTHER), Err(SwitchError::NotInstalled));
    fake.set_error(Some(SwitchError::NoTarget));
    assert_eq!(fake.request(ID), Err(SwitchError::NoTarget));
    assert!(fake.requests().is_empty());
}

#[test]
fn a_removed_layout_leaves_the_list() {
    let fake = example();
    fake.add(other());
    fake.remove(ID);
    assert_eq!(fake.installed(), [OTHER]);
    assert_eq!(fake.keys(ID, &[Q]), [KeyChars::default()]);
}

#[test]
fn caps_and_the_app_in_front_follow_the_test() {
    let fake = example();
    assert!(!fake.caps_on());
    fake.set_caps(true);
    assert!(fake.caps_on());
    fake.set_foreground(None);
    assert_eq!(fake.foreground(), None);
}

#[test]
fn the_captured_tables_serve_english_french_and_right_to_left_arabic() {
    let fake = FakeLayoutSource::golden(&["en-us", "fr-fr", "ar-tn"]).unwrap();
    let facts: Vec<LangFacts> = fake
        .installed()
        .into_iter()
        .map(|id| fake.facts(id))
        .collect();
    let isos: Vec<&str> = facts.iter().map(|f| f.iso.as_str()).collect();
    assert_eq!(isos, ["en", "fr", "ar"]);
    let rtl: Vec<bool> = facts.iter().map(|f| f.rtl).collect();
    assert_eq!(rtl, [false, false, true]);
}

#[test]
fn a_missing_golden_table_is_an_error() {
    assert!(FakeLayoutSource::golden(&["no-such-layout"]).is_err());
}

mod held;
