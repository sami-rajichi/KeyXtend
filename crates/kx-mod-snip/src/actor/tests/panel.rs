//! The Snip panel: its pages show `[snip]` as last written, each choice is written at once, Change writes the folder
//! the system's dialog chose, Open folder shows it, and the editor's place is kept for next time.

use kx_module_api::ports::{
    AskError, CaptureSheet, FolderError, FolderKey, RecordAsk, Rect, Screen, SnipAside, SnipClick,
    SnipKeyboard, SnipNote, SnipSheet, SnipTab, VideoRow, VideoSheet,
};
use toml::Value;

use super::fakes::{CHOSEN, Fakes, SCREEN, SNIPS, dir};
use super::rig::{Heard, Rig, held, refuse};
use crate::COUNTDOWN_S;

/// The Video page told last.
fn video(rig: &Rig) -> VideoSheet {
    let Some(SnipSheet::Video(sheet)) = rig.page().panel else {
        panic!("the Video page shows");
    };
    sheet
}

/// A rig with the panel open on `tab`.
fn opened(tab: SnipTab) -> Rig {
    let mut rig = Rig::new();
    rig.act(SnipClick::Open);
    rig.act(SnipClick::Tab(tab));
    rig
}

#[test]
fn open_shows_the_capture_page_with_the_keyboard_and_the_save_folder() {
    let mut rig = Rig::new();
    rig.act(SnipClick::Open);
    let capture = CaptureSheet {
        keyboard: SnipKeyboard::Bubble,
        folder: Some(dir(SNIPS)),
    };
    assert_eq!(rig.page().panel, Some(SnipSheet::Capture(capture)));
    rig.act(SnipClick::ClosePanel);
    assert_eq!(rig.page().panel, None);
}

/// A rig with the panel open on its Video page, the rows and their steps put as `(key, value)`.
fn video_with(rows: &[(&str, i64)]) -> Rig {
    let mut rig = Rig::build(|_, host| {
        for (key, value) in rows {
            host.put(key, Value::Integer(*value));
        }
    });
    rig.act(SnipClick::Open);
    rig.act(SnipClick::Tab(SnipTab::Video));
    rig
}

/// Clicks `row`'s plus, or its minus.
fn step(rig: &mut Rig, row: VideoRow, up: bool) {
    rig.act(SnipClick::Step { row, up });
}

#[test]
fn each_video_row_is_written_at_once_and_shown_by_its_step() {
    let rows = [
        ("countdown_s", 3),
        ("countdown_step_s", 2),
        ("max_min", 30),
        ("max_min_step", 10),
    ];
    let mut rig = video_with(&rows);
    for (row, up) in [
        (VideoRow::Countdown, true),
        (VideoRow::Fps, false),
        (VideoRow::Longest, true),
        (VideoRow::Mic, true),
        (VideoRow::Sound, true),
    ] {
        step(&mut rig, row, up);
    }
    let slow = *RecordAsk::FPS.first().unwrap();
    let saved = rig.saved();
    assert_eq!((saved.countdown_s, saved.fps, saved.max_min), (5, slow, 40));
    assert!(saved.mic_on && saved.sound_on);
    let sheet = video(&rig);
    assert_eq!((sheet.countdown_s, sheet.fps, sheet.max_min), (5, slow, 40));
    assert!(sheet.mic_on && sheet.sound_on);
}

#[test]
fn the_frame_rate_row_picks_the_first_or_the_last_rate() {
    let mut rig = opened(SnipTab::Video);
    for (up, rate) in [
        (false, RecordAsk::FPS.first()),
        (true, RecordAsk::FPS.last()),
    ] {
        rig.act(SnipClick::Step {
            row: VideoRow::Fps,
            up,
        });
        assert_eq!(Some(&rig.saved().fps), rate);
    }
}

#[test]
fn a_row_at_its_end_says_so_and_goes_no_further() {
    let most = *COUNTDOWN_S.end();
    let mut rig = video_with(&[("countdown_s", i64::from(most))]);
    assert!(video(&rig).countdown.high);
    step(&mut rig, VideoRow::Countdown, true);
    assert_eq!(video(&rig).countdown_s, most);
    let mut rig = video_with(&[("max_min", 7), ("max_min_step", 5)]);
    step(&mut rig, VideoRow::Longest, true);
    assert_eq!(video(&rig).max_min, 10, "onto the step's multiples");
    step(&mut rig, VideoRow::Longest, false);
    assert_eq!(video(&rig).max_min, 5);
}

#[test]
fn the_keyboard_choice_is_written_and_the_next_snip_follows_it() {
    let mut rig = opened(SnipTab::Capture);
    rig.act(SnipClick::Keyboard(SnipKeyboard::Hidden));
    assert_eq!(rig.saved().keyboard, SnipKeyboard::Hidden);
    rig.key();
    let aside = SnipAside::Hidden;
    assert!(rig.sides().contains(&Heard::Aside(aside)));
}

#[test]
fn clicks_on_a_closed_panel_change_nothing() {
    let mut rig = Rig::new();
    let before = rig.actor.settings.clone();
    step(&mut rig, VideoRow::Fps, false);
    rig.act(SnipClick::Keyboard(SnipKeyboard::Shown));
    assert_eq!(rig.actor.settings, before);
}

#[test]
fn open_folder_shows_the_save_folder_in_the_file_manager() {
    let mut rig = opened(SnipTab::Capture);
    rig.act(SnipClick::Folder(FolderKey::Open));
    assert_eq!(rig.fake.reveal.shown(), [dir(SNIPS)]);
}

#[test]
fn change_writes_the_folder_the_dialog_chose_and_cancel_keeps_it() {
    let mut rig = Rig::build(|fake, _| {
        fake.dialogs.queue_folder(Ok(Some(dir(CHOSEN))));
        fake.dialogs.queue_folder(Ok(None));
        fake.dialogs.queue_folder(Err(AskError::Gone));
    });
    rig.act(SnipClick::Open);
    rig.act(SnipClick::Folder(FolderKey::Change));
    assert_eq!(rig.saved().save_dir, CHOSEN);
    for _ in 0..2 {
        rig.act(SnipClick::Folder(FolderKey::Change));
        assert_eq!(rig.saved().save_dir, CHOSEN);
    }
    assert_eq!(
        rig.fake.dialogs.folders(),
        [dir(SNIPS), dir(CHOSEN), dir(CHOSEN)]
    );
    let limits = rig.actor.settings.pick_limits();
    assert_eq!(rig.fake.dialogs.limits(), [limits; 3], "the settings' own");
    let Some(SnipSheet::Capture(capture)) = rig.page().panel else {
        panic!("the Capture page shows");
    };
    assert_eq!(capture.folder, Some(dir(CHOSEN)));
}

#[test]
fn a_refused_folder_or_a_folder_window_that_fails_says_so() {
    for (answer, note) in [
        (Err(AskError::BadPath), SnipNote::PlaceRefused),
        (Err(AskError::Gone), SnipNote::FolderFailed),
    ] {
        let mut rig = Rig::build(|fake, _| fake.dialogs.queue_folder(answer));
        rig.act(SnipClick::Open);
        rig.act(SnipClick::Folder(FolderKey::Change));
        assert_eq!(rig.page().note, Some(note));
        assert!(rig.actor.settings.save_dir.is_empty(), "kept");
    }
    let mut rig = opened(SnipTab::Capture);
    rig.fake.reveal.set_error(Some(FolderError::Refused(5)));
    rig.act(SnipClick::Folder(FolderKey::Open));
    assert_eq!(rig.page().note, Some(SnipNote::FolderFailed));
    let mut rig = opened(SnipTab::Capture);
    rig.actor.ports.spawn = refuse;
    for key in [FolderKey::Open, FolderKey::Change] {
        rig.act(SnipClick::Folder(key));
        assert_eq!(rig.page().note, Some(SnipNote::FolderFailed), "{key:?}");
    }
}

#[test]
fn the_view_tells_the_folder_dialog_until_it_answers() {
    let mut rig = opened(SnipTab::Capture);
    assert!(!rig.page().dialog);
    rig.actor.ports.spawn = held;
    rig.act(SnipClick::Folder(FolderKey::Change));
    assert!(rig.page().dialog, "the folder dialog shows");
    rig.drain();
    assert!(!rig.page().dialog, "it answered");
}

#[test]
fn a_hidden_run_has_no_folder_to_show_or_change() {
    let mut rig = Rig::with(Fakes::hidden(), |_, _| {});
    rig.act(SnipClick::Open);
    rig.act(SnipClick::Folder(FolderKey::Open));
    rig.act(SnipClick::Folder(FolderKey::Change));
    assert!(rig.fake.reveal.shown().is_empty());
    assert!(rig.fake.dialogs.folders().is_empty());
    let Some(SnipSheet::Capture(capture)) = rig.page().panel else {
        panic!("the Capture page shows");
    };
    assert_eq!(capture.folder, None);
}

#[test]
fn the_editors_place_is_kept_for_next_time() {
    let mut rig = Rig::new();
    let rect = Rect {
        x: 8,
        y: 4,
        w: 40,
        h: 30,
    };
    let screens = vec![Screen {
        rect: SCREEN,
        dpi: 96,
        primary: true,
    }];
    rig.act(SnipClick::Placed {
        rect,
        full: None,
        screens,
    });
    assert_eq!(rig.saved().editor, Some(rect));
}
