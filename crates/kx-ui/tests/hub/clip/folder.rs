//! Open folder through the hub: it shows the folder the app handed over, one window at a time, and the panel's JSON
//! says whether it can act and, in each language, why not.

use super::{clip, heard, page, row, shown};
use crate::Rig;
use crate::words::speaking;
use kx_platform::ui::RevealError;
use std::path::PathBuf;
use std::time::Duration;

/// The longest a test waits for the file manager's thread.
const WAIT: Duration = Duration::from_secs(5);
/// The history folder the app hands over in these tests.
const FOLDER: &str = r"D:\data\KeyXtend\clipboard";

#[test]
fn open_folder_shows_the_folder_the_app_handed_over() {
    let rig = Rig::new();
    let folder = PathBuf::from(FOLDER);
    assert!(!rig.hub.open_folder(), "none handed over");
    assert!(rig.reveal.shown().is_empty());
    rig.hub.set_clip_folder(folder.clone());
    rig.hub.set_clip_folder(PathBuf::from(r"C:\other"));
    rig.hub.open_folder();
    assert!(rig.reveal.wait_shown(1, WAIT));
    assert_eq!(rig.reveal.shown(), [folder], "the first one stays");
}

/// The refusal's warning is logged on the reveal's own thread, which the log capture does not hear.
#[test]
fn a_refused_folder_ends_its_reveal_without_a_panic() {
    let rig = Rig::new();
    rig.reveal.set_error(Some(RevealError::Refused(5)));
    rig.hub.set_clip_folder(PathBuf::from(FOLDER));
    rig.hub.open_folder();
    assert!(rig.reveal.wait_left(1, WAIT));
    assert_eq!(rig.reveal.shown(), [PathBuf::from(FOLDER)]);
}

#[test]
fn a_double_tap_opens_one_window() {
    let rig = Rig::new();
    rig.hub.set_clip_folder(PathBuf::from(FOLDER));
    rig.reveal.hold(true);
    assert!(rig.hub.open_folder(), "the first tap starts a reveal");
    assert!(rig.reveal.wait_shown(1, WAIT));
    assert!(!rig.hub.open_folder(), "one is in flight");
    assert!(!rig.hub.open_folder());
    rig.reveal.hold(false);
    assert!(rig.reveal.wait_left(1, WAIT));
    assert_eq!(
        rig.reveal.shown().len(),
        1,
        "the taps meanwhile were ignored"
    );
}

#[test]
fn open_folder_can_act_only_once_the_app_hands_a_folder_over() {
    let rig = Rig::new();
    let (bus, acts, _subs) = heard(&rig);
    shown(&rig, &bus, &acts, page(1, vec![row(7, "a")]));
    assert_eq!(
        clip(&rig)["opens"],
        false,
        "a run that saves nothing hands none"
    );
    rig.hub.set_clip_folder(PathBuf::from(FOLDER));
    assert_eq!(clip(&rig)["opens"], true);
}

#[test]
fn readers_hear_why_open_folder_cannot_act_in_each_language() {
    let why = [
        (
            "en",
            "Nothing is saved this time, so there is no folder to open.",
        ),
        (
            "fr",
            "Rien n’est enregistré cette fois, il n’y a donc aucun dossier à ouvrir.",
        ),
        ("ar", "لا يُحفَظ أي شيء هذه المرة، لذا لا يوجد مجلد لفتحه."),
    ];
    for (iso, want) in why {
        let (rig, _host) = speaking(iso, 5);
        let (bus, acts, _subs) = heard(&rig);
        shown(&rig, &bus, &acts, page(1, vec![row(7, "a")]));
        assert_eq!(clip(&rig)["words"]["noFolder"], want, "{iso}");
    }
}
