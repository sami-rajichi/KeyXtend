//! The markers: which copies they skip, how their names compare, and which ones a put-back reads and a write sets.

use super::*;

fn marks(present: &[&str], zero: &[&str]) -> Marks {
    let owned = |n: &[&str]| n.iter().map(ToString::to_string).collect();
    Marks::new(owned(present), owned(zero))
}

#[test]
fn a_present_marker_skips_a_copy_whatever_else_it_holds() {
    let m = marks(
        &["Clipboard Viewer Ignore"],
        &["CanIncludeInClipboardHistory"],
    );
    assert!(m.marked(|n| n == "Clipboard Viewer Ignore", |_| Some(1)));
    assert!(!m.marked(|n| n == "HTML Format", |_| None), "plain copy");
}

#[test]
fn a_number_marker_skips_only_when_it_holds_zero() {
    let m = marks(&[], &["CanIncludeInClipboardHistory"]);
    assert!(m.marked(|_| false, |_| Some(0)));
    assert!(!m.marked(|_| false, |_| Some(1)));
    assert!(!m.marked(|_| false, |_| None), "absent");
}

#[test]
fn a_number_marker_that_cannot_be_named_or_read_counts_as_zero() {
    use crate::clip::marks::number;
    assert_eq!(number(None, |_| false, |_| None), Some(0), "not registered");
    assert_eq!(
        number(Some(1), |_| true, |_| None),
        Some(0),
        "there, unreadable"
    );
    assert_eq!(number(Some(1), |_| true, |_| Some(7)), Some(7), "read");
    assert_eq!(number(Some(1), |_| false, |_| Some(7)), None, "absent");
}

#[test]
fn marker_names_ignore_case_as_windows_does() {
    let m = marks(
        &["Clipboard Viewer Ignore"],
        &["CanIncludeInClipboardHistory"],
    );
    assert!(m.has("clipboard viewer ignore") && m.has("CANINCLUDEINCLIPBOARDHISTORY"));
    assert!(!m.has("PNG"));
}

#[test]
fn without_drops_one_marker_and_keeps_the_rest() {
    let m = Table::parse(TABLE).unwrap().marks();
    let fewer = m.without("canincludeinclipboardhistory");
    assert!(!fewer.has("CanIncludeInClipboardHistory"));
    assert_eq!(fewer.names().count(), m.names().count() - 1);
    assert!(
        m.names()
            .all(|n| fewer.has(n) || n == "CanIncludeInClipboardHistory")
    );
}

#[test]
fn a_put_back_checks_only_the_presence_markers_our_put_backs_never_set() {
    let t = Table::parse(TABLE).unwrap();
    let live = t.marks().presence();
    assert!(
        !live.has(&t.marker) && !live.has(&t.cloud_off),
        "our own write passes"
    );
    let shipped = [
        "ExcludeClipboardContentFromMonitorProcessing",
        "Clipboard Viewer Ignore",
    ];
    assert!(shipped.iter().all(|n| live.has(n)));
    assert!(
        !live.marked(|_| false, |_| Some(0)),
        "a zero alone is no presence marker"
    );
}

#[test]
#[cfg(windows)]
fn only_a_lent_write_sets_the_presence_markers_and_every_write_sets_ours() {
    let t = Table::parse(TABLE).unwrap();
    let (plain, lent) = (write::marks(&t, false), write::marks(&t, true));
    assert_eq!(plain, [t.marker.clone(), t.cloud_off.clone()]);
    let presence = t.marks().presence();
    assert!(
        plain.iter().all(|n| !presence.has(n)),
        "a put-back stays readable"
    );
    assert!(
        presence.names().all(|n| lent.iter().any(|l| l == n)),
        "each presence marker"
    );
    assert!(plain.iter().all(|n| lent.contains(n)), "and ours");
}
