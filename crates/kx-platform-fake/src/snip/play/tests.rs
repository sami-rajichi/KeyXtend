//! The fake player, held to the port: only a video of the Snips folder plays, a gone one is missing, a refusal comes
//! as set, and every video asked is kept.

use super::*;
use crate::FakeClock;
use crate::snip::Stored;
use crate::snip::tests::{CHOSEN, SNIPS, UNSAVED, WITHIN, file, png, rig};
use kx_module_api::Mono;
use kx_module_api::ports::FileKind;
use std::sync::Arc;
use std::time::Duration;

/// A refusal's code.
const CODE: i32 = 7;

/// A video `1` ms long.
fn video() -> Stored {
    Stored::Video { length_ms: 1 }
}

#[test]
fn only_a_video_of_the_snips_folder_plays() {
    let (_, folder) = rig();
    let player = FakeMediaOpen::new(&folder);
    let recording = file(UNSAVED, "1.mp4", FileKind::Mp4);
    let saved = file(SNIPS, "2.mp4", FileKind::Mp4);
    let outside = file(CHOSEN, "3.mp4", FileKind::Mp4);
    let picture = file(UNSAVED, "4.png", FileKind::Png);
    for held in [&recording, &saved, &outside] {
        folder.put(held.clone(), video());
    }
    folder.put(picture.clone(), png());
    assert_eq!(player.play(&recording), Ok(()));
    assert_eq!(player.play(&saved), Ok(()));
    assert_eq!(player.play(&outside), Err(PlayError::BadPath));
    assert_eq!(player.play(&picture), Err(PlayError::BadPath));
    let gone = file(UNSAVED, "5.mp4", FileKind::Mp4);
    assert_eq!(player.play(&gone), Err(PlayError::Missing));
    let below = file(&format!("{UNSAVED}/old"), "6.mp4", FileKind::Mp4);
    folder.put(below.clone(), video());
    assert_eq!(
        player.play(&below),
        Err(PlayError::BadPath),
        "only directly"
    );
    let asked = [recording, saved, outside, picture, gone, below];
    assert_eq!(player.played(), asked, "refused ones count");
}

#[test]
fn a_refusal_comes_as_set_until_cleared() {
    let (_, folder) = rig();
    let player = FakeMediaOpen::new(&folder);
    let recording = file(UNSAVED, "1.mp4", FileKind::Mp4);
    folder.put(recording.clone(), video());
    player.set_error(Some(PlayError::Refused(CODE)));
    assert_eq!(player.play(&recording), Err(PlayError::Refused(CODE)));
    player.set_error(None);
    assert_eq!(player.play(&recording), Ok(()));
}

#[test]
fn a_test_waits_for_a_play_on_another_thread() {
    let (_, folder) = rig();
    let player = FakeMediaOpen::new(&folder);
    let recording = file(UNSAVED, "1.mp4", FileKind::Mp4);
    folder.put(recording.clone(), video());
    assert!(!player.wait_played(1, Duration::ZERO), "none yet");
    let port: Arc<dyn MediaOpen> = Arc::new(player.clone());
    let module = std::thread::spawn(move || port.play(&recording));
    assert!(player.wait_played(1, WITHIN));
    assert_eq!(module.join().unwrap(), Ok(()));
}

#[test]
fn a_hidden_run_plays_nothing_and_debug_shows_no_path() {
    let clock = FakeClock::new(Mono::default());
    let folder = FakeSnipFolder::hidden(Arc::new(clock));
    let player = FakeMediaOpen::new(&folder);
    let recording = file(UNSAVED, "1.mp4", FileKind::Mp4);
    assert_eq!(player.play(&recording), Err(PlayError::BadPath));
    let shown = format!("{player:?}");
    assert!(shown.contains("FakeMediaOpen"), "{shown}");
    assert!(!shown.contains("Alex"), "{shown}");
}
