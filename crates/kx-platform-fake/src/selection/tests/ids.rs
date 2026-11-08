//! Which box a read or a `still` is for: a box focus left is closed and not counted, and a box with no id (0) is
//! never the box asked about, as the platform's rule says.

use super::*;
use kx_module_api::Redacted;

#[test]
fn a_read_for_a_box_focus_left_is_closed_and_not_counted() {
    let fake = FakeSelectionWatch::new();
    let (mut run, _) = watched(&fake);
    fake.set_text(ARABIC);
    fake.focus(selected());
    let old = fake.spot().box_id;
    fake.focus(selected());
    assert_eq!(
        run.read(old, 5).map(Redacted::into_inner),
        Err(SelError::Closed)
    );
    assert_eq!(fake.calls().reads, 0, "never reached");
    assert_eq!(
        read(&fake, run.as_mut(), 5),
        Ok("مرحبا".to_owned()),
        "the box in focus"
    );
}

/// The fake keeps the platform's rule: a box with no id (0) is never the box asked about.
#[test]
fn a_box_with_no_id_is_never_still_and_never_read() {
    let fake = FakeSelectionWatch::new();
    let (mut run, _) = watched(&fake);
    fake.set_text(ARABIC);
    fake.change(selected());
    assert_eq!(fake.spot().box_id, 0, "no focus gave it an id");
    assert!(!run.still(0));
    let read = run.read(0, 5).map(Redacted::into_inner);
    assert_eq!(read, Err(SelError::Closed));
    assert_eq!(fake.calls().reads, 0, "never reached");
}
