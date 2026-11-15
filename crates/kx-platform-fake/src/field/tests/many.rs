//! Several watches at once, as the real adapter runs one per module: each live one hears every focus change, and a
//! dropped one stops hearing while the others go on.

use super::*;

#[test]
fn every_live_watch_hears_each_focus_and_a_dropped_one_stops_hearing() {
    let fake = FakeFieldWatch::new();
    let (first, first_heard) = watched(&fake);
    let (second, second_heard) = watched(&fake);
    fake.focus(FieldKind::Text);
    let count = |heard: &Heard| heard.lock().unwrap().len();
    assert_eq!((count(&first_heard), count(&second_heard)), (1, 1));
    drop(first);
    fake.focus(FieldKind::Password);
    assert_eq!((count(&first_heard), count(&second_heard)), (1, 2));
    assert!(fake.watching(), "the second still watches");
    drop(second);
    assert!(!fake.watching());
}
