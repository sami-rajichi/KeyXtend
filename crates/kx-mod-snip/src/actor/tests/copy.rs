//! Copy: a picture goes to the clipboard as its PNG and its bitmap, kept by both histories; a video as a file
//! reference, marked copied, re-timed first at another speed. A busy clipboard is tried again as `[snip]` says, then a
//! note, and the strip stays (edge 5).

use std::path::PathBuf;
use std::time::Duration;

use kx_module_api::Clock;
use kx_module_api::ports::{
    BGRX_BYTES, BarKey, ClipError, ClipPart, ClipWait, ImageCodec, Keep, MB, PictureError, Point,
    Rect, SaveError, SnipClick, SnipNote, SnipStage, Speed, StripKey,
};
use kx_platform_fake::WriteKind;
use toml::Value;

use super::fakes::{Fakes, PNG};
use super::rig::{Rig, refuse};

/// The codecs of an image write's parts, in order.
fn codecs(parts: &[ClipPart]) -> Vec<ImageCodec> {
    parts
        .iter()
        .filter_map(|p| match p {
            ClipPart::Image { codec, .. } => Some(*codec),
            _ => None,
        })
        .collect()
}

/// The paths of a file write.
fn paths(parts: &[ClipPart]) -> Vec<PathBuf> {
    match parts {
        [ClipPart::Files(paths)] => paths.expose().clone(),
        _ => Vec::new(),
    }
}

#[test]
fn copy_writes_the_pictures_png_and_bitmap_kept_by_both_histories_then_ends() {
    let mut rig = Rig::new();
    rig.snap();
    rig.keep(Keep::Copy);
    let writes = rig.fake.clip.writes();
    assert_eq!(rig.fake.clip.kinds(), [WriteKind::Kept]);
    assert_eq!(codecs(&writes[0]), [ImageCodec::Png, ImageCodec::Dib]);
    let ClipPart::Image { bytes, .. } = &writes[0][0] else {
        panic!("a PNG first");
    };
    assert_eq!(bytes.expose(), &PNG);
    let ClipPart::Image { bytes: dib, .. } = &writes[0][1] else {
        panic!("a bitmap next");
    };
    assert_eq!(
        dib.expose().len(),
        40 + 20 * 3 * 10,
        "20 by 10 pixels, rows on four bytes"
    );
    assert_eq!(rig.fake.clip.last_wait(), Some(rig.actor.held.copy_wait()));
    let SnipStage::Strip(taken) = rig.stage() else {
        panic!("the strip ticks");
    };
    assert_eq!(taken.done, Some(Keep::Copy));
    rig.advance(Duration::from_secs(2));
    assert_eq!(rig.stage(), SnipStage::Idle, "then the snip ends");
}

#[test]
fn the_png_is_made_with_a_cap_and_a_deadline_sized_from_the_pixels() {
    let mut rig = Rig::new();
    rig.snap();
    rig.keep(Keep::Copy);
    let config = rig.fake.pictures.config().unwrap();
    let ask = rig.fake.pictures.last_encode().unwrap();
    assert_eq!((ask.shot.width(), ask.shot.height()), (20, 10));
    assert_eq!(config.memory_mb, ask.memory_mb());
    assert_eq!(config.max_pixels, 200);
    assert!(config.deadline_ms >= rig.actor.held.encode_ms);
    assert!(
        !rig.fake.pictures.running(),
        "the worker's session is dropped once done"
    );
}

#[test]
fn the_pngs_waits_are_the_ones_snip_sets() {
    let big = Rect {
        x: 0,
        y: 0,
        w: 1_024,
        h: 512,
    };
    let mut rig = Rig::build(|fake, host| {
        fake.freeze.set_screens(vec![(big, 96)]);
        for (key, value) in [
            ("encode_ms", 2_000),
            ("encode_mb_ms", 50),
            ("encode_idle_s", 9),
            ("encode_restart_ms", 700),
        ] {
            host.put(key, Value::Integer(value));
        }
    });
    rig.key();
    rig.act(SnipClick::Bar(BarKey::Full));
    rig.act(SnipClick::Spot(Point { x: 1, y: 1 }));
    rig.keep(Keep::Copy);
    let config = rig.fake.pictures.config().unwrap();
    let bytes = u64::try_from(big.w * big.h).unwrap() * u64::try_from(BGRX_BYTES).unwrap();
    let mb = u32::try_from(bytes / MB).unwrap();
    let waits = (config.deadline_ms, config.idle_s, config.restart_ms);
    assert_eq!(waits, (2_000 + 50 * mb, 9, 700));
    assert_eq!(
        config.restart_max_ms, 700,
        "each Encode opens its own session"
    );
}

#[test]
fn a_picture_without_its_png_is_copied_as_a_bitmap_alone() {
    let mut rig = Rig::with(Fakes::making(Err(PictureError::Timeout)), |_, _| {});
    rig.snap();
    rig.keep(Keep::Copy);
    assert_eq!(codecs(&rig.fake.clip.writes()[0]), [ImageCodec::Dib]);
}

#[test]
fn the_strips_copy_puts_a_video_on_the_clipboard_as_its_file_which_stays_after_close() {
    let mut rig = Rig::new();
    rig.recorded(2_000);
    rig.keep(Keep::Copy);
    let run = &rig.fake.recorder.runs()[0];
    let writes = rig.fake.clip.writes();
    assert_eq!(paths(&writes[0]), [run.file.expose().to_path_buf()]);
    assert_eq!(rig.fake.clip.kinds(), [WriteKind::Kept]);
    rig.advance(Duration::from_secs(2));
    assert!(
        matches!(rig.stage(), SnipStage::Strip(_)),
        "a video's strip stays after its Copy"
    );
    rig.act(SnipClick::Strip(StripKey::Close));
    assert_eq!(rig.stage(), SnipStage::Idle, "Close ends it unasked");
    assert!(
        rig.fake.folder.stored(&run.file).is_some(),
        "a copied video stays"
    );
}

#[test]
fn a_sped_up_copy_puts_the_retimed_file_on_the_clipboard() {
    let mut rig = Rig::new();
    rig.recorded(4_000);
    rig.act(SnipClick::Strip(StripKey::Speed));
    rig.keep(Keep::Copy);
    let speeds = rig.fake.recorder.speeds();
    assert_eq!(speeds.len(), 1);
    assert_eq!(speeds[0].speed, Speed::new(2).unwrap());
    let held = &rig.actor.held;
    assert_eq!(
        (speeds[0].min_free_mb, speeds[0].job_mb, speeds[0].waits),
        (held.min_free_mb, held.record_mb, held.record_waits())
    );
    let files = rig.fake.folder.files();
    let copy = files
        .iter()
        .find(|f| *f != &speeds[0].from)
        .unwrap()
        .clone();
    assert_eq!(
        paths(&rig.fake.clip.writes()[0]),
        [copy.expose().to_path_buf()]
    );
    assert_eq!(
        rig.fake.folder.marks(),
        std::slice::from_ref(&copy),
        "the copy is marked, not the recording"
    );
    rig.act(SnipClick::Strip(StripKey::Close));
    assert_eq!(
        rig.fake.folder.files(),
        [copy],
        "the copy stays, the recording goes"
    );
}

#[test]
fn the_clipboard_wait_is_the_one_snip_sets() {
    let mut rig = Rig::build(|_, host| {
        host.put("copy_ms", Value::Integer(600));
        host.put("copy_poll_ms", Value::Integer(20));
    });
    rig.snap();
    rig.keep(Keep::Copy);
    let wait = ClipWait {
        open_ms: 600,
        poll_ms: 20,
    };
    assert_eq!(rig.fake.clip.last_wait(), Some(wait));
}

/// The pause before a busy clipboard's next try, as `[snip]` says.
fn pause(rig: &Rig) -> Duration {
    Duration::from_millis(u64::from(rig.actor.held.copy_again_ms))
}

#[test]
fn edge_5_a_busy_clipboard_is_tried_again_as_set_then_a_note_and_the_strip_stays() {
    const AGAIN: u8 = 3;
    let mut rig = Rig::build(|fake, host| {
        host.put("copy_again", Value::Integer(AGAIN.into()));
        host.put("copy_again_ms", Value::Integer(700));
        for _ in 0..=AGAIN {
            fake.clip.queue(Err(ClipError::Busy));
        }
    });
    rig.snap();
    rig.keep(Keep::Copy);
    for _ in 0..AGAIN {
        assert_eq!(rig.page().note, None, "not yet");
        let tried = rig.fake.clip.tries();
        rig.advance(Duration::from_millis(699));
        assert_eq!(rig.fake.clip.tries(), tried, "not before its pause");
        rig.advance(Duration::from_millis(1));
        assert_eq!(rig.fake.clip.tries(), tried + 1);
    }
    assert_eq!(rig.fake.clip.tries(), usize::from(AGAIN) + 1);
    assert_eq!(rig.page().note, Some(SnipNote::ClipBusy));
    let SnipStage::Strip(taken) = rig.stage() else {
        panic!("the strip stays");
    };
    assert_eq!(taken.done, None);
    rig.keep(Keep::Copy);
    assert_eq!(rig.fake.clip.writes().len(), 1, "a later Copy goes through");
}

#[test]
fn a_clipboard_refusing_for_another_reason_says_the_copy_failed_at_once() {
    let mut rig = Rig::build(|fake, _| fake.clip.queue(Err(ClipError::Refused)));
    rig.snap();
    rig.keep(Keep::Copy);
    assert_eq!(rig.fake.clip.tries(), 1, "no try again");
    assert_eq!(rig.page().note, Some(SnipNote::CopyFailed));
    let mut rig = Rig::new();
    rig.snap();
    rig.actor.ports.spawn = refuse;
    rig.keep(Keep::Copy);
    assert_eq!(
        rig.page().note,
        Some(SnipNote::CopyFailed),
        "no job started"
    );
}

#[test]
fn a_copied_video_is_marked_so_the_sweep_keeps_it_and_a_failed_mark_still_copies() {
    let mut rig = Rig::new();
    rig.recorded(2_000);
    rig.keep(Keep::Copy);
    let recording = rig.fake.recorder.runs()[0].file.clone();
    assert_eq!(
        rig.fake.folder.marked(&recording),
        Some(rig.fake.clock.now())
    );
    let mut rig = Rig::new();
    rig.recorded(2_000);
    rig.fake.folder.set_mark_error(Some(SaveError::Io));
    rig.keep(Keep::Copy);
    assert_eq!(rig.fake.clip.writes().len(), 1, "copied unmarked");
    assert!(matches!(rig.stage(), SnipStage::Strip(t) if t.done == Some(Keep::Copy)));
}

#[test]
fn edge_5_a_clipboard_let_go_on_a_later_try_copies() {
    let mut rig = Rig::build(|fake, _| fake.clip.queue(Err(ClipError::Busy)));
    rig.snap();
    rig.keep(Keep::Copy);
    assert!(rig.fake.clip.writes().is_empty());
    rig.advance(pause(&rig));
    assert_eq!(rig.fake.clip.writes().len(), 1);
    assert_eq!(
        codecs(&rig.fake.clip.writes()[0]),
        [ImageCodec::Png, ImageCodec::Dib]
    );
    assert!(matches!(rig.stage(), SnipStage::Strip(t) if t.done == Some(Keep::Copy)));
}
