//! Follow-the-clear and skipped apps: a clear by the same app within the window deletes the last kept entry once;
//! another app, an unknown app or a late clear deletes nothing; skipped apps match by id, ignoring case.

use kx_module_api::Clock;
use kx_platform_fake::FakeClock;

use super::*;
use crate::fixture::{FOLLOW, app};

/// Ten seconds.
const TEN_S: Duration = Duration::from_secs(10);
/// One millisecond.
const ONE_MS: Duration = Duration::from_millis(1);

/// A follow with no skipped app, and its clock at zero.
fn follow() -> (Follow, FakeClock) {
    (
        Follow::new(FOLLOW, Vec::new()),
        FakeClock::new(Mono::default()),
    )
}

#[test]
fn a_clear_by_the_same_app_within_the_window_deletes_once() {
    let (mut f, clock) = follow();
    f.kept(7, Some(&app("vault.exe")), clock.now());
    clock.advance(FOLLOW);
    assert_eq!(f.cleared(Some(&app("vault.exe")), clock.now()), Some(7));
    assert_eq!(
        f.cleared(Some(&app("vault.exe")), clock.now()),
        None,
        "followed once"
    );
}

#[test]
fn a_clear_after_the_window_deletes_nothing() {
    let (mut f, clock) = follow();
    f.kept(7, Some(&app("vault.exe")), clock.now());
    clock.advance(FOLLOW + ONE_MS);
    assert_eq!(f.cleared(Some(&app("vault.exe")), clock.now()), None);
}

#[test]
fn a_clear_by_another_app_deletes_nothing_and_the_follow_goes_on() {
    let (mut f, clock) = follow();
    f.kept(7, Some(&app("vault.exe")), clock.now());
    clock.advance(TEN_S);
    assert_eq!(f.cleared(Some(&app("notepad.exe")), clock.now()), None);
    clock.advance(TEN_S);
    assert_eq!(f.cleared(Some(&app("vault.exe")), clock.now()), Some(7));
}

#[test]
fn an_unknown_app_is_never_followed() {
    let (mut f, clock) = follow();
    f.kept(7, None, clock.now());
    assert_eq!(f.cleared(None, clock.now()), None);
    assert_eq!(f.cleared(Some(&app("vault.exe")), clock.now()), None);
    f.kept(8, Some(&app("vault.exe")), clock.now());
    assert_eq!(f.cleared(None, clock.now()), None);
    assert_eq!(f.cleared(Some(&app("vault.exe")), clock.now()), Some(8));
}

#[test]
fn app_ids_match_ignoring_case() {
    let (mut f, clock) = follow();
    f.kept(7, Some(&app("KeePass.EXE")), clock.now());
    assert_eq!(f.cleared(Some(&app("keepass.exe")), clock.now()), Some(7));
}

#[test]
fn a_newer_kept_entry_replaces_the_followed_one() {
    let (mut f, clock) = follow();
    f.kept(7, Some(&app("vault.exe")), clock.now());
    f.kept(8, Some(&app("notepad.exe")), clock.now());
    assert_eq!(f.cleared(Some(&app("vault.exe")), clock.now()), None);
    assert_eq!(f.cleared(Some(&app("notepad.exe")), clock.now()), Some(8));
}

#[test]
fn skipped_apps_match_the_id_ignoring_case() {
    let f = Follow::new(FOLLOW, vec!["KeePassXC.exe".to_owned()]);
    assert!(f.skips(Some(&app("keepassxc.EXE"))));
    assert!(!f.skips(Some(&app("notepad.exe"))));
    assert!(!f.skips(None), "an unknown app is not skipped");
    let named = AppRef {
        id: "other.exe".to_owned(),
        name: "KeePassXC.exe".to_owned(),
    };
    assert!(!f.skips(Some(&named)), "the id counts, not the name");
    assert!(
        !follow().0.skips(Some(&app("keepassxc.exe"))),
        "skip_apps starts empty"
    );
}

#[test]
fn debug_hides_the_app_ids() {
    let mut f = Follow::new(FOLLOW, vec!["skipme.exe".to_owned()]);
    f.kept(1, Some(&app("vault.exe")), Mono::default());
    let shown = format!("{f:?}");
    assert!(
        !shown.contains("skipme") && !shown.contains("vault"),
        "{shown}"
    );
}
