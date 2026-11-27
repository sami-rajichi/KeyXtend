//! Every end: the keyboard comes back and the key goes dark once, as the keyboard choice stepped aside, and the snip's
//! videos are forgotten. A re-timed copy never written needs no delete, and a snip keeps the settings it started with.

use std::time::Duration;

use kx_module_api::ports::{
    AskKey, BarKey, EditKey, FileKind, FreezeError, Keep, PanelKey, RecKey, RecordAsk, RecordEnd,
    RecordError, SnipAside, SnipClick, SnipLit, SnipNote, SnipStage, SnipTab, StripKey, VideoRow,
};
use kx_platform_fake::{Pick, RecordCall};
use toml::Value;

use super::fakes::{CHOSEN, FAR, NEAR, file};
use super::rig::{Heard, Rig};

/// The lights and steps aside a whole snip tells: on and aside, then back and dark.
pub(super) const WHOLE: [Heard; 4] = [
    Heard::Lit(SnipLit::On),
    Heard::Aside(SnipAside::Bubble),
    Heard::Lit(SnipLit::Off),
    Heard::Aside(SnipAside::Back),
];

/// The lights and steps aside told since the start's dark key.
pub(super) fn since_start(rig: &Rig) -> Vec<Heard> {
    rig.sides().split_off(1)
}

/// A way a snip ends, by name.
type End = (&'static str, fn(&mut Rig));

/// Closes the strip without keeping its snip.
fn close(rig: &mut Rig) {
    rig.act(SnipClick::Strip(StripKey::Close));
    rig.act(SnipClick::Ask(AskKey::Drop));
}

/// The ends a picture or a video reaches by the strip, the bar and the recorder.
const ENDS: [End; 10] = [
    ("Snip key while picking", |r| {
        r.key();
        r.key();
    }),
    ("Esc while picking", |r| {
        r.key();
        r.line(PanelKey::Esc);
    }),
    ("Copy", |r| {
        r.snap();
        r.keep(Keep::Copy);
        r.advance(Duration::from_secs(2));
    }),
    ("Save", |r| {
        let chosen = file(CHOSEN, "shot.png", FileKind::Png);
        r.fake.dialogs.queue_save(Pick::File(chosen));
        r.snap();
        r.keep(Keep::Save);
        r.advance(Duration::from_secs(2));
    }),
    ("Don't save", |r| {
        r.snap();
        close(r);
    }),
    ("Stop, then Close", |r| {
        r.recorded(1_000);
        close(r);
    }),
    ("the longest video, then Close", |r| {
        r.record();
        assert!(r.fake.recorder.end(RecordEnd::Longest));
        r.pump();
        close(r);
    }),
    ("Cancel", |r| {
        r.record();
        r.act(SnipClick::Record(RecKey::Cancel));
    }),
    ("a dead worker", |r| {
        r.record();
        assert!(r.fake.recorder.die());
        r.pump();
    }),
    ("a refused freeze", |r| {
        r.fake.freeze.set_error(Some(FreezeError::Refused(5)));
        r.key();
    }),
];

#[test]
fn every_end_brings_the_keyboard_back_darkens_the_key_once_and_forgets_its_videos() {
    for (end, run) in ENDS {
        let mut rig = Rig::new();
        run(&mut rig);
        assert_eq!(rig.stage(), SnipStage::Idle, "{end}");
        assert_eq!(since_start(&rig), WHOLE, "{end}");
        assert_eq!(rig.actor.files.count(), 0, "{end}");
    }
}

#[test]
fn the_editor_brings_the_keyboard_back_and_its_end_tells_nothing_more() {
    let mut rig = Rig::new();
    rig.snap();
    rig.act(SnipClick::Strip(StripKey::Edit));
    assert_eq!(since_start(&rig), WHOLE, "back as the editor opens");
    rig.act(SnipClick::Edit(EditKey::Close));
    rig.act(SnipClick::Ask(AskKey::Drop));
    assert_eq!(rig.stage(), SnipStage::Idle);
    assert_eq!(since_start(&rig), WHOLE);
}

#[test]
fn the_keyboard_steps_aside_as_chosen() {
    let mut rig = Rig::build(|_, host| host.put("keyboard", Value::String("hidden".into())));
    rig.key();
    rig.key();
    let hidden = [
        Heard::Lit(SnipLit::On),
        Heard::Aside(SnipAside::Hidden),
        Heard::Lit(SnipLit::Off),
        Heard::Aside(SnipAside::Back),
    ];
    assert_eq!(since_start(&rig), hidden);
    let mut rig = Rig::build(|_, host| host.put("keyboard", Value::String("shown".into())));
    rig.key();
    rig.key();
    let shown = [Heard::Lit(SnipLit::On), Heard::Lit(SnipLit::Off)];
    assert_eq!(
        since_start(&rig),
        shown,
        "a shown keyboard never steps aside"
    );
}

#[test]
fn a_freeze_past_the_cap_ends_with_its_note_shown_for_note_s() {
    let mut rig = Rig::build(|_, host| host.put("note_s", Value::Integer(6)));
    rig.fake.freeze.set_error(Some(FreezeError::TooBig));
    rig.key();
    let ended = (rig.stage(), rig.page().note);
    assert_eq!(ended, (SnipStage::Idle, Some(SnipNote::TooBig)));
    rig.advance(Duration::from_millis(5_999));
    assert_eq!(rig.page().note, Some(SnipNote::TooBig));
    rig.advance(Duration::from_millis(1));
    assert_eq!(rig.page().note, None);
}

#[test]
fn a_retimed_copy_never_written_says_so_and_needs_no_delete() {
    let mut rig = Rig::build(|fake, _| fake.recorder.fail(RecordCall::Speed, RecordError::Gone));
    rig.recorded(4_000);
    let recording = rig.fake.recorder.runs()[0].file.clone();
    rig.act(SnipClick::Strip(StripKey::Speed));
    rig.keep(Keep::Copy);
    assert_eq!(rig.page().note, Some(SnipNote::SpeedFailed));
    assert!(
        rig.fake.folder.stored(&recording).is_some(),
        "the recording stays"
    );
    rig.act(SnipClick::Strip(StripKey::Close));
    rig.act(SnipClick::Ask(AskKey::Drop));
    assert_eq!(rig.stage(), SnipStage::Idle);
    let removed = rig.fake.folder.removes();
    assert_eq!(removed, [recording], "only the recording had a file");
    assert!(rig.fake.folder.files().is_empty());
    assert_eq!(rig.actor.files.count(), 0, "nothing waits for the copy");
}

/// Lowers the frames a second and the countdown on the panel's Video page, opened from the bar, then closes it.
fn lower_fps_and_countdown(rig: &mut Rig) {
    rig.act(SnipClick::Open);
    rig.act(SnipClick::Tab(SnipTab::Video));
    for row in [VideoRow::Fps, VideoRow::Countdown] {
        rig.act(SnipClick::Step { row, up: false });
    }
    rig.act(SnipClick::ClosePanel);
}

#[test]
fn a_snip_keeps_the_settings_it_started_with() {
    let (fast, slow) = (
        *RecordAsk::FPS.last().unwrap(),
        *RecordAsk::FPS.first().unwrap(),
    );
    let mut rig = Rig::build(|_, host| {
        host.put("fps", Value::Integer(fast.into()));
        host.put("countdown_s", Value::Integer(3));
        host.put("countdown_step_s", Value::Integer(1));
    });
    rig.key();
    lower_fps_and_countdown(&mut rig);
    assert_eq!(
        (rig.actor.settings.fps, rig.actor.settings.countdown_s),
        (slow, 2)
    );
    rig.act(SnipClick::Bar(BarKey::VideoRegion));
    rig.act(SnipClick::Spot(NEAR));
    rig.act(SnipClick::Spot(FAR));
    rig.act(SnipClick::Strip(StripKey::Record));
    rig.advance(Duration::from_secs(2));
    let counting = matches!(rig.stage(), SnipStage::Count(_));
    assert!(counting, "the old countdown runs");
    rig.advance(Duration::from_secs(1));
    assert!(rig.fake.recorder.time());
    rig.pump();
    let first = rig.fake.recorder.starts()[0].fps;
    assert_eq!(first, fast, "the old frames a second");
    rig.act(SnipClick::Record(RecKey::Cancel));
    rig.record();
    let next = rig.fake.recorder.starts()[1].fps;
    assert_eq!(next, slow, "the next snip takes the new ones");
}
